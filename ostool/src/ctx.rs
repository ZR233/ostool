use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::anyhow;
use cargo_metadata::Metadata;
use colored::Colorize;
use cursive::{Cursive, CursiveExt, event::Key};
use jkconfig::{
    data::app_data::{AppData, default_schema_by_init},
    ui::{components::menu::menu_view, handle_back, handle_quit, handle_save},
};

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
        // 创建AppData实例
        let mut app_data = AppData::new(Some(config_path), Some(schema_path))?;

        // 设置features_callback以获取本地仓库的features
        app_data.features_callback = Some(std::sync::Arc::new(|| {
            let mut features = Vec::new();

            // 尝试从当前目录获取cargo项目的features，类似metadata方法的实现
            if let Ok(metadata) = cargo_metadata::MetadataCommand::new().no_deps().exec() {
                // 获取workspace根目录
                let workspace_root = metadata.workspace_root.clone();

                // 查找当前仓库的包（manifest_path与workspace根目录匹配的包）
                if let Some(current_package) = metadata
                    .packages
                    .iter()
                    .find(|p| p.manifest_path.starts_with(&workspace_root))
                {
                    // 添加当前仓库包的所有features
                    info!("Current package: {}", current_package.name);
                    info!(
                        "features: {:?}",
                        current_package.features.keys().collect::<Vec<_>>()
                    );
                    info!(
                        "dependencies: {:?}",
                        current_package
                            .dependencies
                            .iter()
                            .map(|d| d.name.clone())
                            .collect::<Vec<_>>()
                    );
                    for (feature_name, _) in &current_package.features {
                        features.push(feature_name.clone());
                    }
                }
            } else {
                // 如果无法获取metadata，添加一些默认features
                features.push("default".to_string());
                info!("Failed to get cargo metadata. Adding default features.");
            }

            features
        }));

        // 设置depend_features_callback以获取依赖项及其features
        app_data.depend_features_callback = Some(std::sync::Arc::new(|| {
            let mut depend_features = HashMap::new();

            // 尝试从当前目录获取cargo项目的依赖项及其features
            if let Ok(metadata) = cargo_metadata::MetadataCommand::new().exec() {
                // 获取workspace根目录
                let workspace_root = metadata.workspace_root.clone();
                // 查找当前仓库的包（manifest_path与workspace根目录匹配的包）
                if let Some(current_package) = metadata
                    .packages
                    .iter()
                    .find(|p| p.manifest_path.starts_with(&workspace_root))
                {
                    // 获取所有依赖项及其features
                    info!("Current package: {}", current_package.name);
                    info!(
                        "dependencies: {:?}",
                        current_package
                            .dependencies
                            .iter()
                            .map(|d| d.name.clone())
                            .collect::<Vec<_>>()
                    );

                    // 遍历所有依赖项
                    for dependency in &current_package.dependencies {
                        let dep_name = dependency.name.clone();
                        let mut dep_features = Vec::new();

                        // 查找依赖包的features（需要在所有包中查找）
                        if let Some(dep_package) =
                            metadata.packages.iter().find(|p| p.name == dep_name)
                        {
                            info!("Dependency package: {}", dep_package.name);
                            info!(
                                "Dependency features: {:?}",
                                dep_package.features.keys().collect::<Vec<_>>()
                            );

                            // 添加依赖包的所有features
                            for (feature_name, _) in &dep_package.features {
                                dep_features.push(feature_name.clone());
                            }
                        }

                        // 如果没有找到依赖包的详细信息，添加一些默认features
                        if dep_features.is_empty() {
                            dep_features.push("default".to_string());
                        }

                        depend_features.insert(dep_name, dep_features);
                    }
                }
            } else {
                // 如果无法获取metadata，添加一些默认依赖项
                depend_features.insert(
                    "default-dependency".to_string(),
                    vec!["default".to_string()],
                );
                info!("Failed to get cargo metadata. Adding default dependency.");
            }

            depend_features
        }));

        let title = app_data.root.title.clone();
        let fields = app_data.root.menu().fields();

        // 添加调试日志
        info!(
            "depend_features_callback is set: {}",
            app_data.depend_features_callback.is_some()
        );
        info!(
            "features_callback is set: {}",
            app_data.features_callback.is_some()
        );

        cursive::logger::init();
        cursive::logger::set_internal_filter_level(log::LevelFilter::Info);

        // 创建Cursive应用
        let mut siv = Cursive::default();
        // 设置AppData为user_data
        siv.set_user_data(app_data);

        // 添加全局键盘事件处理
        siv.add_global_callback('q', handle_quit);
        siv.add_global_callback('Q', handle_quit);
        siv.add_global_callback('s', handle_save);
        siv.add_global_callback('S', handle_save);
        siv.add_global_callback(Key::Esc, handle_back);
        siv.add_global_callback('~', cursive::Cursive::toggle_debug_console);
        // 初始菜单路径为空
        siv.add_fullscreen_layer(menu_view(&title, "", fields));
        // 运行应用
        siv.run();

        println!("Exiting jkconfig...");
        let mut app = siv.take_user_data::<AppData>().unwrap();
        println!("Data: \n{:#?}", app.root);
        app.on_exit()?;

        Ok(true)
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
                        self.launch_config_ui_and_get_config().await
                    }
                }
            }
            Err(e) => {
                println!("Failed to read configuration file: {}", e);
                self.launch_config_ui_and_get_config().await
            }
        }
    }

    /// Launch configuration UI interface and get configuration
    async fn launch_config_ui_and_get_config(&mut self) -> anyhow::Result<BuildConfig> {
        println!("Launching UI interface for configuration editing...");
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
            println!("Warning: Failed to config jkconfig UI");
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
