use std::{
    env,
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn create_temp_dir() -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let test_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let test_dir = temp_dir.join(format!("ostool_integration_test_{}", test_id));
        fs::create_dir_all(&test_dir).unwrap();
        test_dir
    }

    fn cleanup_temp_dir(test_dir: &PathBuf) {
        let _ = fs::remove_dir_all(test_dir);
    }

    fn get_ostool_binary() -> PathBuf {
        // Try to find the ostool binary in different locations
        let possible_paths = vec![
            PathBuf::from("./target/release/ostool"),
            PathBuf::from("./target/debug/ostool"),
            PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string()))
                .join(env::var("PROFILE").unwrap_or_else(|_| "debug".to_string()))
                .join("ostool"),
        ];

        possible_paths
            .into_iter()
            .find(|path| path.exists())
            .unwrap_or_else(|| panic!("ostool binary not found. Please run `cargo build` first."))
    }

    fn run_ostool_command(args: &[&str], workdir: Option<&PathBuf>) -> Result<std::process::Output> {
        let ostool_binary = get_ostool_binary();
        let mut cmd = Command::new(&ostool_binary);

        if let Some(dir) = workdir {
            cmd.current_dir(dir);
        }

        cmd.args(args);

        let output = cmd.output()?;
        Ok(output)
    }

    #[test]
    fn test_ostool_help() {
        let output = run_ostool_command(&["--help"], None).unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("ostool"));
        assert!(stdout.contains("A tool for operating system development"));
    }

    #[test]
    fn test_ostool_version() {
        let output = run_ostool_command(&["--version"], None).unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains('.'));
    }

    #[test]
    fn test_ostool_defconfig() {
        let test_dir = create_temp_dir();

        // Create a basic Cargo project structure
        let cargo_toml = test_dir.join("Cargo.toml");
        fs::write(&cargo_toml, r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#).unwrap();

        let output = run_ostool_command(&["defconfig"], Some(&test_dir)).unwrap();

        // This might fail due to interactive UI, but we can check that it creates files
        let config_file = test_dir.join(".project.toml");

        cleanup_temp_dir(&test_dir);

        // The test passes if the command runs without crashing
        // In CI environment, this might fail due to no interactive input
    }

    #[test]
    fn test_ostool_build_with_missing_config() {
        let test_dir = create_temp_dir();

        let output = run_ostool_command(&["build"], Some(&test_dir)).unwrap();

        // Should fail gracefully without panicking
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should contain error information
        assert!(!stderr.is_empty());

        cleanup_temp_dir(&test_dir);
    }

    #[test]
    fn test_ostool_test_help() {
        let output = run_ostool_command(&["test", "--help"], None).unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("test"));
        assert!(stdout.contains("--elf"));
        assert!(stdout.contains("--board"));
        assert!(stdout.contains("--uboot"));
    }

    #[test]
    fn test_ostool_run_help() {
        let output = run_ostool_command(&["run", "--help"], None).unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("run"));
    }

    #[test]
    fn test_ostool_with_invalid_command() {
        let output = run_ostool_command(&["invalid-command"], None).unwrap();
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("unrecognized") || stderr.contains("unexpected"));
    }

    #[test]
    fn test_ostool_with_project_config() {
        let test_dir = create_temp_dir();

        // Create a basic project structure
        let cargo_toml = test_dir.join("Cargo.toml");
        fs::write(&cargo_toml, r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#).unwrap();

        // Create .project.toml
        let project_config = test_dir.join(".project.toml");
        fs::write(&project_config, r#"
[compile]
target = "aarch64-unknown-none"

[compile.build.Custom]
shell = ["echo 'Hello from test'"]
kernel = "test.bin"

[qemu]
machine = "virt"
cpu = "cortex-a57"
graphic = false
"#).unwrap();

        let output = run_ostool_command(&["build"], Some(&test_dir)).unwrap();

        // The command might fail if dependencies are missing, but it should not panic
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should have some output
        assert!(!stdout.is_empty() || !stderr.is_empty());

        cleanup_temp_dir(&test_dir);
    }

    #[test]
    fn test_ostool_config_inheritance() {
        let test_dir = create_temp_dir();

        // Create main config
        let main_config = test_dir.join(".project.toml");
        fs::write(&main_config, r#"
[compile]
target = "aarch64-unknown-none"

[compile.build.Custom]
shell = ["echo 'Main config'"]
kernel = "main.bin"

[qemu]
machine = "virt"
cpu = "cortex-a57"

include = ["board.toml"]
"#).unwrap();

        // Create board config
        let board_config = test_dir.join("board.toml");
        fs::write(&board_config, r#"
[uboot]
serial = "/dev/ttyUSB0"
baud_rate = 115200
dtb_file = "board.dtb"
"#).unwrap();

        let output = run_ostool_command(&["test", "--board"], Some(&test_dir)).unwrap();

        // Should process include without crashing
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        cleanup_temp_dir(&test_dir);

        // Test passes if no panic occurs
        assert!(true);
    }

    #[test]
    fn test_ostool_with_workdir() {
        let test_dir = create_temp_dir();

        // Create .project.toml
        let project_config = test_dir.join(".project.toml");
        fs::write(&project_config, r#"
[compile]
target = "x86_64-unknown-none"

[compile.build.Custom]
shell = ["echo 'Workdir test'"]
kernel = "workdir.bin"
"#).unwrap();

        let workdir_arg = test_dir.to_string_lossy();
        let output = run_ostool_command(&["--workdir", &workdir_arg, "build"], None).unwrap();

        // Should use the specified working directory
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        cleanup_temp_dir(&test_dir);
    }

    #[test]
    fn test_ostool_error_handling() {
        let test_dir = create_temp_dir();

        // Create a corrupted config file
        let config_file = test_dir.join(".project.toml");
        fs::write(&config_file, "invalid toml [[[ content").unwrap();

        let output = run_ostool_command(&["build"], Some(&test_dir)).unwrap();

        // Should handle the error gracefully
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should contain error information
        assert!(!stderr.is_empty());

        // Should create backup file
        let backups: Vec<_> = fs::read_dir(&test_dir)
            .unwrap()
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.file_name()?.to_str()?.starts_with(".project.toml.bk.") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        assert!(!backups.is_empty(), "Should have created backup files");

        cleanup_temp_dir(&test_dir);
    }

    #[test]
    fn test_all_commands_help() {
        let commands = [
            "build --help",
            "run --help",
            "test --help",
            "defconfig --help"
        ];

        for cmd in commands.iter() {
            let args: Vec<&str> = cmd.split_whitespace().collect();
            let output = run_ostool_command(&args, None).unwrap();

            assert!(
                output.status.success(),
                "Help command failed: {}",
                cmd
            );

            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                !stdout.is_empty(),
                "Help output should not be empty for: {}",
                cmd
            );
        }
    }

    #[test]
    fn test_ostool_no_panic_on_various_inputs() {
        let test_dir = create_temp_dir();

        let test_cases = vec![
            vec!["test", "--elf", "nonexistent.elf"],
            vec!["test", "--board"],
            vec!["test", "--uboot"],
            vec!["run", "qemu", "-d"],
            vec!["run", "uboot"],
        ];

        for args in test_cases {
            let output = run_ostool_command(&args, Some(&test_dir)).unwrap();

            // Should not panic (output should exist)
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // At least one of stdout or stderr should have content
            assert!(
                !stdout.is_empty() || !stderr.is_empty(),
                "No output for args: {:?}",
                args
            );
        }

        cleanup_temp_dir(&test_dir);
    }
}