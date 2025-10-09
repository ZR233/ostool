use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use colored::Colorize;

use crate::{
    config::{ProjectConfig, compile::BuildSystem},
    shell::metadata,
    step::UbootConfig,
};

/// Unified configuration loader that handles .project.toml and includes
pub struct ConfigLoader {
    workspace_root: PathBuf,
}

impl ConfigLoader {
    pub fn new(workdir: &Path) -> Result<Self> {
        let meta = metadata(workdir);
        Ok(Self {
            workspace_root: meta.workspace_root.as_std_path().to_path_buf(),
        })
    }

    /// Load configuration with support for includes and test-specific configs
    pub fn load_config(&self, mode: ConfigMode) -> Result<ProjectConfig> {
        let config_path = self.workspace_root.join(".project.toml");

        let mut config = if config_path.exists() {
            self.load_main_config(&config_path)?
        } else {
            self.create_default_config(&self.workspace_root)?
        };

        // Handle includes
        if let Some(include_files) = config.include.clone() {
            self.process_includes(&mut config, &include_files)?;
        }

        // Handle mode-specific configurations
        match mode {
            ConfigMode::Normal => {}
            ConfigMode::Test { elf_path, board_mode } => {
                self.setup_test_config(&mut config, elf_path, board_mode)?;
            }
        }

        Ok(config)
    }

    fn load_main_config(&self, config_path: &Path) -> Result<ProjectConfig> {
        let content = fs::read_to_string(config_path)?;

        let config_result = toml::from_str(&content);
        match config_result {
            Ok(config) => Ok(config),
            Err(e) => {
                // Backup old config and create new one
                let backup_path = self.workspace_root.join(format!(
                    ".project.toml.bk.{}",
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                ));

                println!(
                    "{}",
                    format!("Config parse error: {}, backing up to: {}", e, backup_path.display()).yellow()
                );

                let _ = fs::rename(config_path, &backup_path);

                // Create new config via UI
                Ok(ProjectConfig::new_by_ui(self.workspace_root.as_path()))
            }
        }
    }

    fn create_default_config(&self, workdir: &Path) -> Result<ProjectConfig> {
        let config = ProjectConfig::new_by_ui(workdir);
        let config_path = self.workspace_root.join(".project.toml");
        config.save(&config_path);
        Ok(config)
    }

    fn process_includes(&self, config: &mut ProjectConfig, include_files: &[PathBuf]) -> Result<()> {
        for include_path in include_files {
            if include_path.exists() {
                let include_content = fs::read_to_string(include_path)?;

                // Try to parse as UbootConfig first (for board.toml compatibility)
                if let Ok(uboot_config) = toml::from_str::<UbootConfig>(&include_content) {
                    config.uboot = Some(uboot_config);
                } else if let Ok(include_config) = toml::from_str::<ProjectConfig>(&include_content) {
                    // Merge full project config
                    self.merge_config(config, &include_config);
                }
            } else {
                println!(
                    "{}",
                    format!("Warning: Include file does not exist: {}", include_path.display()).yellow()
                );
            }
        }
        Ok(())
    }

    pub fn merge_config(&self, base: &mut ProjectConfig, overlay: &ProjectConfig) {
        // Simple merge strategy - overlay takes precedence
        if !overlay.compile.target.is_empty() {
            base.compile.target = overlay.compile.target.clone();
        }

        // Handle build system merge
        match (&base.compile.build, &overlay.compile.build) {
            (BuildSystem::Cargo(_), BuildSystem::Cargo(_)) => {
                // Cargo configs are complex, for now just use overlay
                base.compile.build = overlay.compile.build.clone();
            }
            (BuildSystem::Custom(_), BuildSystem::Custom(_)) |
            (BuildSystem::Custom(_), BuildSystem::Cargo(_)) |
            (BuildSystem::Cargo(_), BuildSystem::Custom(_)) => {
                base.compile.build = overlay.compile.build.clone();
            }
        }

        // Merge QEMU config
        if overlay.qemu.machine.is_some() || overlay.qemu.cpu.is_some() {
            base.qemu = overlay.qemu.clone();
        }

        // Merge U-Boot config
        if overlay.uboot.is_some() {
            base.uboot = overlay.uboot.clone();
        }
    }

    fn setup_test_config(&self, config: &mut ProjectConfig, elf_path: Option<String>, board_mode: bool) -> Result<()> {
        if board_mode {
            // Setup for board test - add .board.toml include if not present
            let board_toml_path = self.workspace_root.join(".board.toml");
            if board_toml_path.exists() {
                if config.include.is_none() {
                    config.include = Some(vec![]);
                }
                if let Some(ref mut includes) = config.include {
                    if !includes.contains(&board_toml_path) {
                        includes.push(board_toml_path);
                    }
                }
            }
        } else if let Some(_elf) = elf_path {
            // Setup for cargo test with specific ELF
            // This will be handled by the test preparation step
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigMode {
    Normal,
    Test {
        elf_path: Option<String>,
        board_mode: bool
    },
}