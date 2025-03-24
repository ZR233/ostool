use std::process::{Command, Stdio};

#[test]
fn test_cmd() {
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
        .stdout(Stdio::inherit())
        .spawn()
        .unwrap();

    let out = out.wait().unwrap();
}
