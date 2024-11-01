use std::{
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use cargo_metadata::{Metadata, Package};

use crate::{
    config::ProjectConfig,
    os::new_config,
    shell::{check_porgram, metadata, Shell},
};

pub struct Project {
    workdir: PathBuf,
    pub config: Option<ProjectConfig>,
    pub arch: Arch,
    pub bin_path: Option<PathBuf>,
    pub is_print_cmd: bool,
}

impl Project {
    pub fn new(workdir: PathBuf) -> Self {
        Self {
            workdir,
            config: None,
            bin_path: None,
            arch: Arch::Aarch64,
            is_print_cmd: true,
        }
    }

    pub fn config_by_file(&mut self, config_path: Option<String>) -> Result<()> {
        let meta = metadata(self.workdir());
        let config_path = config_path
            .map(PathBuf::from)
            .unwrap_or(meta.workspace_root.as_std_path().join(".project.toml"));

        let config;
        if !fs::exists(&config_path)? {
            config = new_config(self.workdir());
            let config_str = toml::to_string(&config).unwrap();
            let mut file = fs::File::create(&config_path).unwrap();
            file.write_all(config_str.as_bytes()).unwrap();
        } else {
            config = toml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
        }
        self.arch = Arch::from_target(&config.compile.target).unwrap();
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
        if !check_porgram("rust-objcopy") {
            self.install_deps();
        }
    }

    fn install_deps(&self) {
        self.shell("cargo")
            .args(["install", "cargo-binutils"])
            .exec(self.is_print_cmd)
            .unwrap();
        self.shell("rustup")
            .args(["component", "add", "llvm-tools-preview", "rust-src"])
            .exec(self.is_print_cmd)
            .unwrap();
    }

    pub fn output_dir(&self, debug: bool) -> PathBuf {
        let pwd = self.workdir.clone();

        let target = &self.config_ref().compile.target;

        pwd.join("target")
            .join(target)
            .join(if debug { "debug" } else { "release" })
    }

    pub fn cargo_metadata(&self) -> Metadata {
        metadata(&self.workdir)
    }

    pub fn package_metadata(&self) -> Package {
        self.cargo_metadata()
            .packages
            .into_iter()
            .find(|one| one.name == self.config_ref().compile.package)
            .unwrap_or_else(|| panic!("Package {} not found!", self.config_ref().compile.package))
    }

    pub fn package_dependencies(&self) -> Vec<String> {
        let meta = self.package_metadata();
        meta.dependencies.into_iter().map(|dep| dep.name).collect()
    }
}

#[derive(Debug, Clone, Copy)]
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

    fn from_target(target: &str) -> Result<Arch> {
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
