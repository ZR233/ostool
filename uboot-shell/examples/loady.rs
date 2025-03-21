use std::process::{Child, Command, Stdio};

use uboot_shell::UbootCli;

fn main() {
    let (mut out, mut uboot) = new_uboot();

    uboot.wait_for_shell();

    uboot.loady(0x40200000, "Cargo.toml");

    println!("finish");

    uboot.wait_for_reply("12345");

    out.wait();
}

fn new_uboot() -> (Child, UbootCli) {
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

    (out, UbootCli::new(stdin, stdout))
}
