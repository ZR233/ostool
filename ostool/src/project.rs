use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use cargo_metadata::{Metadata, Package};

use crate::{
    config::{ProjectConfig, compile::BuildSystem, loader::{ConfigLoader, ConfigMode}},
    shell::{Shell, check_porgram, metadata},
};

#[derive(Default)]
pub struct Project {
    workdir: PathBuf,
    pub config: Option<ProjectConfig>,
    pub arch: Option<Arch>,
    pub out_dir: Option<PathBuf>,
    pub kernel: Option<PathBuf>,
    pub is_print_cmd: bool,
}

impl Project {
    pub fn new(workdir: PathBuf) -> Self {
        Self {
            workdir,
            is_print_cmd: true,
            ..Default::default()
        }
    }

    pub fn config_with_file(&mut self) -> Result<()> {
        self.load_config(ConfigMode::Normal)
    }

    pub fn test_config(&mut self, elf_path: Option<String>, board_mode: bool) -> Result<()> {
        self.load_config(ConfigMode::Test { elf_path, board_mode })
    }

    fn load_config(&mut self, mode: ConfigMode) -> Result<()> {
        let loader = ConfigLoader::new(self.workdir())?;
        let config = loader.load_config(mode)?;
        self.arch = Some(Arch::from_target(&config.compile.target).unwrap());
        self.config = Some(config);
        Ok(())
    }

    
    pub fn config_ref(&self) -> &ProjectConfig {
        self.config.as_ref().unwrap()
    }

    pub fn workdir(&self) -> &Path {
        &self.workdir
    }

    pub fn shell<S: AsRef<OsStr>>(&self, program: S) -> Command {
        let mut cmd = Command::new(program);
        cmd.current_dir(self.workdir());
        cmd
    }

    pub fn prepere_deps(&self) {
        println!("check dependencies");
        if !check_porgram("rust-objcopy") {
            self.install_deps();
        }
        println!("dependencies ready");
    }

    pub fn save_config(&self) {
        let config_path = self.workdir().join(".project.toml");
        self.config_ref().save(&config_path);
    }

    fn install_deps(&self) {
        println!("install dependencies");
        self.shell("cargo")
            .args(["install", "cargo-binutils"])
            .exec(self.is_print_cmd)
            .unwrap();
        self.shell("rustup")
            .args(["component", "add", "llvm-tools-preview", "rust-src"])
            .exec(self.is_print_cmd)
            .unwrap();
    }

    pub fn workspace_root(&self) -> PathBuf {
        let meta = metadata(self.workdir());
        let pwd = meta.workspace_root.as_std_path();
        pwd.into()
    }

    pub fn out_dir_with_profile(&self, debug: bool) -> PathBuf {
        let meta = metadata(self.workdir());
        let pwd = meta.workspace_root.as_std_path();

        let target = &self.config_ref().compile.target;

        pwd.join("target")
            .join(target)
            .join(if debug { "debug" } else { "release" })
    }

    pub fn out_dir(&self) -> PathBuf {
        self.out_dir.clone().unwrap()
    }

    pub fn cargo_metadata(&self) -> Metadata {
        metadata(&self.workdir)
    }

    pub fn package_metadata(&self) -> Package {
        if let BuildSystem::Cargo(config) = &self.config_ref().compile.build {
            self.cargo_metadata()
                .packages
                .into_iter()
                .find(|one| one.name == config.package)
                .unwrap_or_else(|| panic!("Package {} not found!", config.package))
        } else {
            panic!("build system not supported")
        }
    }

    pub fn package_dependencies(&self) -> Vec<String> {
        let meta = self.package_metadata();
        meta.dependencies.into_iter().map(|dep| dep.name).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Arch {
    Aarch64,
    Riscv64,
    X86_64,
}

impl Default for Arch {
    fn default() -> Self {
        Self::Aarch64
    }
}

impl Arch {
    pub fn qemu_program(&self) -> String {
        format!("qemu-system-{}", self.qemu_arch())
    }
    pub fn qemu_arch(&self) -> String {
        match self {
            Arch::Aarch64 => "aarch64",
            Arch::Riscv64 => "riscv64",
            Arch::X86_64 => "x86_64",
        }
        .to_string()
    }

    pub fn from_target(target: &str) -> Result<Arch> {
        if target.contains("aarch64") {
            return Ok(Arch::Aarch64);
        }

        if target.contains("riscv64") {
            return Ok(Arch::Riscv64);
        }

        if target.contains("x86_64") {
            return Ok(Arch::X86_64);
        }

        Err(anyhow::anyhow!("Unsupportedtarget: {}", target))
    }
}
