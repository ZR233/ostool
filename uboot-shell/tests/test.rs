use std::{
    io::Read,
    net::TcpStream,
    process::{Child, Command, Stdio},
    sync::atomic::AtomicU32,
    time::Duration,
};

use ntest::timeout;
use uboot_shell::UbootShell;

static PORT: AtomicU32 = AtomicU32::new(10000);

fn new_uboot() -> (Child, UbootShell) {
    let port = PORT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    // qemu-system-aarch64 -machine virt -cpu cortex-a57 -nographic -bios assets/u-boot.bin
    let mut out = Command::new("qemu-system-aarch64")
        .arg("-serial")
        .arg(format!("tcp::{port},server,nowait"))
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

    let tx = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();

    tx.set_read_timeout(Some(Duration::from_millis(300)))
        .unwrap();

    let rx = tx.try_clone().unwrap();

    (out, UbootShell::new(tx, rx).unwrap())
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
