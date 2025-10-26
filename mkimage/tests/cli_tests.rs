//! CLI tests for mkimage

use assert_cmd::Command;
use tempfile::NamedTempFile;
use std::io::Write;
use std::fs;

/// Test basic CLI functionality
#[test]
fn test_cli_basic() {
    let mut cmd = Command::cargo_bin("mkimage").unwrap();
    cmd.assert().success();
}

/// Test CLI version
#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("mkimage").unwrap();
    cmd.arg("--version").assert().success();
}

/// Test creating a simple image
#[test]
fn test_cli_create_simple() {
    // Create test data file
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "Test kernel data").unwrap();
    input_file.flush().unwrap();

    let output_file = NamedTempFile::new().unwrap();

    let mut cmd = Command::cargo_bin("mkimage").unwrap();
    cmd.args(&[
        "create",
        "-T", "kernel",
        "-A", "arm64",
        "-n", "Test Kernel",
        "-d", input_file.path().to_str().unwrap(),
        "-o", output_file.path().to_str().unwrap(),
    ])
    .assert()
    .success();

    // Verify output file exists and has content
    assert!(output_file.path().exists());
    let output_data = fs::read(output_file.path()).unwrap();
    assert!(output_data.len() > 64); // Should have header + data
}

/// Test listing image information
#[test]
fn test_cli_list() {
    // First create an image
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "Test data for listing").unwrap();
    input_file.flush().unwrap();

    let image_file = NamedTempFile::new().unwrap();

    let mut create_cmd = Command::cargo_bin("mkimage").unwrap();
    create_cmd
        .args(&[
            "create",
            "-T", "firmware",
            "-A", "arm",
            "-n", "Test Firmware",
            "-d", input_file.path().to_str().unwrap(),
            "-o", image_file.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Now list the image
    let mut list_cmd = Command::cargo_bin("mkimage").unwrap();
    list_cmd
        .args(&["list", image_file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("Test Firmware"))
        .stdout(predicates::str::contains("firmware"))
        .stdout(predicates::str::contains("arm"));
}

/// Test verifying an image
#[test]
fn test_cli_verify() {
    // Create an image first
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "Test data for verification").unwrap();
    input_file.flush().unwrap();

    let image_file = NamedTempFile::new().unwrap();

    let mut create_cmd = Command::cargo_bin("mkimage").unwrap();
    create_cmd
        .args(&[
            "create",
            "-T", "ramdisk",
            "-A", "x86_64",
            "-n", "Test Ramdisk",
            "-d", input_file.path().to_str().unwrap(),
            "-o", image_file.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Verify the image
    let mut verify_cmd = Command::cargo_bin("mkimage").unwrap();
    verify_cmd
        .args(&["verify", image_file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("verification successful"));
}

/// Test creating image with total CRC
#[test]
fn test_cli_create_with_total_crc() {
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "Test data with total CRC").unwrap();
    input_file.flush().unwrap();

    let output_file = NamedTempFile::new().unwrap();

    let mut cmd = Command::cargo_bin("mkimage").unwrap();
    cmd.args(&[
        "create",
        "-T", "kernel",
        "-A", "arm64",
        "-n", "Test with CRC",
        "-d", input_file.path().to_str().unwrap(),
        "-o", output_file.path().to_str().unwrap(),
        "--add-total-crc",
    ])
    .assert()
    .success();

    // Verify the image with total CRC
    let mut verify_cmd = Command::cargo_bin("mkimage").unwrap();
    verify_cmd
        .args(&[
            "verify",
            output_file.path().to_str().unwrap(),
            "--total-crc",
        ])
        .assert()
        .success();
}

/// Test verbose output
#[test]
fn test_cli_verbose() {
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "Test verbose output").unwrap();
    input_file.flush().unwrap();

    let output_file = NamedTempFile::new().unwrap();

    let mut cmd = Command::cargo_bin("mkimage").unwrap();
    cmd.args(&[
        "create",
        "-T", "kernel",
        "-A", "arm64",
        "-n", "Verbose Test",
        "-d", input_file.path().to_str().unwrap(),
        "-o", output_file.path().to_str().unwrap(),
        "-v",  // verbose
    ])
    .assert()
    .success()
    .stderr(predicates::str::contains("Creating U-Boot image"))
    .stderr(predicates::str::contains("Loading data from"))
    .stderr(predicates::str::contains("Writing image to"));
}

/// Test print info option
#[test]
fn test_cli_print_info() {
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "Test print info").unwrap();
    input_file.flush().unwrap();

    let output_file = NamedTempFile::new().unwrap();

    let mut cmd = Command::cargo_bin("mkimage").unwrap();
    cmd.args(&[
        "create",
        "-T", "kernel",
        "-A", "arm64",
        "-n", "Info Test",
        "-d", input_file.path().to_str().unwrap(),
        "-o", output_file.path().to_str().unwrap(),
        "--print-info",
    ])
    .assert()
    .success()
    .stdout(predicates::str::contains("Image: Info Test"))
    .stdout(predicates::str::contains("Type: kernel"))
    .stdout(predicates::str::contains("Arch: arm64"));
}

/// Test JSON output
#[test]
fn test_cli_json_output() {
    // Create an image first
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "Test JSON output").unwrap();
    input_file.flush().unwrap();

    let image_file = NamedTempFile::new().unwrap();

    let mut create_cmd = Command::cargo_bin("mkimage").unwrap();
    create_cmd
        .args(&[
            "create",
            "-T", "script",
            "-A", "x86",
            "-n", "JSON Test",
            "-d", input_file.path().to_str().unwrap(),
            "-o", image_file.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // List with JSON output
    let mut list_cmd = Command::cargo_bin("mkimage").unwrap();
    list_cmd
        .args(&[
            "list",
            image_file.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"name\": \"JSON Test\""))
        .stdout(predicates::str::contains("\"type\": \"script\""));
}

/// Test quiet mode
#[test]
fn test_cli_quiet_mode() {
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "Test quiet mode").unwrap();
    input_file.flush().unwrap();

    let output_file = NamedTempFile::new().unwrap();

    let mut cmd = Command::cargo_bin("mkimage").unwrap();
    cmd.args(&[
        "create",
        "-T", "kernel",
        "-A", "arm64",
        "-n", "Quiet Test",
        "-d", input_file.path().to_str().unwrap(),
        "-o", output_file.path().to_str().unwrap(),
        "-q",  // quiet
    ])
    .assert()
    .success()
    .stderr(predicates::str::is_empty()); // No stderr output in quiet mode
}

/// Test default create command (no subcommand)
#[test]
fn test_cli_default_create() {
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "Test default create").unwrap();
    input_file.flush().unwrap();

    let output_file = NamedTempFile::new().unwrap();

    // Test with just create options, no subcommand
    let mut cmd = Command::cargo_bin("mkimage").unwrap();
    cmd.args(&[
        "-T", "kernel",
        "-A", "arm64",
        "-n", "Default Test",
        "-d", input_file.path().to_str().unwrap(),
        "-o", output_file.path().to_str().unwrap(),
    ])
    .assert()
    .success();

    assert!(output_file.path().exists());
}

