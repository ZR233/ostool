use std::{path::PathBuf, sync::Arc};

use anyhow::anyhow;
use cargo_metadata::Metadata;
use colored::Colorize;
use cursive::Cursive;
use jkconfig::{
    ElemHock,
    data::{app_data::AppData, item::ItemType, types::ElementType},
    ui::components::editors::{show_feature_select, show_list_select},
};

use object::{Architecture, Object};
use tokio::fs;

use crate::build::config::BuildConfig;

/// Configuration for output directories (set from external config)
#[derive(Default, Clone)]
pub struct OutputConfig {
    pub build_dir: Option<PathBuf>,
    pub bin_dir: Option<PathBuf>,
}

/// Build artifacts (generated during build)
#[derive(Default, Clone)]
pub struct OutputArtifacts {
    pub elf: Option<PathBuf>,
    pub bin: Option<PathBuf>,
}

/// Path configuration grouping all path-related fields
#[derive(Default, Clone)]
pub struct PathConfig {
    pub workspace: PathBuf,
    pub manifest: PathBuf,
    pub config: OutputConfig,
    pub artifacts: OutputArtifacts,
}

impl PathConfig {
    /// Get build directory, defaulting to manifest/target if not configured
    pub fn build_dir(&self) -> PathBuf {
        self.config
            .build_dir
            .clone()
            .unwrap_or_else(|| self.manifest.join("target"))
    }

    /// Get bin directory, defaulting to build_dir if not configured
    pub fn bin_dir(&self) -> Option<PathBuf> {
        self.config.bin_dir.clone()
    }
}

#[derive(Default, Clone)]
pub struct AppContext {
    pub paths: PathConfig,
    pub debug: bool,
    pub arch: Option<Architecture>,
    pub build_config: Option<BuildConfig>,
    pub build_config_path: Option<PathBuf>,
}

impl AppContext {
    pub fn shell_run_cmd(&self, cmd: &str) -> anyhow::Result<()> {
        let mut command = self.command("sh");
        command.arg("-c");
        command.arg(cmd);

        if let Some(elf) = &self.paths.artifacts.elf {
            command.env("KERNEL_ELF", elf.display().to_string());
        }

        command.run()?;

        Ok(())
    }

    pub fn command(&self, program: &str) -> crate::utils::Command {
        let this = self.clone();
        crate::utils::Command::new(program, &self.paths.manifest, move |s| {
            this.value_replace_with_var(s)
        })
    }

    pub fn metadata(&self) -> anyhow::Result<Metadata> {
        let res = cargo_metadata::MetadataCommand::new()
            .current_dir(&self.paths.manifest)
            .no_deps()
            .exec()?;
        Ok(res)
    }

