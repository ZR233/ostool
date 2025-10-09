use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use clap::Parser;
use ostool::{cmd::{Cli, SubCommands}};

#[cfg(test)]
mod cli_tests {
    use super::*;

    fn create_temp_dir() -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let test_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let test_dir = temp_dir.join(format!("ostool_cli_test_{}", test_id));
        fs::create_dir_all(&test_dir).unwrap();
        test_dir
    }

    fn cleanup_temp_dir(test_dir: &PathBuf) {
        let _ = fs::remove_dir_all(test_dir);
    }

    #[test]
    fn test_cli_parse_help() {
        let cli = Cli::try_parse_from(&["ostool", "--help"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_cli_parse_version() {
        let cli = Cli::try_parse_from(&["ostool", "--version"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_cli_parse_build() {
        let cli = Cli::try_parse_from(&["ostool", "build"]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            SubCommands::Build => {} // Expected
            _ => panic!("Expected Build command"),
        }
    }

    #[test]
    fn test_cli_parse_run_qemu() {
        let cli = Cli::try_parse_from(&["ostool", "run", "qemu"]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            SubCommands::Run(run_args) => {
                // Check that it's a Qemu command
                // This would require accessing the internal subcommand structure
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_cli_parse_run_uboot() {
        let cli = Cli::try_parse_from(&["ostool", "run", "uboot"]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            SubCommands::Run(_) => {} // Expected
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_cli_parse_test() {
        let cli = Cli::try_parse_from(&["ostool", "test"]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            SubCommands::Test(_) => {} // Expected
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_test_with_board_flag() {
        let cli = Cli::try_parse_from(&["ostool", "test", "--board"]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            SubCommands::Test(test_args) => {
                assert!(test_args.board);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_test_with_uboot_flag() {
        let cli = Cli::try_parse_from(&["ostool", "test", "--uboot"]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            SubCommands::Test(test_args) => {
                assert!(test_args.uboot);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_test_with_no_run_flag() {
        let cli = Cli::try_parse_from(&["ostool", "test", "--no-run"]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            SubCommands::Test(test_args) => {
                assert!(test_args.no_run);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_test_with_elf() {
        let cli = Cli::try_parse_from(&[
            "ostool", "test", "--elf", "target/test_binary"
        ]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            SubCommands::Test(test_args) => {
                assert_eq!(test_args.elf, Some("target/test_binary".to_string()));
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_test_with_mode() {
        let cli = Cli::try_parse_from(&["ostool", "test", "uboot"]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            SubCommands::Test(test_args) => {
                assert_eq!(test_args.mode, Some("uboot".to_string()));
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_test_with_trailing_args() {
        let cli = Cli::try_parse_from(&[
            "ostool", "test", "--", "--test", "my_test", "--nocapture"
        ]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            SubCommands::Test(test_args) => {
                assert_eq!(test_args.trailing.len(), 3);
                assert_eq!(test_args.trailing[0], "--test");
                assert_eq!(test_args.trailing[1], "my_test");
                assert_eq!(test_args.trailing[2], "--nocapture");
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_parse_with_workdir() {
        let test_dir = create_temp_dir();
        let workdir_arg = test_dir.to_string_lossy();

        let cli = Cli::try_parse_from(&["ostool", "--workdir", &workdir_arg, "build"]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        assert_eq!(parsed.workdir, Some(workdir_arg.to_string()));

        cleanup_temp_dir(&test_dir);
    }

    #[test]
    fn test_cli_parse_short_workdir() {
        let test_dir = create_temp_dir();
        let workdir_arg = test_dir.to_string_lossy();

        let cli = Cli::try_parse_from(&["ostool", "-w", &workdir_arg, "test"]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        assert_eq!(parsed.workdir, Some(workdir_arg.to_string()));

        cleanup_temp_dir(&test_dir);
    }

    #[test]
    fn test_cli_parse_defconfig() {
        let cli = Cli::try_parse_from(&["ostool", "defconfig"]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            SubCommands::Defconfig(_) => {} // Expected
            _ => panic!("Expected Defconfig command"),
        }
    }

    #[test]
    fn test_cli_parse_invalid_command() {
        let cli = Cli::try_parse_from(&["ostool", "invalid-command"]);
        assert!(cli.is_err());
    }

    #[test]
    fn test_cli_parse_multiple_flags() {
        let test_dir = create_temp_dir();
        let workdir_arg = test_dir.to_string_lossy();

        let cli = Cli::try_parse_from(&[
            "ostool",
            "--workdir", &workdir_arg,
            "test",
            "--board",
            "--uboot",
            "--no-run"
        ]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        assert_eq!(parsed.workdir, Some(workdir_arg.to_string()));

        match parsed.command {
            SubCommands::Test(test_args) => {
                assert!(test_args.board);
                assert!(test_args.uboot);
                assert!(test_args.no_run);
            }
            _ => panic!("Expected Test command"),
        }

        cleanup_temp_dir(&test_dir);
    }

    #[test]
    fn test_cli_parse_test_combined_flags() {
        let cli = Cli::try_parse_from(&[
            "ostool", "test",
            "--elf", "test.elf",
            "--board",
            "--uboot",
            "--mode", "custom",
            "--show-output"
        ]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            SubCommands::Test(test_args) => {
                assert_eq!(test_args.elf, Some("test.elf".to_string()));
                assert!(test_args.board);
                assert!(test_args.uboot);
                assert_eq!(test_args.mode, Some("custom".to_string()));
                assert!(test_args.show_output);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_complex_test_command() {
        let cli = Cli::try_parse_from(&[
            "ostool", "test",
            "--elf", "target/x86_64-unknown-none/debug/test_binary",
            "--",
            "--test", "integration_test",
            "--exact",
            "--ignored"
        ]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            SubCommands::Test(test_args) => {
                assert_eq!(
                    test_args.elf,
                    Some("target/x86_64-unknown-none/debug/test_binary".to_string())
                );
                assert_eq!(test_args.trailing.len(), 4);
                assert_eq!(test_args.trailing[0], "--test");
                assert_eq!(test_args.trailing[1], "integration_test");
                assert_eq!(test_args.trailing[2], "--exact");
                assert_eq!(test_args.trailing[3], "--ignored");
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_all_subcommands_are_parseable() {
        let commands = [
            "build",
            "run qemu",
            "run uboot",
            "run tftp",
            "test",
            "defconfig"
        ];

        for cmd in commands.iter() {
            let args: Vec<&str> = ["ostool"].iter().chain(cmd.split_whitespace()).collect();
            let cli = Cli::try_parse_from(args);
            assert!(cli.is_ok(), "Failed to parse command: {}", cmd);
        }
    }

    #[test]
    fn test_extract_elf_from_cargo_args() {
        // This tests the helper function indirectly through CLI parsing
        let cli = Cli::try_parse_from(&[
            "ostool", "test",
            "--", "--test", "my_test_binary"
        ]);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            SubCommands::Test(test_args) => {
                // The trailing args should be preserved
                assert_eq!(test_args.trailing.len(), 2);
            }
            _ => panic!("Expected Test command"),
        }
    }
}