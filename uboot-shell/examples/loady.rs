use std::{
    io::Read,
    net::TcpStream,
    process::{Child, Command, Stdio},
    time::Duration,
};

use uboot_shell::UbootShell;

fn main() {
    let (mut out, mut uboot) = new_uboot();

    uboot.loady(0x40200000, "Cargo.toml", |_r, _a| {}).unwrap();

    println!("finish");
    let _ = out.kill();
    let _ = out.wait();
}

fn new_uboot() -> (Child, UbootShell) {
    // qemu-system-aarch64 -machine virt -cpu cortex-a57 -nographic -bios assets/u-boot.bin
    let mut out = Command::new("qemu-system-aarch64")
        .args([
            "-machine",
            "virt",
            "-cpu",
            "cortex-a57",
            "-nographic",
            "-serial",
            "tcp::12345,server,nowait",
            "-bios",
            "assets/u-boot.bin",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = out.stdout.take().unwrap();
    let mut buff = vec![];
    for i in stdout.bytes() {
        buff.push(i.unwrap());
        if String::from_utf8_lossy(&buff).contains("qemu") {
            break;
        }
    }

    let tx = TcpStream::connect("127.0.0.1:12345").unwrap();

    let rx = tx.try_clone().unwrap();
    rx.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

    (out, UbootShell::new(tx, rx).unwrap())
}
