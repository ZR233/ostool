use std::{
    collections::HashMap,
    path::{Path, PathBuf},
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

use crate::{build::config::BuildConfig, utils::prepare_config};

#[derive(Default, Clone)]
pub struct AppContext {
    pub workspace_folder: PathBuf,
    pub manifest_dir: PathBuf,
    pub debug: bool,
    pub elf_path: Option<PathBuf>,
    pub bin_path: Option<PathBuf>,
    pub arch: Option<Architecture>,
    pub build_config: Option<BuildConfig>,
    pub build_config_path: Option<PathBuf>,
}

fn run_tui(app_data: AppData) -> anyhow::Result<()> {
    let title = app_data.root.title.clone();
    let fields = app_data.root.menu().fields();

    cursive::logger::init();
    cursive::logger::set_filter_levels_from_env();
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
    app.on_exit()?;

    Ok(())
}

impl AppContext {
    pub fn shell_run_cmd(&self, cmd: &str) -> anyhow::Result<()> {
        let mut command = self.command("sh");
        command.arg("-c");
        command.arg(cmd);

        if let Some(elf) = &self.elf_path {
            command.env("KERNEL_ELF", elf.display().to_string());
        }

        command.run()?;

        Ok(())
    }

    // Helper function to launch jkconfig UI
    pub fn launch_jkconfig_ui(config_path: &Path, schema_path: &Path) -> anyhow::Result<bool> {
        let mut app_data = AppData::new(Some(config_path), Some(schema_path))?;

        // app_data.features_callback = Some(std::sync::Arc::new(|| {
        //     let mut features = Vec::new();

        //     if let Ok(metadata) = cargo_metadata::MetadataCommand::new().no_deps().exec() {
        //         let workspace_root = metadata.workspace_root.clone();
        //         if let Some(current_package) = metadata
        //             .packages
        //             .iter()
        //             .find(|p| p.manifest_path.starts_with(&workspace_root))
        //         {
        //             for (feature_name, _) in &current_package.features {
        //                 features.push(feature_name.clone());
        //             }
        //         }
        //     }
        //     features
        // }));

        // // 设置depend_features_callback以获取依赖项及其features
        // app_data.depend_features_callback = Some(std::sync::Arc::new(|| {
        //     let mut depend_features = HashMap::new();

        //     if let Ok(metadata) = cargo_metadata::MetadataCommand::new().exec() {
        //         let workspace_root = metadata.workspace_root.clone();
        //         if let Some(current_package) = metadata
        //             .packages
        //             .iter()
        //             .find(|p| p.manifest_path.starts_with(&workspace_root))
        //         {
        //             // 遍历所有依赖项
        //             for dependency in &current_package.dependencies {
        //                 let dep_name = dependency.name.clone();
        //                 let mut dep_features = Vec::new();

        //                 if let Some(dep_package) =
        //                     metadata.packages.iter().find(|p| p.name == dep_name)
        //                 {
        //                     for (feature_name, _) in &dep_package.features {
        //                         dep_features.push(feature_name.clone());
        //                     }
        //                 }

        //                 depend_features.insert(dep_name, dep_features);
        //             }
        //         }
        //     }
        //     depend_features
        // }));

        run_tui(app_data)?;

        Ok(true)
    }

    // axvisor
    pub async fn launch_menuconfig_ui(
        &mut self,
        self_features: Vec<String>,
        depend_features: HashMap<String, Vec<String>>,
    ) -> anyhow::Result<bool> {
        let config_path = match &self.build_config_path {
            Some(path) => path.clone(),
            None => self.workspace_folder.join(".build.toml"),
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

        let mut app_data = AppData::new(Some(config_path), Some(schema_path))?;

        app_data.features_callback = Some(std::sync::Arc::new(move || self_features.clone()));

        app_data.depend_features_callback =
            Some(std::sync::Arc::new(move || depend_features.clone()));

        run_tui(app_data)?;

        Ok(true)
    }

    pub fn command(&self, program: &str) -> crate::utils::Command {
        let this = self.clone();
        crate::utils::Command::new(program, &self.manifest_dir, move |s| {
            this.value_replace_with_var(s)
        })
    }

    pub fn metadata(&self) -> anyhow::Result<Metadata> {
        let res = cargo_metadata::MetadataCommand::new()
            .current_dir(&self.manifest_dir)
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
                        self.prepare_launch_ui().await
                    }
                }
            }
            Err(e) => {
                println!("Failed to read configuration file: {}", e);
                self.prepare_launch_ui().await
            }
        }
    }

    /// Launch configuration UI interface and get configuration
    pub async fn prepare_launch_ui(&mut self) -> anyhow::Result<BuildConfig> {
        println!("Launching UI interface for configuration editing...");
        // Get configuration file path
        let config_path = match &self.build_config_path {
            Some(path) => path.clone(),
            None => self.workspace_folder.join(".build.toml"),
        };

        // Ensure the configuration file exists
        if !config_path.exists() {
            println!(
                "Configuration file does not exist, creating new file: {}",
                config_path.display()
            );
            tokio::fs::write(&config_path, "").await?;
        }

        let schema_path = default_schema_by_init(&config_path);

        if !config_path.exists() {
            println!(
                "Creating empty configuration file: {}",
                config_path.display()
            );
            std::fs::write(&config_path, "")?;
        }

        println!("Starting configuration UI interface...");

        let _ = Self::launch_jkconfig_ui(&config_path, &schema_path)?;

        let config_content: String = tokio::fs::read_to_string(&config_path)
            .await
            .map_err(|e| anyhow!("Failed to read configuration file after UI editing: {}", e))?;

        let config: BuildConfig = toml::from_str(&config_content)
            .map_err(|e| anyhow!("Failed to parse configuration file after UI editing: {}", e))?;

        self.build_config = Some(config.clone());

        Ok(config)
    }

    pub fn is_cargo_build(&self) -> bool {
        match &self.build_config {
            Some(cfg) => matches!(cfg.system, crate::build::config::BuildSystem::Cargo(_)),
            None => false,
        }
    }

    pub fn value_replace_with_var<S>(&self, value: S) -> String
    where
        S: AsRef<std::ffi::OsStr>,
    {
        let raw = value.as_ref().to_string_lossy();
        raw.replace(
            "${workspaceFolder}",
            format!("{}", self.workspace_folder.display()).as_ref(),
        )
    }
}
