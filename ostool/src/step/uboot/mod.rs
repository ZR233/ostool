use std::{
    collections::{HashMap, VecDeque},
    fs::{self},
    io::{self, Read, Write},
    path::PathBuf,
    process::exit,
    thread::{self, sleep},
    time::Duration,
};

use colored::Colorize;
use crossterm::event::{self, Event, KeyCode};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use serde::{Deserialize, Serialize};
use uboot_shell::UbootShell;

use crate::{project::Project, step::Tftp, ui};

use super::Step;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct UbootConfig {
    pub serial: String,
    pub baud_rate: i64,
    pub dtb_file: String,
    pub net: Option<Net>,
}

impl UbootConfig {
    pub fn config_by_select() -> Self {
        let ports = serialport::available_ports().expect("No ports found!");
        let names = ports
            .iter()
            .map(|p| p.port_name.to_string())
            .collect::<Vec<_>>();
        let i = ui::shell_select("请选择串口设备", &names);

        let serial = names[i].clone();
        println!("请设置波特率:");

        let baud_rate = loop {
            let mut line = String::new();
            match std::io::stdin().read_line(&mut line) {
                Ok(_) => match line.trim().parse::<i64>() {
                    Ok(n) => break n,
                    Err(_) => println!("请输入数字"),
                },
                Err(_) => println!("请输入有效数字"),
            }
        };

        let interfaces = NetworkInterface::show().unwrap();

        let mut net_list = interfaces
            .iter()
            .map(|i| {
                let mut addr = String::new();
                for o in &i.addr {
                    if let Addr::V4(v4) = o {
                        addr = v4.ip.to_string();
                    }
                }
                format!("[{}] - [{}]", i.name, addr)
            })
            .collect::<Vec<_>>();
        net_list.push("无网络，用串口传输".to_string());

        let i = ui::shell_select("请选择网卡", &net_list);

        let mut net = None;

        if i < interfaces.len() {
            let interface = interfaces[i].name.clone();

            println!("请输入板卡IP，留空使用DHCP获取：");
            let board_ip_input = std::io::stdin()
                .lines()
                .next()
                .unwrap()
                .expect("Invalid input")
                .trim()
                .to_string();
            let mut some_net = Net {
                interface,
                board_ip: None,
            };

            if !board_ip_input.is_empty() {
                some_net.board_ip = Some(board_ip_input);
            }

            net = Some(some_net)
        }

        println!("请输入dtb文件路径:");
        let dtb_file = std::io::stdin()
            .lines()
            .next()
            .unwrap()
            .expect("Invalid input");

        Self {
            serial,
            baud_rate,
            net,
            dtb_file,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Net {
    pub interface: String,
    pub board_ip: Option<String>,
}

pub struct Uboot {
    is_check_test: bool,
}

impl Uboot {
    pub fn new_boxed(is_check_test: bool) -> Box<Self> {
        Box::new(Self { is_check_test })
    }
}

impl Step for Uboot {
    fn run(&mut self, project: &mut Project) -> anyhow::Result<()> {
        let config = project.config_ref().uboot.clone().unwrap();

        let kernel_bin = project
            .kernel
            .as_ref()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let out_dir = project.out_dir();
        println!("内核：{}", kernel_bin);
        let boot_cmd;
        let mut fdtfile = String::new();
        let mut env_map = HashMap::new();

        if let Some(dtb) = PathBuf::from(&config.dtb_file).file_name() {
            let dtb_name = dtb.to_str().unwrap().to_string();

            fdtfile = dtb_name.to_string();

            let ftp_dtb = out_dir.join(&dtb_name);

            let _ = fs::copy(&config.dtb_file, ftp_dtb);
        }

        match &config.net {
            Some(net) => {
                Tftp::new_boxed().run(project).unwrap();

                let mut ip_string = String::new();

                let interfaces = NetworkInterface::show().unwrap();
                for interface in interfaces.iter() {
                    if interface.name == net.interface {
                        let addr_list: Vec<Addr> = interface.addr.to_vec();
                        for one in addr_list {
                            if let Addr::V4(v4_if_addr) = one {
                                ip_string = v4_if_addr.ip.to_string();
                            }
                        }
                    }
                }

                println!("TFTP : {}", ip_string);

                env_map.insert("serverip", ip_string.clone());

                let mut boot_cmd_base = String::new();
                if let Some(board_ip) = &net.board_ip {
                    env_map.insert("ipaddr", board_ip.to_string());
                } else {
                    boot_cmd_base = format!("{boot_cmd_base}dhcp;");
                }
                boot_cmd = if !fdtfile.is_empty() {
                    format!("{boot_cmd_base}tftp $loadaddr {ip_string}:$bootfile;tftp $fdt_addr {ip_string}:$fdtfile;fdt addr $fdt_addr;booti $loadaddr - $fdt_addr")
                } else {
                    println!("DTB file not provided");
                    format!("{boot_cmd_base}tftp $loadaddr {ip_string}:$bootfile;dcache flush;go $loadaddr")
                };
            }
            None => {
                boot_cmd = if fdtfile.is_empty() {
                    println!("DTB file not provided");
                    "dcache flush;go $loadaddr".to_string()
                } else {
                    "fdt addr $fdt_addr;booti $loadaddr - $fdt_addr".to_string()
                };
            }
        }
        println!("启动命令：{}", boot_cmd);
        let kernel_size = fs::metadata(project.kernel.as_ref().unwrap())
            .unwrap()
            .len() as usize;

        let rx = serialport::new(&config.serial, config.baud_rate as _)
            .timeout(Duration::from_millis(3000))
            .open()
            .map_err(|e| format!("无法打开串口 {}: {:?}", config.serial, e))
            .unwrap();

        let tx = rx.try_clone().unwrap();

        println!(
            "串口：{}, {}",
            rx.name().unwrap_or_default(),
            rx.baud_rate().unwrap()
        );

        println!("等待 U-Boot 启动...");
        let mut uboot = UbootShell::new(tx, rx).unwrap();

        println!();
        println!("{}", "Uboot shell ok".green());

        let loadaddr = uboot.env_int("loadaddr").unwrap_or_else(|_e| {
            println!("$loadaddr not found");

            let loadaddr = uboot
                .env_int("kernel_addr_r")
                .expect("kernel_addr_r not found");

            println!("$loadaddr set to {:#x}", loadaddr);

            uboot.set_env("loadaddr", format!("{loadaddr:#x}")).unwrap();

            loadaddr
        });

        let mut fdt_addr = loadaddr + kernel_size + 0x100000;
        fdt_addr = (fdt_addr + 0xFFF) & !0xFFF;

        if let Ok(addr) = uboot.env_int("fdt_addr") {
            fdt_addr = addr;
        } else {
            uboot
                .set_env("fdt_addr", format!("{:#x}", fdt_addr))
                .unwrap();
        }

        uboot.set_env("autoload", "no").unwrap();

        uboot.set_env("bootfile", kernel_bin).unwrap();

        if !fdtfile.is_empty() {
            uboot.set_env("fdtfile", &fdtfile).unwrap();
        }

        for (k, v) in env_map {
            uboot.set_env(k, v).unwrap();
        }

        if config.net.is_none() {
            uboot_load(&mut uboot, loadaddr, project.kernel.as_ref().unwrap());

            if !config.dtb_file.is_empty() {
                sleep(Duration::from_millis(500));
                uboot_load(&mut uboot, fdt_addr, &config.dtb_file);
            }
        }
        sleep(Duration::from_millis(500));
        println!("boot up");
        uboot.cmd_without_reply(&boot_cmd, false).unwrap();

        if !self.is_check_test {
            let mut port_tx = uboot.tx.take().unwrap();

            thread::spawn(move || loop {
                if let Ok(Event::Key(key)) = event::read() {
                    if matches!(key.kind, event::KeyEventKind::Release) {
                        match key.code {
                            KeyCode::Char(ch) => {
                                port_tx.write_all(&[ch as u8]).unwrap();
                            }
                            KeyCode::Backspace => {
                                port_tx.write_all(&[127]).unwrap();
                            }
                            KeyCode::Enter => {
                                port_tx.write_all(b"\r\n").unwrap();
                            }
                            _ => {}
                        }
                    }
                }
            });
        }

        let mut history = VecDeque::new();
        let mut line_tmp = Vec::new();

        let mut buff = [0u8; 1];

        let need_check_test = self.is_check_test;

        for byte in uboot.rx.as_mut().unwrap().bytes() {
            match byte {
                Ok(b) => {
                    if b == b'\n' && buff[0] != b'\r' {
                        io::stdout().write_all(b"\r").unwrap();
                        line_tmp.push(b'\r');
                    }
                    buff[0] = b;
                    line_tmp.push(b);

                    if line_tmp.ends_with(b"\r\n") {
                        let line = String::from_utf8_lossy(&line_tmp).to_string();
                        if need_check_test {
                            if line.contains("All tests passed") {
                                exit(0);
                            }
                            if line.contains("panicked at") {
                                println!("{}", "Test failed!".red());
                                exit(1);
                            }
                        }

                        history.push_back(line);
                        if history.len() > 100 {
                            history.pop_front();
                        }

                        line_tmp.clear();
                        io::stdout().flush().unwrap();
                    }

                    io::stdout().write_all(&buff).unwrap();
                    // io::stdout().flush().unwrap();
                }
                Err(e) => match e.kind() {
                    io::ErrorKind::TimedOut => sleep(Duration::from_micros(1)),
                    _ => panic!("{}", e),
                },
            }
        }

        Ok(())
    }
}

fn uboot_load(uboot: &mut UbootShell, addr: usize, file: impl Into<PathBuf>) {
    println!("{}", "\r\nsend file".green());

    let pb = ProgressBar::new(100);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn core::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

    let res = uboot
        .loady(addr, file, |x, a| {
            pb.set_length(a as _);
            pb.set_position(x as _);
        })
        .unwrap();

    pb.finish_with_message("upload done");

    println!("{}", res);
    println!("send ok");
}
