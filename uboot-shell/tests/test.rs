use std::{
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, sleep},
    time::{Duration, Instant},
};

use ntest::timeout;
use uboot_shell::UbootCli;

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
            "../assets/u-boot.bin",
        ])
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = out.stdin.take().unwrap();
    let stdout = out.stdout.take().unwrap();

    (out, UbootCli::new(stdin, stdout))
}

#[test]
#[timeout(5000)]
fn test_shell() {
    let (mut out, mut uboot) = new_uboot();

    uboot.wait_for_shell();

    let _ = out.kill();
    out.wait().unwrap();
}

#[test]
#[timeout(5000)]
fn test_cmd() {
    let (mut out, mut uboot) = new_uboot();

    uboot.wait_for_shell();

    let res = uboot.cmd("help");

    println!("{}", res);

    let _ = out.kill();
    out.wait().unwrap();
}

#[test]
#[timeout(5000)]
fn test_setenv() {
    let (mut out, mut uboot) = new_uboot();

    uboot.wait_for_shell();

    uboot.set_env("ipaddr", "127.0.0.1");

    let _ = out.kill();
    out.wait().unwrap();
}

#[test]
#[timeout(5000)]
fn test_env() {
    let (mut out, mut uboot) = new_uboot();

    uboot.wait_for_shell();

    assert_eq!(uboot.env_int("fdt_addr"), Some(0x40000000));

    let _ = out.kill();
    out.wait().unwrap();
}
