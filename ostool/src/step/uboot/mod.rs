use std::{
    collections::VecDeque,
    fs::{self},
    io::{self, stdout, Read, Write},
    path::PathBuf,
    process::exit,
    thread::{self, sleep},
    time::{Duration, Instant},
};

use colored::Colorize;
use crossterm::event::{self, Event, KeyCode};
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use serde::{Deserialize, Serialize};
use serialport::SerialPort;

use crate::{project::Project, ui};

use super::Step;

#[cfg(test)]
mod tests;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct UbootConfig {
    pub serial: String,
    pub baud_rate: i64,
    pub net: String,
    pub dtb_file: String,
    pub dhcp: bool,
    pub board_ip: Option<String>,
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
            dhcp: true,
            ..Default::default()
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

        let mut ip_string = String::new();

        let interfaces = NetworkInterface::show().unwrap();
        for interface in interfaces.iter() {
            if interface.name == config.net {
                let addr_list: Vec<Addr> = interface.addr.to_vec();
                for one in addr_list {
                    if let Addr::V4(v4_if_addr) = one {
                        ip_string = v4_if_addr.ip.to_string();
                    }
                }
            }
        }

        println!("TFTP : {}", ip_string);

        println!("内核：{}", kernel_bin);

        let out_dir = project.out_dir();

        let mut boot_cmd_base = String::new();

        if config.dhcp {
            boot_cmd_base = format!("{boot_cmd_base}dhcp;");
        }

        let mut fdtfile = String::new();

        let boot_cmd = if let Some(dtb) = PathBuf::from(&config.dtb_file).file_name() {
            // mkimage(project, &config.dtb_file);

            let dtb_name = dtb.to_str().unwrap().to_string();

            fdtfile = dtb_name.to_string();

            let ftp_dtb = out_dir.join(&dtb_name);

            let _ = fs::copy(config.dtb_file, ftp_dtb);

            format!("{boot_cmd_base}tftp $loadaddr {ip_string}:$bootfile;tftp $fdt_addr {ip_string}:$fdtfile;fdt addr $fdt_addr;booti $loadaddr - $fdt_addr")
        } else {
            println!("DTB file not provided");
            format!("{boot_cmd_base}tftp $loadaddr {ip_string}:$bootfile;dcache flush;go $loadaddr")
        };
        println!("启动命令：{}", boot_cmd);

        println!("等待 U-Boot 启动...");

        let kernel_size = fs::metadata(project.to_load_kernel.as_ref().unwrap())
            .unwrap()
            .len();

