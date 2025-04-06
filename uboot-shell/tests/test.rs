use std::process::{Child, Command, Stdio};

use ntest::timeout;
use uboot_shell::UbootShell;

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
            "../assets/u-boot.bin",
        ])
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = out.stdin.take().unwrap();
    let stdout = out.stdout.take().unwrap();

    (out, UbootShell::new(stdin, stdout).unwrap())
}

#[test]
#[timeout(5000)]
fn test_shell() {
    let (mut out, _uboot) = new_uboot();
    let _ = out.kill();
    out.wait().unwrap();
}

fn with_uboot(f: impl FnOnce(&mut UbootShell)) {
    let (mut out, mut uboot) = new_uboot();

    f(&mut uboot);

    let _ = out.kill();
    out.wait().unwrap();
}

#[test]
#[timeout(5000)]
fn test_cmd() {
    with_uboot(|uboot| {
        let res = uboot.cmd("help").unwrap();
        println!("{}", res);
    });
}

#[test]
#[timeout(5000)]
fn test_setenv() {
    with_uboot(|uboot| {
        uboot.set_env("ipaddr", "127.0.0.1").unwrap();
    });
}

#[test]
#[timeout(5000)]
fn test_env() {
    with_uboot(|uboot| {
        assert_eq!(uboot.env_int("fdt_addr").unwrap(), 0x40000000);
    });
}
