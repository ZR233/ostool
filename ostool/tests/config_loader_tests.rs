use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use ostool::config::{loader::{ConfigLoader, ConfigMode}, ProjectConfig};

#[cfg(test)]
mod config_loader_tests {
    use super::*;

    fn create_temp_dir() -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let test_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let test_dir = temp_dir.join(format!("ostool_test_{}", test_id));
        fs::create_dir_all(&test_dir).unwrap();
        test_dir
    }

    fn cleanup_temp_dir(test_dir: &PathBuf) {
        let _ = fs::remove_dir_all(test_dir);
    }

    #[test]
    fn test_config_loader_new() -> Result<()> {
        let test_dir = create_temp_dir();
        let _loader = ConfigLoader::new(&test_dir)?;
        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_load_normal_config_without_file() -> Result<()> {
        let test_dir = create_temp_dir();
        let loader = ConfigLoader::new(&test_dir)?;

        // This should create a new config via UI, but we can't test UI in unit tests
        // So we'll create a minimal config file first
        let config_path = test_dir.join(".project.toml");
        fs::write(&config_path, r#"
[compile]
target = "aarch64-unknown-none"

[compile.build.Custom]
shell = ["echo test"]
kernel = "test.bin"

[qemu]
machine = "virt"
cpu = "cortex-a57"
graphic = false
"#)?;

        let config = loader.load_config(ConfigMode::Normal)?;
        assert_eq!(config.compile.target, "aarch64-unknown-none");
        assert_eq!(config.qemu.machine, Some("virt".to_string()));

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_load_config_with_include() -> Result<()> {
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

include = ["board.toml"]
"#)?;

        // Create board config
        let board_config = test_dir.join("board.toml");
        fs::write(&board_config, r#"
[uboot]
serial = "/dev/ttyUSB0"
baud_rate = 115200
dtb_file = "board.dtb"
"#)?;

        let loader = ConfigLoader::new(&test_dir)?;
        let config = loader.load_config(ConfigMode::Normal)?;

        assert!(config.uboot.is_some());
        let uboot_config = config.uboot.unwrap();
        assert_eq!(uboot_config.serial, "/dev/ttyUSB0");
        assert_eq!(uboot_config.baud_rate, 115200);

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_load_test_config_with_board_mode() -> Result<()> {
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
serial = "/dev/ttyACM0"
baud_rate = 9600
dtb_file = "test_board.dtb"
"#)?;

        let loader = ConfigLoader::new(&test_dir)?;
        let config = loader.load_config(ConfigMode::Test {
            elf_path: None,
            board_mode: true
        })?;

        assert!(config.uboot.is_some());
        let uboot_config = config.uboot.unwrap();
        assert_eq!(uboot_config.serial, "/dev/ttyACM0");
        assert_eq!(uboot_config.baud_rate, 9600);

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_load_config_with_missing_include() -> Result<()> {
        let test_dir = create_temp_dir();

        // Create main config with missing include
        let main_config = test_dir.join(".project.toml");
        fs::write(&main_config, r#"
[compile]
target = "aarch64-unknown-none"

[compile.build.Custom]
shell = ["echo test"]
kernel = "test.bin"

[include]
missing_file = ["nonexistent.toml"]
"#)?;

        let loader = ConfigLoader::new(&test_dir)?;
        let config = loader.load_config(ConfigMode::Normal)?;

        // Should still load main config despite missing include
        assert_eq!(config.compile.target, "aarch64-unknown-none");

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_merge_config() -> Result<()> {
        let test_dir = create_temp_dir();

        // Create base config
        let base_config = test_dir.join("base.toml");
        fs::write(&base_config, r#"
[compile]
target = "aarch64-unknown-none"

[compile.build.Custom]
shell = ["echo base"]
kernel = "base.bin"

[qemu]
machine = "virt"
"#)?;

        // Create overlay config
        let overlay_config = test_dir.join("overlay.toml");
        fs::write(&overlay_config, r#"
[compile]
target = "riscv64gc-unknown-none-gnu"

[qemu]
cpu = "cortex-a72"
graphic = true
"#)?;

        let loader = ConfigLoader::new(&test_dir)?;

        // Load base config
        let mut base_config_data: ProjectConfig = toml::from_str(&fs::read_to_string(&base_config)?)?;

        // Load overlay config
        let overlay_config_data: ProjectConfig = toml::from_str(&fs::read_to_string(&overlay_config)?)?;

        // Manually merge (simulating what ConfigLoader does)
        loader.merge_config(&mut base_config_data, &overlay_config_data);

        // Check that overlay took precedence
        assert_eq!(base_config_data.compile.target, "riscv64gc-unknown-none-gnu");
        assert_eq!(base_config_data.qemu.cpu, Some("cortex-a72".to_string()));
        assert_eq!(base_config_data.qemu.graphic, true);

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_corrupted_config_handling() -> Result<()> {
        let test_dir = create_temp_dir();

        // Create corrupted config
        let config_path = test_dir.join(".project.toml");
        fs::write(&config_path, "invalid toml content [[[")?;

        let loader = ConfigLoader::new(&test_dir)?;

        // This should backup the corrupted file and create a new one
        // In unit tests we can't test the UI part, but we can verify the backup was created
        let result = loader.load_config(ConfigMode::Normal);

        // Should fail because we can't create UI config in unit tests
        assert!(result.is_err());

        // Check that backup was created
        let backups: Vec<_> = fs::read_dir(&test_dir)?
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
        Ok(())
    }
}