        let mut sh = UbootShell {
            boot_cmd,
            need_check_test: self.is_check_test,
            kernel_size: kernel_size as _,
            bootfile: kernel_bin.to_string(),
            fdtfile,
            server_ip: ip_string,
            board_ip: config.board_ip.clone(),
            ..Default::default()
        };

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
#[derive(Default)]
struct UbootShell {
    boot_cmd: String,
    need_check_test: bool,
    kernel_size: usize,
    bootfile: String,
    fdtfile: String,
    server_ip: String,
    board_ip: Option<String>,
    _rx: Option<Box<dyn SerialPort>>,
    tx: Option<Box<dyn SerialPort>>,
}

impl UbootShell {
    pub fn run(&mut self, port_path: &str, baud_rate: u32) {
        let port_rx = serialport::new(port_path, baud_rate)
            .timeout(Duration::from_millis(300))
            .open()
            .map_err(|e| format!("无法打开串口 {port_path}: {:?}", e))
            .unwrap();

        self._rx = Some(port_rx);

        println!(
            "串口：{}, {}",
            self.rx().name().unwrap_or_default(),
            self.rx().baud_rate().unwrap()
        );

        self.tx = Some(self.rx().try_clone().unwrap());

        self.wait_for_shell();

        println!();
        println!("{}", "Uboot shell ok".green());

        let loadaddr = self.env_int_read("loadaddr").unwrap_or_else(|| {
            let loadaddr = self
                .env_int_read("kernel_addr_r")
                .expect("kernel_addr_r not found");

            println!("$loadaddr not found, set to {:#x}", loadaddr);

            self.set_env("loadaddr", &format!("{loadaddr:#x}"));

            loadaddr
        });

        let mut fdt_addr = loadaddr + self.kernel_size as u64 + 0x100000;
        fdt_addr = (fdt_addr + 0xFFF) & !0xFFF;

        self.set_env("autoload", "no");

        self.set_env("bootfile", &self.bootfile.to_string());

        self.set_env("fdtfile", &self.fdtfile.to_string());

        self.set_env("serverip", &self.server_ip.to_string());

        self.set_env("fdt_addr", &format!("{:#x}", fdt_addr));

        if let Some(board_ip) = self.board_ip.clone() {
            self.set_env("ipaddr", &board_ip);
        }

        self.send_cmd(&self.boot_cmd.to_string());

        if !self.need_check_test {
            let mut port_tx = self.tx.take().unwrap();

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

        let need_check_test = self.need_check_test;

        for byte in self.rx().bytes() {
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
    }

    fn rx(&mut self) -> &mut Box<dyn SerialPort> {
        self._rx.as_mut().unwrap()
    }

    fn send_cmd(&mut self, cmd: &str) {
        self.tx.as_mut().unwrap().write_all(cmd.as_bytes()).unwrap();
        self.tx
            .as_mut()
            .unwrap()
            .write_all("\r\n".as_bytes())
            .unwrap();
        self.tx.as_mut().unwrap().flush().unwrap();

        let line = self.read_line();

        println!("{line}");
        sleep(Duration::from_millis(100));
    }

    fn wait_for_shell(&mut self) {
        let mut buf = [0u8; 1];
        let mut history: Vec<u8> = Vec::new();
        const CTRL_C: u8 = 0x03;

        let mut last = Instant::now();

        loop {
            match self.rx().read(&mut buf) {
                Ok(n) => {
                    if n == 1 {
                        let ch = buf[0];
                        if ch == b'\n' && history.last() != Some(&b'\r') {
                            stdout().write_all(b"\r").unwrap();
                            history.push(b'\r');
                        }
                        history.push(ch);

                        io::stdout().write_all(&buf).unwrap();

                        if history.ends_with(c"<INTERRUPT>".to_bytes()) {
                            return;
                        }

                        if last.elapsed() > Duration::from_millis(100) {
                            let _ = self.tx.as_mut().unwrap().write_all(&[CTRL_C]);
                            last = Instant::now();
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }

    fn env_int_read(&mut self, name: &str) -> Option<u64> {
        // clean buffer
        loop {
            if let Err(_e) = self.rx().read(&mut [0; 100]) {
                break;
            }
        }

        self.tx
            .as_mut()?
            .write_all(format!("echo ${name}\r\n").as_bytes())
            .unwrap();

        let mut raw;

        loop {
            let bytes = serial_read_until(self.rx(), b"\n");
            raw = String::from_utf8(bytes).unwrap();
            if raw.contains("0x") || raw.trim().is_empty() {
                break;
            }
        }
        println!("{}", format!("${name}: {raw}").green());
        parse_value(&raw)
    }

    fn set_env(&mut self, name: &str, value: &str) {
        self.send_cmd(&format!("setenv {name} {value}"));
    }

    fn read_line(&mut self) -> String {
        let mut line_raw = Vec::new();
        let mut byte = [0; 1];

        while let Ok(n) = self.rx().read(&mut byte) {
            if n == 0 {
                break;
            }

            if byte[0] == b'\r' {
                continue;
            }

            if byte[0] == b'\n' {
                break;
            }

            line_raw.push(byte[0]);
        }

        if line_raw.is_empty() {
            return String::new();
        }

        let line = String::from_utf8_lossy(&line_raw);
        line.trim().to_string()
    }
}

fn parse_value(line: &str) -> Option<u64> {
    let mut line = line.trim();
    let mut radix = 10;
    if line.starts_with("0x") {
        line = &line[2..];
        radix = 16;
    }
    u64::from_str_radix(line, radix).ok()
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shell_run() {
        let mut sh = UbootShell {
            boot_cmd: "".to_string(),
            need_check_test: false,
            kernel_size: 0,
            bootfile: "".to_string(),
            fdtfile: "".to_string(),
            _rx: None,
            tx: None,
            server_ip: "".to_string(),
            board_ip: None,
        };

        sh.run("COM3", 115200);
    }
}