/// Test hex address parsing
#[test]
fn test_cli_hex_addresses() {
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "Test hex addresses").unwrap();
    input_file.flush().unwrap();

    let output_file = NamedTempFile::new().unwrap();

    let mut cmd = Command::cargo_bin("mkimage").unwrap();
    cmd.args(&[
        "create",
        "-T", "kernel",
        "-A", "arm64",
        "-n", "Hex Address Test",
        "-d", input_file.path().to_str().unwrap(),
        "-o", output_file.path().to_str().unwrap(),
        "-a", "0x80000",  // hex load address
        "-e", "0x80000",  // hex entry point
    ])
    .assert()
    .success();

    // Verify the image
    let mut verify_cmd = Command::cargo_bin("mkimage").unwrap();
    verify_cmd
        .args(&["list", output_file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("0x0080000")); // Load address
}

/// Test error handling - missing data file
#[test]
fn test_cli_missing_data_file() {
    let output_file = NamedTempFile::new().unwrap();

    let mut cmd = Command::cargo_bin("mkimage").unwrap();
    cmd.args(&[
        "create",
        "-T", "kernel",
        "-A", "arm64",
        "-n", "Missing File Test",
        "-d", "/nonexistent/file.bin",
        "-o", output_file.path().to_str().unwrap(),
    ])
    .assert()
    .failure(); // Should fail with file not found error
}

/// Test error handling - invalid image file for verification
#[test]
fn test_cli_verify_invalid_image() {
    let mut cmd = Command::cargo_bin("mkimage").unwrap();
    cmd.args(&["verify", "/nonexistent/image.bin"])
        .assert()
        .failure();
}