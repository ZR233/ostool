use std::{
    io::{Write, stdout},
    time::Duration,
};

use uboot_shell::UbootCli;

fn main() {
    let mut uboot = new_uboot();

    println!("wait for uboot");
    uboot.wait_for_shell();

    // let addr = uboot.env_int("kernel_addr_r").unwrap();
    let addr = 0x90000000;

    uboot.loady(addr, "target/tmp.bin");

    println!("finish");

    loop {
        let mut buf = [0; 1];
        if uboot.rx.as_mut().unwrap().read_exact(&mut buf).is_ok() {
            stdout().write_all(&buf);
        }
    }
}

fn new_uboot() -> UbootCli {
    let port = "/dev/ttyUSB0";
    let baud = 115200;

    let rx = serialport::new(port, baud)
        .timeout(Duration::from_millis(300))
        .open()
        .map_err(|e| format!("无法打开串口: {:?}", e))
        .unwrap();

    let tx = rx.try_clone().unwrap();

    UbootCli::new(tx, rx)
}
