use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use ostool::{
    project::Project,
    step::{Step, Compile, TestPrepare},
    config::ProjectConfig,
};

#[cfg(test)]
mod step_tests {
    use super::*;

    fn create_temp_dir() -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let test_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let test_dir = temp_dir.join(format!("ostool_step_test_{}", test_id));
        fs::create_dir_all(&test_dir).unwrap();
        test_dir
    }

    fn cleanup_temp_dir(test_dir: &PathBuf) {
        let _ = fs::remove_dir_all(test_dir);
    }

    fn create_test_project(test_dir: &PathBuf) -> Project {
        // Create a minimal Cargo project structure
        let cargo_toml = test_dir.join("Cargo.toml");
        fs::write(&cargo_toml, r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "test_binary"
path = "src/main.rs"
"#).unwrap();

        let src_dir = test_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let main_rs = src_dir.join("main.rs");
        fs::write(&main_rs, r#"
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
"#).unwrap();

        // Create config
        let config_path = test_dir.join(".project.toml");
        fs::write(&config_path, r#"
[compile]
target = "aarch64-unknown-none"

[compile.build.Cargo]
package = "test-project"
kernel_bin_name = "test_binary"
kernel_is_bin = true

[qemu]
machine = "virt"
cpu = "cortex-a57"
graphic = false
"#).unwrap();

        let mut project = Project::new(test_dir.clone());
        project.config_with_file().unwrap();
        project
    }

    #[test]
    fn test_compile_new_boxed() {
        let compile_step = Compile::new_boxed(false);
        assert!(!compile_step.as_ref().is_debug); // Check it's not debug mode
    }

    #[test]
    fn test_compile_new_boxed_debug() {
        let compile_step = Compile::new_boxed(true);
        assert!(compile_step.as_ref().is_debug); // Check it's debug mode
    }

    #[test]
    fn test_test_prepare_new_boxed() {
        let test_step = TestPrepare::new_boxed(None, false, false);
        assert!(test_step.as_ref().elf.is_none());
        assert!(!test_step.as_ref().uboot);
    }

    #[test]
    fn test_test_prepare_new_boxed_with_elf() {
        let test_step = TestPrepare::new_boxed(Some("test.elf".to_string()), true, true);
        assert_eq!(test_step.as_ref().elf.as_ref().unwrap(), "test.elf");
        assert!(test_step.as_ref().uboot);
    }

    #[test]
    fn test_test_prepare_run_without_elf() -> Result<()> {
        let test_dir = create_temp_dir();
        let project = create_test_project(&test_dir);

        let mut test_step = TestPrepare::new_boxed(None, false, false);

        // Should run successfully without ELF (standard test mode)
        let result = test_step.run(&mut project);
        assert!(result.is_ok());

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_test_prepare_run_with_elf_missing_file() -> Result<()> {
        let test_dir = create_temp_dir();
        let project = create_test_project(&test_dir);

        let mut test_step = TestPrepare::new_boxed(
            Some("nonexistent_elf.elf".to_string()),
            false,
            true
        );

        // Should fail because ELF doesn't exist
        let result = test_step.run(&mut project);
        assert!(result.is_err());

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_create_fake_elf_for_test() -> Result<()> {
        let test_dir = create_temp_dir();
        let project = create_test_project(&test_dir);

        // Create a fake ELF file (just for testing the step logic)
        let fake_elf_path = test_dir.join("fake_test.elf");
        fs::write(&fake_elf_path, vec![0x7f, b'E', b'L', b'F', 0x02, 0x01, 0x01, 0x00])?;

        let mut test_step = TestPrepare::new_boxed(
            Some(fake_elf_path.to_string_lossy().to_string()),
            false,
            true
        );

        // This will likely fail because it's not a real ELF, but tests the flow
        let result = test_step.run(&mut project);
        // We expect this to fail due to invalid ELF, but that's expected behavior
        assert!(result.is_err());

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_step_trait_implementations() {
        // Test that all steps implement the Step trait
        fn assert_step<T: Step + ?Sized>(_: &T) {}

        let compile_step = Compile::new_boxed(false);
        assert_step(compile_step.as_ref());

        let test_step = TestPrepare::new_boxed(None, false, false);
        assert_step(test_step.as_ref());
    }

    #[test]
    fn test_multiple_steps_execution() -> Result<()> {
        let test_dir = create_temp_dir();
        let mut project = create_test_project(&test_dir);

        let mut steps: Vec<Box<dyn Step>> = vec![];

        // Add test preparation step
        steps.push(TestPrepare::new_boxed(None, false, false));

        // Execute all steps
        for step in &mut steps {
            let result = step.run(&mut project);
            assert!(result.is_ok(), "Step execution failed: {:?}", result);
        }

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_step_error_handling() -> Result<()> {
        let test_dir = create_temp_dir();
        let mut project = create_test_project(&test_dir);

        let mut test_step = TestPrepare::new_boxed(
            Some("definitely_missing_elf.elf".to_string()),
            false,
            true
        );

        let result = test_step.run(&mut project);
        assert!(result.is_err());

        // Verify error message contains useful information
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.len() > 0, "Error message should not be empty");

        cleanup_temp_dir(&test_dir);
        Ok(())
    }

    #[test]
    fn test_project_state_after_step_execution() -> Result<()> {
        let test_dir = create_temp_dir();
        let mut project = create_test_project(&test_dir);

        // Before step execution
        assert!(project.config.is_some());
        assert!(project.arch.is_some());

        let mut test_step = TestPrepare::new_boxed(None, false, false);
        test_step.run(&mut project)?;

        // After step execution (verify project state is maintained)
        assert!(project.config.is_some());
        assert!(project.arch.is_some());

        cleanup_temp_dir(&test_dir);
        Ok(())
    }
}