    pub async fn set_elf_path(&mut self, path: PathBuf) {
        self.paths.artifacts.elf = Some(path.clone());
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

    pub fn objcopy_elf(&mut self) -> anyhow::Result<PathBuf> {
        let elf_path = self
            .paths
            .artifacts
            .elf
            .as_ref()
            .ok_or(anyhow!("elf not exist"))?
            .canonicalize()?;

        let stripped_elf_path = elf_path.with_file_name(
            elf_path
                .file_stem()
                .ok_or(anyhow!("Invalid file path"))?
                .to_string_lossy()
                .to_string()
                + ".elf",
        );
        println!(
            "{}",
            format!(
                "Stripping ELF file...\r\n  original elf: {}\r\n  stripped elf: {}",
                elf_path.display(),
                stripped_elf_path.display()
            )
            .bold()
            .purple()
        );

        let mut objcopy = self.command("rust-objcopy");

        objcopy.arg(format!(
            "--binary-architecture={}",
            format!("{:?}", self.arch.unwrap()).to_lowercase()
        ));
        objcopy.arg(&elf_path);
        objcopy.arg(&stripped_elf_path);

        objcopy.run()?;
        self.paths.artifacts.elf = Some(stripped_elf_path.clone());

        Ok(stripped_elf_path)
    }

    pub fn objcopy_output_bin(&mut self) -> anyhow::Result<PathBuf> {
        if self.paths.artifacts.bin.is_some() {
            debug!("BIN file already exists: {:?}", self.paths.artifacts.bin);
            return Ok(self.paths.artifacts.bin.as_ref().unwrap().clone());
        }

        let elf_path = self
            .paths
            .artifacts
            .elf
            .as_ref()
            .ok_or(anyhow!("elf not exist"))?
            .canonicalize()?;

        let bin_name = elf_path
            .file_stem()
            .ok_or(anyhow!("Invalid file path"))?
            .to_string_lossy()
            .to_string()
            + ".bin";

        let bin_path = if let Some(bin_dir) = self.paths.config.bin_dir.clone() {
            bin_dir.join(bin_name)
        } else {
            elf_path.with_file_name(bin_name)
        };

        if let Some(parent) = bin_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

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
        self.paths.artifacts.bin = Some(bin_path.clone());

        Ok(bin_path)
    }

    pub async fn prepare_build_config(
        &mut self,
        config_path: Option<PathBuf>,
        menu: bool,
    ) -> anyhow::Result<BuildConfig> {
        let config_path = match config_path {
            Some(path) => path,
            None => self.paths.workspace.join(".build.toml"),
        };
        self.build_config_path = Some(config_path.clone());

        let Some(c): Option<BuildConfig> = jkconfig::run(
            config_path,
            menu,
            &[self.ui_hock_feature_select(), self.ui_hock_pacage_select()],
        )
        .await?
        else {
            anyhow::bail!("No build configuration obtained");
        };

        self.build_config = Some(c.clone());
        Ok(c)
    }

    pub fn value_replace_with_var<S>(&self, value: S) -> String
    where
        S: AsRef<std::ffi::OsStr>,
    {
        let raw = value.as_ref().to_string_lossy();
        raw.replace(
            "${workspaceFolder}",
            format!("{}", self.paths.workspace.display()).as_ref(),
        )
    }

    pub fn ui_hocks(&self) -> Vec<ElemHock> {
        vec![self.ui_hock_feature_select(), self.ui_hock_pacage_select()]
    }

    fn ui_hock_feature_select(&self) -> ElemHock {
        let path = "system.features";
        let cargo_toml = self.paths.workspace.join("Cargo.toml");
        ElemHock {
            path: path.to_string(),
            callback: Arc::new(move |siv: &mut Cursive, _path: &str| {
                let mut package = String::new();
                if let Some(app) = siv.user_data::<AppData>()
                    && let Some(pkg) = app.root.get_by_key("system.package")
                    && let ElementType::Item(item) = pkg
                    && let ItemType::String { value: Some(v), .. } = &item.item_type
                {
                    package = v.clone();
                }

                // 调用显示特性选择对话框的函数
                show_feature_select(siv, &package, &cargo_toml, None);
            }),
        }
    }

    fn ui_hock_pacage_select(&self) -> ElemHock {
        let path = "system.package";
        let cargo_toml = self.paths.workspace.join("Cargo.toml");

        ElemHock {
            path: path.to_string(),
            callback: Arc::new(move |siv: &mut Cursive, path: &str| {
                let mut items = Vec::new();
                if let Ok(metadata) = cargo_metadata::MetadataCommand::new()
                    .manifest_path(&cargo_toml)
                    .no_deps()
                    .exec()
                {
                    for pkg in &metadata.packages {
                        items.push(pkg.name.to_string());
                    }
                }

                // 调用显示包选择对话框的函数
                show_list_select(siv, "Pacage", &items, path, on_package_selected);
            }),
        }
    }
}

fn on_package_selected(app: &mut AppData, path: &str, selected: &str) {
    let ElementType::Item(item) = app.root.get_mut_by_key(path).unwrap() else {
        panic!("Not an item element");
    };
    let ItemType::String { value, .. } = &mut item.item_type else {
        panic!("Not a string item");
    };
    *value = Some(selected.to_string());
}
