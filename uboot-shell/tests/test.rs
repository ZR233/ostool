use std::{
    net::TcpStream,
    process::{Child, Command},
    sync::atomic::AtomicU32,
    time::Duration,
};

use log::{debug, info};
use ntest::timeout;
use uboot_shell::UbootShell;

static PORT: AtomicU32 = AtomicU32::new(10000);

fn new_uboot() -> (Child, UbootShell) {
    let port = PORT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    // qemu-system-aarch64 -machine virt -cpu cortex-a57 -nographic -bios assets/u-boot.bin
    let out = Command::new("qemu-system-aarch64")
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
        .spawn()
        .unwrap();

    let tx;

    loop {
        std::thread::sleep(Duration::from_millis(100));
        match TcpStream::connect(format!("127.0.0.1:{port}")) {
            Ok(s) => {
                tx = s;
                break;
            }
            Err(e) => {
                debug!("wait for qemu serial port ready: {e}");
            }
        }
    }

    let rx = tx.try_clone().unwrap();
    rx.set_read_timeout(Some(Duration::from_millis(300)))
        .unwrap();
    info!("connect ok");
    (out, UbootShell::new(tx, rx).unwrap())
}

#[test]
#[timeout(5000)]
fn test_shell() {
    env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .init();

    let (mut out, _uboot) = new_uboot();
    info!("test_shell ok");
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
    env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .init();

    with_uboot(|uboot| {
        uboot.set_env("fdt_addr", "0x40000000").unwrap();
        info!("set fdt_addr ok");
        assert_eq!(uboot.env_int("fdt_addr").unwrap(), 0x40000000);
    });
}
