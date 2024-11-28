use std::{
    fs,
    io::{self, stdin, stdout, Read, Write},
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
    process::exit,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

use colored::Colorize;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use serde::{Deserialize, Serialize};

use crate::{project::Project, ui};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UbootConfig {
    pub serial: String,
    pub baud_rate: i64,
    pub net: String,
    pub dtb_file: String,
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

        let net_list = interfaces
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

        let i = ui::shell_select("请选择网卡", &net_list);
        let net = interfaces[i].name.clone();

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

pub struct Uboot {}

impl Uboot {
    pub fn run(project: &mut Project, is_check_test: bool) {
        let config = project.config_ref().uboot.clone().unwrap();

        let kernel_bin = project
            .bin_path
            .as_ref()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy();

        let port = serialport::new(config.serial.clone(), 115_200)
            .timeout(Duration::from_millis(10))
            .open()
            .expect("Failed to open port");

        let mut ip = String::new();

        let interfaces = NetworkInterface::show().unwrap();
        for interface in interfaces.iter() {
            if interface.name == config.net {
                let addr_list: Vec<Addr> = interface.addr.to_vec();
                for one in addr_list {
                    if let Addr::V4(v4_if_addr) = one {
                        ip = v4_if_addr.ip.to_string();
                    }
                }
            }
        }

        println!("串口：{}", port.name().unwrap_or_default());
        println!("TFTP: {}", ip);
        println!("内核：{}", kernel_bin);

        let port = Arc::new(Mutex::new(port));
        thread::spawn({
            let port = port.clone();
            move || loop {
                let mut buf = [0u8; 1];
                let _ = stdin().read_exact(&mut buf);
                let _ = port.lock().unwrap().write_all(&buf);
            }
        });

        let out_dir = project.out_dir();

        let boot_cmd = if let Some(dtb) = PathBuf::from(&config.dtb_file).file_name() {
            let dtb_load_addr = "0x90600000";

            let dtb_name = dtb.to_str().unwrap().to_string();

            let ftp_dtb = out_dir.join(&dtb_name);

            let _ = fs::copy(config.dtb_file, ftp_dtb);

            format!(
            "dhcp {dtb_load_addr} {ip}:{dtb_name};fdt addr {dtb_load_addr};bootp {ip}:{kernel_bin};booti $loadaddr - {dtb_load_addr}"
        )
        } else {
            println!("DTB file not provided");
            format!("dhcp $loadaddr {ip}:{kernel_bin};go $loadaddr")
        };

        let mut in_shell = false;
        println!("启动命令：{}", boot_cmd);

        println!("等待 U-Boot 启动...");

        let mut buf = [0u8; 1];
        let mut history = Vec::new();

        loop {
            let res = {
                let mut port = port.lock().unwrap();
                port.read(&mut buf)
            };
            match res {
                Ok(_t) => {
                    let ch = buf[0];
                    if ch == b'\n' && history.last() != Some(&b'\r') {
                        stdout().write_all(b"\r").unwrap();
                        history.push(b'\r');
                    }
                    history.push(ch);

                    if !in_shell {
                        if let Ok(s) = String::from_utf8(history.clone()) {
                            if s.contains("Hit any key to stop autoboot") {
                                in_shell = true;
                                let mut port = port.lock().unwrap();
                                port.write_all(b"a").unwrap();
                                sleep(Duration::from_secs(1));

                                port.write_all(boot_cmd.as_bytes()).unwrap();
                                port.write_all(b"\r\n").unwrap();
                                history.clear();
                            }
                        }
                    }

                    if history.ends_with(b"\r\n") {
                        let s = String::from_utf8(history.to_vec()).unwrap();
                        if in_shell && is_check_test {
                            if s.contains("All tests passed") {
                                exit(0);
                            }
                            if s.contains("panicked at") {
                                println!("{}", "Test failed!".red());
                                exit(1);
                            }
                        }
                    }

                    stdout().write_all(&buf).unwrap();
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                    stdout().flush().unwrap();
                }
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }

    fn run_tftp(file_dir: &Path) {
        use tftpd::{Config, Server};
        println!("启动 TFTP 服务器...");
        println!("文件目录：{}", file_dir.display());
        let mut config = Config::default();
        config.directory = PathBuf::from(file_dir);
        config.send_directory = config.directory.clone();
        config.port = 69;
        config.ip_address = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

        std::thread::spawn(move || {
            let mut server = Server::new(&config).unwrap();
            server.listen();
        });
    }
}
