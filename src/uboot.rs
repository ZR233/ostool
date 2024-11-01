use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use serde::{Deserialize, Serialize};

use crate::{shell, ui};

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
