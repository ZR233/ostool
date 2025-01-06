use std::{
    collections::VecDeque,
    fs::{self},
    io::{self, stdout, Read, Write},
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
    process::exit,
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};

use colored::Colorize;
use crossterm::event::{self, Event, KeyCode};
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use serde::{Deserialize, Serialize};
use serialport::SerialPort;

use crate::{project::Project, ui};

use super::Step;

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

pub struct Uboot {
    is_check_test: bool,
}

impl Uboot {
    pub fn new_boxed(is_check_test: bool) -> Box<Self> {
        Box::new(Self { is_check_test })
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

impl Step for Uboot {
    fn run(&mut self, project: &mut Project) -> anyhow::Result<()> {
        let config = project.config_ref().uboot.clone().unwrap();

        let kernel_bin = project
            .bin_path
            .as_ref()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy();

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

        println!("TFTP: {}", ip);
        println!("内核：{}", kernel_bin);

        let out_dir = project.out_dir();

        let boot_cmd = if let Some(dtb) = PathBuf::from(&config.dtb_file).file_name() {
            // mkimage(project, &config.dtb_file);

            // let dtb_load_addr = "0x90600000";

            let dtb_name = dtb.to_str().unwrap().to_string();

            let ftp_dtb = out_dir.join(&dtb_name);

            let _ = fs::copy(config.dtb_file, ftp_dtb);

            // format!(
            // "dhcp {dtb_load_addr} {ip}:{dtb_name};fdt addr {dtb_load_addr};bootp {ip}:{kernel_bin};booti $loadaddr - {dtb_load_addr}"

            format!(
            "dhcp $fdt_addr {ip}:{dtb_name};fdt addr $fdt_addr;bootp {ip}:{kernel_bin};booti $loadaddr - $fdt_addr"
        )
        } else {
            println!("DTB file not provided");
            format!("dhcp $loadaddr {ip}:{kernel_bin};dcache flush;go $loadaddr")
        };
        Self::run_tftp(&out_dir);
        println!("启动命令：{}", boot_cmd);

        println!("等待 U-Boot 启动...");

        let kernel_size = fs::metadata(project.to_load_kernel.as_ref().unwrap())
            .unwrap()
            .len();

        let sh = Arc::new(UbootShell {
            boot_cmd,
            need_check_test: self.is_check_test,
            kernel_size: kernel_size as _,
        });

        sh.run(&config.serial, config.baud_rate as _);
        Ok(())
    }
}

// fn mkimage(project: &Project, fdt: &str) {
//     let mut cmd = Command::new("mkimage");

//     let bin = project.bin_path.as_ref().unwrap().to_string_lossy();

//     let d = format!("\"{}\":\"{}\"", bin, fdt);
//     // let d = format!("{}", bin);

//     cmd.args(["-A", "arm"])
//         .args(["-O", "linux"])
//         .args(["-T", "multi"])
//         .args(["-C", "none"])
//         .args(["-n", "Speareal Kernel Image"])
//         // .args(["-a", "0x1000000"])
//         // .args(["-e", "0x1000000"])
//         .arg("-d")
//         // .arg(d)
//         .arg(d)
//         .arg(project.out_dir().join("uImage"));

//     cmd.exec(true).unwrap();
// }

struct UbootShell {
    boot_cmd: String,
    need_check_test: bool,
    kernel_size: usize,
}

impl UbootShell {
    pub fn run(self: Arc<Self>, port_path: &str, baud_rate: u32) {
        let mut port_rx = serialport::new(port_path, baud_rate)
            .timeout(Duration::from_millis(500))
            .open()
            .unwrap();

        println!(
            "串口：{}, {}",
            port_rx.name().unwrap_or_default(),
            baud_rate
        );

        let mut port_tx = port_rx.try_clone().unwrap();

        serial_wait_for_shell(&mut port_rx);

        println!();
        println!("{}", "Uboot shell ok".green());

        port_tx.write_all("echo $loadaddr\r\n".as_bytes()).unwrap();
        // skip display echo
        let _ = serial_read_until(&mut port_rx, b"\n");
        let bytes = serial_read_until(&mut port_rx, b"\n");
        let loadaddr_raw = String::from_utf8(bytes).unwrap();

        println!("loadaddr: {}", loadaddr_raw);

        let loadaddr = parse_value(&loadaddr_raw);

        let mut fdt_addr = loadaddr + self.kernel_size as u32 + 0x100000;
        fdt_addr = (fdt_addr + 0xFFF) & !0xFFF;

        println!("{}", format!("set fdt_addr to {:#x}\r\n", fdt_addr).green());

        port_tx
            .write_all(format!("setenv fdt_addr {:#x}\r\n", fdt_addr).as_bytes())
            .unwrap();

        port_tx.write_all(self.boot_cmd.as_bytes()).unwrap();
        port_tx.write_all(b"\r\n").unwrap();
        println!("{}", format!("Exec: {}", self.boot_cmd).green());

        if !self.need_check_test {
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
        for byte in port_rx.bytes() {
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
                        if self.need_check_test {
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
    }
}

fn parse_value(line: &str) -> u32 {
    let mut line = line.trim();
    let mut radix = 10;
    if line.starts_with("0x") {
        line = &line[2..];
        radix = 16;
    }
    u32::from_str_radix(line, radix).unwrap()
}

fn serial_read_until(port: &mut Box<dyn SerialPort>, until: &[u8]) -> Vec<u8> {
    let mut buf = [0u8; 1];
    let mut vec = Vec::new();
    loop {
        match port.read(&mut buf) {
            Ok(0) => break,
            Ok(_) => {
                vec.push(buf[0]);
                if vec.ends_with(until) {
                    break;
                }
            }
            Err(_) => {
                panic!("Error reading serial port")
            }
        }
    }
    vec
}

fn serial_wait_for_shell(port: &mut Box<dyn SerialPort>) {
    let mut buf = [0u8; 1];
    let mut history: Vec<u8> = Vec::new();
    let mut is_itr = false;
    loop {
        match port.read(&mut buf) {
            Ok(n) => {
                if n == 1 {
                    let ch = buf[0];
                    if ch == b'\n' && history.last() != Some(&b'\r') {
                        stdout().write_all(b"\r").unwrap();
                        history.push(b'\r');
                    }
                    history.push(ch);

                    io::stdout().write_all(&buf).unwrap();

                    if let Ok(s) = String::from_utf8(history.clone()) {
                        if s.contains("Hit any key to stop autoboot") && !is_itr {
                            port.write_all(b"a").unwrap();
                            is_itr = true;
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                let check_start = history.len().saturating_sub(5);
                let to_check = &history[check_start..];

                if to_check.contains(&b'#') || to_check.contains(&b'>') || is_itr {
                    return;
                }
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shell_run() {
        let sh = Arc::new(UbootShell {
            boot_cmd: "".to_string(),
            need_check_test: false,
            kernel_size: 0,
        });

        sh.run("COM3", 115200);
    }
}
