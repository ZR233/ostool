use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use ostool::project::{Project, Arch};
use ostool::config::ProjectConfig;

#[cfg(test)]
mod project_tests {
    use super::*;

    fn create_temp_dir() -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let test_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let test_dir = temp_dir.join(format!("ostool_project_test_{}", test_id));
        fs::create_dir_all(&test_dir).unwrap();
        test_dir
    }

    fn cleanup_temp_dir(test_dir: &PathBuf) {
        let _ = fs::remove_dir_all(test_dir);
    }

    fn create_test_project_config(test_dir: &PathBuf) -> ProjectConfig {
        let config_path = test_dir.join(".project.toml");
        let config_content = r#"
[compile]
target = "aarch64-unknown-none"

[compile.build.Custom]
shell = ["echo test"]
kernel = "test.bin"

[qemu]
machine = "virt"
cpu = "cortex-a57"
graphic = false
"#;
        fs::write(&config_path, config_content).unwrap();

        toml::from_str(config_content).unwrap()
    }

    #[test]
    fn test_project_new() {
        let test_dir = create_temp_dir();
        let project = Project::new(test_dir.clone());

        assert_eq!(project.workdir(), &test_dir);
        assert!(project.config.is_none());
        assert!(project.arch.is_none());
        assert!(project.out_dir.is_none());
        assert!(project.kernel.is_none());
        assert!(project.is_print_cmd); // default value

        cleanup_temp_dir(&test_dir);
    }

    #[test]
    fn test_project_shell_command() -> Result<()> {
        let test_dir = create_temp_dir();
        let project = Project::new(test_dir.clone());

        let cmd = project.shell("echo");
        assert_eq!(cmd.get_program(), "echo");

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_project_config_with_file() -> Result<()> {
        let test_dir = create_temp_dir();
        create_test_project_config(&test_dir);

        let mut project = Project::new(test_dir.clone());
        project.config_with_file()?;

        assert!(project.config.is_some());
        assert!(project.arch.is_some());
        assert_eq!(project.arch.unwrap(), Arch::Aarch64);

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_project_test_config() -> Result<()> {
        let test_dir = create_temp_dir();
        create_test_project_config(&test_dir);

        let mut project = Project::new(test_dir.clone());
        project.test_config(None, false)?;

        assert!(project.config.is_some());
        assert!(project.arch.is_some());
        assert_eq!(project.arch.unwrap(), Arch::Aarch64);

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_project_test_config_with_elf() -> Result<()> {
        let test_dir = create_temp_dir();
        create_test_project_config(&test_dir);

        let mut project = Project::new(test_dir.clone());
        project.test_config(Some("test_binary.elf".to_string()), false)?;

        assert!(project.config.is_some());
        assert!(project.arch.is_some());

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_project_test_config_board_mode() -> Result<()> {
        let test_dir = create_temp_dir();

        // Create main config
        let main_config = test_dir.join(".project.toml");
        fs::write(&main_config, r#"
[compile]
target = "aarch64-unknown-none"

[compile.build.Custom]
shell = ["echo test"]
kernel = "test.bin"

[qemu]
machine = "virt"
cpu = "cortex-a57"
"#)?;

        // Create board config
        let board_config = test_dir.join(".board.toml");
        fs::write(&board_config, r#"
[uboot]
serial = "/dev/ttyUSB0"
baud_rate = 115200
dtb_file = "board.dtb"
"#)?;

        let mut project = Project::new(test_dir.clone());
        project.test_config(None, true)?;

        assert!(project.config.is_some());
        assert!(project.config_ref().uboot.is_some());

        let uboot_config = project.config_ref().uboot.as_ref().unwrap();
        assert_eq!(uboot_config.serial, "/dev/ttyUSB0");
        assert_eq!(uboot_config.baud_rate, 115200);

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_project_config_ref() -> Result<()> {
        let test_dir = create_temp_dir();
        create_test_project_config(&test_dir);

        let mut project = Project::new(test_dir.clone());
        project.config_with_file()?;

        let config_ref = project.config_ref();
        assert_eq!(config_ref.compile.target, "aarch64-unknown-none");
        assert_eq!(config_ref.qemu.machine, Some("virt".to_string()));

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_arch_from_target() -> Result<()> {
        assert_eq!(Arch::from_target("aarch64-unknown-none")?, Arch::Aarch64);
        assert_eq!(Arch::from_target("aarch64-unknown-none-softfloat")?, Arch::Aarch64);
        assert_eq!(Arch::from_target("riscv64gc-unknown-none-gnu")?, Arch::Riscv64);
        assert_eq!(Arch::from_target("riscv64-unknown-none")?, Arch::Riscv64);
        assert_eq!(Arch::from_target("x86_64-unknown-none-gnu")?, Arch::X86_64);
        assert_eq!(Arch::from_target("x86_64-unknown-none")?, Arch::X86_64);

        // Test invalid target
        assert!(Arch::from_target("invalid-target").is_err());
        Ok(())
    }

    #[test]
    fn test_arch_qemu_program() {
        assert_eq!(Arch::Aarch64.qemu_program(), "qemu-system-aarch64");
        assert_eq!(Arch::Riscv64.qemu_program(), "qemu-system-riscv64");
        assert_eq!(Arch::X86_64.qemu_program(), "qemu-system-x86_64");
    }

    #[test]
    fn test_arch_qemu_arch() {
        assert_eq!(Arch::Aarch64.qemu_arch(), "aarch64");
        assert_eq!(Arch::Riscv64.qemu_arch(), "riscv64");
        assert_eq!(Arch::X86_64.qemu_arch(), "x86_64");
    }

    #[test]
    fn test_arch_default() {
        let default_arch = Arch::default();
        assert_eq!(default_arch, Arch::Aarch64);
    }

    #[test]
    fn test_project_save_config() -> Result<()> {
        let test_dir = create_temp_dir();
        let mut project = Project::new(test_dir.clone());

        // Manually set config
        let config = create_test_project_config(&test_dir);
        project.config = Some(config.clone());

        project.save_config();

        // Verify config was saved
        let config_path = test_dir.join(".project.toml");
        assert!(config_path.exists());

        let saved_content = fs::read_to_string(&config_path)?;
        let loaded_config: ProjectConfig = toml::from_str(&saved_content)?;

        assert_eq!(loaded_config.compile.target, config.compile.target);
        assert_eq!(loaded_config.qemu.machine, config.qemu.machine);

        cleanup_temp_dir(&test_dir);
        Ok(())
    }
}