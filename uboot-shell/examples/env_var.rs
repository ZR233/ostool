use std::{
    net::TcpStream,
    process::{Child, Command},
    time::Duration,
};

use log::info;
use uboot_shell::UbootShell;

fn main() {
    env_logger::init();

    let (mut out, mut uboot) = new_uboot();

    uboot.set_env("fdt_addr", "0x40000000").unwrap();
    info!("set fdt_addr ok");
    assert_eq!(uboot.env_int("fdt_addr").unwrap(), 0x40000000);

    info!("finish");
    let _ = out.kill();
    let _ = out.wait();
}

fn new_uboot() -> (Child, UbootShell) {
    // qemu-system-aarch64 -machine virt -cpu cortex-a57 -nographic -bios assets/u-boot.bin -serial tcp::12345,server
    let out = Command::new("qemu-system-aarch64")
        .args([
            "-machine",
            "virt",
            "-cpu",
            "cortex-a57",
            "-nographic",
            "-serial",
            "tcp::12345,server",
            "-bios",
            "assets/u-boot.bin",
        ])
        .spawn()
        .unwrap();

    let tx;

    loop {
        std::thread::sleep(Duration::from_millis(100));
        match TcpStream::connect("127.0.0.1:12345") {
            Ok(s) => {
                tx = s;
                break;
            }
            Err(e) => {
                println!("wait for qemu serial port ready: {e}");
            }
        }
    }

    let rx = tx.try_clone().unwrap();
    rx.set_read_timeout(Some(Duration::from_millis(300)))
        .unwrap();
    println!("connect ok");
    (out, UbootShell::new(tx, rx).unwrap())
}
