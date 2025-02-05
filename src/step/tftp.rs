use std::net::{IpAddr, Ipv4Addr};

use super::Step;
use tftpd::{Config, Server};

pub struct Tftp {}

impl Tftp {
    pub fn new_boxed() -> Box<dyn Step> {
        Box::new(Self {})
    }
}

impl Step for Tftp {
    fn run(&mut self, project: &mut crate::project::Project) -> anyhow::Result<()> {
        let file_dir = project.out_dir();
        println!("启动 TFTP 服务器...");
        println!("文件目录：{}", file_dir.display());
        let mut config = Config::default();
        config.directory = file_dir;
        config.send_directory = config.directory.clone();
        config.port = 69;
        config.ip_address = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

        std::thread::spawn(move || {
            let mut server = Server::new(&config)
                .map_err(|e| format!("TFTP server 启动失败：{e:?}。若权限不足，尝试执行 `sudo setcap cap_net_bind_service=+eip $(which ostool)` 并重启终端"))
                .unwrap();
            server.listen();
        });

        Ok(())
    }
}
