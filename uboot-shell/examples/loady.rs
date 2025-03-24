use std::process::{Child, Command, Stdio};

use uboot_shell::UbootShell;

fn main() {
    let (mut out, mut uboot) = new_uboot();

    // uboot.loady(0x40200000, "Cargo.toml");

    println!("finish");

    uboot.wait_for_reply("12345").unwrap();

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
            "-bios",
            "assets/u-boot.bin",
        ])
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = out.stdin.take().unwrap();
    let stdout = out.stdout.take().unwrap();

    (out, UbootShell::new(stdin, stdout).unwrap())
}
