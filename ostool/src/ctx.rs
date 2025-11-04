use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::anyhow;
use cargo_metadata::Metadata;
use colored::Colorize;
use jkconfig::data::app_data::default_schema_by_init;

use object::{Architecture, Object};
use tokio::fs;

use crate::{
    build::config::BuildConfig,
    utils::{ShellRunner, prepare_config},
};

#[derive(Default, Clone)]
pub struct AppContext {
    pub workdir: PathBuf,
    pub debug: bool,
    pub elf_path: Option<PathBuf>,
    pub bin_path: Option<PathBuf>,
    pub arch: Option<Architecture>,
    pub build_config: Option<BuildConfig>,
    pub build_config_path: Option<PathBuf>,
}

impl AppContext {
    pub fn shell_run_cmd(&self, cmd: &str) -> anyhow::Result<()> {
        let mut parts = cmd.split_whitespace();
        let mut command = self.command(parts.next().unwrap());
        command.current_dir(&self.workdir);
        for arg in parts {
            command.arg(arg);
        }
        if let Some(elf) = &self.elf_path {
            command.env("KERNEL_ELF", elf.display().to_string());
        }

        command.run()?;

        Ok(())
    }

    // Helper function to launch jkconfig UI
    pub fn launch_jkconfig_ui(config_path: &Path, schema_path: &Path) -> anyhow::Result<bool> {
        // First try standalone binary
        println!("Trying to run jkconfig as standalone binary");
        if let Ok(mut cmd) = std::process::Command::new("jkconfig")
            .arg("-c")
            .arg(config_path.to_string_lossy().as_ref())
            .arg("-s")
            .arg(schema_path.to_string_lossy().as_ref())
            .arg("tui")
            .spawn()
        {
            if let Ok(status) = cmd.wait() {
                if status.success() {
                    return Ok(true);
                }
            }
        }

        // If binary failed, try cargo run
        println!("Trying to run jkconfig via cargo");
        if let Ok(mut cmd) = std::process::Command::new("cargo")
            .args(["run", "--bin=jkconfig"])
            .args([
                "--",
                "-c",
                config_path.to_string_lossy().as_ref(),
                "-s",
                schema_path.to_string_lossy().as_ref(),
                "tui",
            ])
            .spawn()
        {
            if let Ok(status) = cmd.wait() {
                if status.success() {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub fn command(&self, program: &str) -> Command {
        let mut command = Command::new(program);
        command.current_dir(&self.workdir);
        command
    }

    pub fn metadata(&self) -> anyhow::Result<Metadata> {
        let res = cargo_metadata::MetadataCommand::new()
            .current_dir(&self.workdir)
            .no_deps()
            .exec()?;
        Ok(res)
    }

    pub async fn set_elf_path(&mut self, path: PathBuf) {
        self.elf_path = Some(path.clone());
        let binary_data = match fs::read(path).await {
            Ok(data) => data,
            Err(e) => {
                println!("Failed to read ELF file: {e}");
                return;
            }
        };
        let file = match object::File::parse(binary_data.as_slice()) {
            Ok(f) => f,
            Err(e) => {
                println!("Failed to parse ELF file: {e}");
                return;
            }
        };
        self.arch = Some(file.architecture())
    }

    pub fn objcopy_output_bin(&mut self) -> anyhow::Result<PathBuf> {
        if self.bin_path.is_some() {
            debug!("BIN file already exists: {:?}", self.bin_path);
            return Ok(self.bin_path.as_ref().unwrap().clone());
        }

        let elf_path = self
            .elf_path
            .as_ref()
            .ok_or(anyhow!("elf not exist"))?
            .canonicalize()?;

        // 去掉原文件的扩展名后添加 .bin
        let bin_path = elf_path.with_file_name(
            elf_path
                .file_stem()
                .ok_or(anyhow!("Invalid file path"))?
                .to_string_lossy()
                .to_string()
                + ".bin",
        );
        println!(
            "{}",
            format!(
                "Converting ELF to BIN format...\r\n  elf: {}\r\n  bin: {}",
                elf_path.display(),
                bin_path.display()
            )
            .bold()
            .purple()
        );

        let mut objcopy = self.command("rust-objcopy");

        if !self.debug {
            objcopy.arg("--strip-all");
        }

        objcopy
            .arg("-O")
            .arg("binary")
            .arg(&elf_path)
            .arg(&bin_path);

        objcopy.run()?;
        self.bin_path = Some(bin_path.clone());

        Ok(bin_path)
    }

    // pub fn objcopy_output_bin(&mut self) -> anyhow::Result<PathBuf> {
    //     let elf_path = self.elf_path.as_ref().ok_or(anyhow!("elf not exist"))?;
    //     let bin_path = elf_path.with_extension("bin");
    //     println!(
    //         "{}",
    //         format!(
    //             "Converting ELF to BIN format...\r\n  elf: {}\r\n  bin: {}",
    //             elf_path.display(),
    //             bin_path.display()
    //         )
    //         .bold()
    //         .purple()
    //     );

    //     // Read ELF file
    //     let binary_data =
    //         std::fs::read(elf_path).map_err(|e| anyhow!("Failed to read ELF file: {}", e))?;

    //     // Parse ELF file
    //     let obj_file = object::File::parse(binary_data.as_slice())
    //         .map_err(|e| anyhow!("Failed to parse ELF file: {}", e))?;

    //     // Extract loadable segments and write to binary file
    //     let mut binary_output = Vec::new();
    //     let mut min_addr = u64::MAX;
    //     let mut max_addr = 0u64;

    //     // First pass: find memory range
    //     for segment in obj_file.segments() {
    //         // Only include loadable segments
    //         if segment.size() > 0 {
    //             let addr = segment.address();
    //             min_addr = min_addr.min(addr);
    //             max_addr = max_addr.max(addr + segment.size());
    //         }
    //     }

    //     if min_addr == u64::MAX {
    //         return Err(anyhow!("No loadable segments found in ELF file"));
    //     }

    //     // Allocate buffer for binary output
    //     let total_size = (max_addr - min_addr) as usize;
    //     binary_output.resize(total_size, 0u8);

    //     // Second pass: copy segment data
    //     for segment in obj_file.segments() {
    //         if let Ok(data) = segment.data()
    //             && !data.is_empty()
    //         {
    //             let addr = segment.address();
    //             let offset = (addr - min_addr) as usize;
    //             if offset + data.len() <= binary_output.len() {
    //                 binary_output[offset..offset + data.len()].copy_from_slice(data);
    //             }
    //         }
    //     }

    //     // Write binary file
    //     std::fs::write(&bin_path, binary_output)
    //         .map_err(|e| anyhow!("Failed to write binary file: {}", e))?;

    //     self.bin_path = Some(bin_path.clone());
    //     Ok(bin_path)
    // }

    pub async fn perpare_build_config(
        &mut self,
        config_path: Option<PathBuf>,
    ) -> anyhow::Result<BuildConfig> {
        // Try to get configuration content, launch UI interface if failed
        match prepare_config::<BuildConfig>(self, config_path, ".build.toml").await {
            Ok(content) => {
                // Try to parse configuration, launch UI interface if parsing failed
                match toml::from_str::<BuildConfig>(&content) {
                    Ok(config) => {
                        println!("Build configuration: {:?}", config);
                        self.build_config = Some(config.clone());
                        Ok(config)
                    }
                    Err(e) => {
                        println!("Configuration file parsing failed: {}", e);
                        println!("Launching UI interface for configuration editing...");
                        self.launch_config_ui_and_get_config().await
                    }
                }
            }
            Err(e) => {
                println!("Failed to read configuration file: {}", e);
                println!("Launching UI interface to create/edit configuration...");
                self.launch_config_ui_and_get_config().await
            }
        }
    }

    /// Launch configuration UI interface and get configuration
    async fn launch_config_ui_and_get_config(&mut self) -> anyhow::Result<BuildConfig> {
        // Get configuration file path
        let config_path = match &self.build_config_path {
            Some(path) => path.clone(),
            None => self.workdir.join(".build.toml"),
        };

        // Ensure the configuration file exists
        if !config_path.exists() {
            println!(
                "Configuration file does not exist, creating new file: {}",
                config_path.display()
            );
            tokio::fs::write(&config_path, "").await?;
        }

        // Generate schema path
        let schema_path = default_schema_by_init(&config_path);

        // Create empty config file if it doesn't exist
        if !config_path.exists() {
            println!(
                "Creating empty configuration file: {}",
                config_path.display()
            );
            std::fs::write(&config_path, "")?;
        }

        // Directly run jkconfig as a standalone binary if available
        println!("Starting configuration UI interface...");

        // Try to run jkconfig in different ways
        let ui_success = Self::launch_jkconfig_ui(&config_path, &schema_path)?;

        // Print success message based on UI launch status
        if ui_success {
            println!("UI interface launched successfully");
        } else {
            println!("Warning: Failed to launch jkconfig UI");
            println!("Will attempt to continue with existing configuration");
        }

        // Re-read and parse the configuration
        let config_content = tokio::fs::read_to_string(&config_path)
            .await
            .map_err(|e| anyhow!("Failed to read configuration file after UI editing: {}", e))?;

        let config: BuildConfig = toml::from_str(&config_content)
            .map_err(|e| anyhow!("Failed to parse configuration file after UI editing: {}", e))?;

        println!("Configuration has been updated from UI editor");
        self.build_config = Some(config.clone());
        Ok(config)
    }

    pub fn is_cargo_build(&self) -> bool {
        match &self.build_config {
            Some(cfg) => matches!(cfg.system, crate::build::config::BuildSystem::Cargo(_)),
            None => false,
        }
    }
}
