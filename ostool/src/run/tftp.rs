use std::net::{IpAddr, Ipv4Addr};

use colored::Colorize as _;
use tftpd::{Config, Server};

use crate::ctx::AppContext;

pub fn run_tftp_server(app: &AppContext) -> anyhow::Result<()> {
    // TFTP server implementation goes here
    let mut file_dir = app.manifest_dir.clone();
    if let Some(elf_path) = &app.elf_path {
        file_dir = elf_path
            .parent()
            .ok_or(anyhow!("{} no parent dir", elf_path.display()))?
            .to_path_buf();
    }

    info!(
        "Starting TFTP server serving files from: {}",
        file_dir.display()
    );

    let mut config = Config::default();
    config.directory = file_dir;
    config.send_directory = config.directory.clone();
    config.port = 69;
    config.ip_address = IpAddr::V4(Ipv4Addr::UNSPECIFIED);

    std::thread::spawn(move || {
        let mut server = Server::new(&config)
                .inspect_err(|e| {
                    println!("{}", e);
                    println!("{}","TFTP server 启动失败：{e:?}。若权限不足，尝试执行 `sudo setcap cap_net_bind_service=+eip $(which cargo-osrun)&&sudo setcap cap_net_bind_service=+eip $(which ostool)` 并重启终端".red());
                    std::process::exit(1);
                }).unwrap();
        server.listen();
    });

    Ok(())
}
