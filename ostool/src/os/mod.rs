use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use sparreal::Sparreal;

use crate::{
    config::{
        compile::{BuildSystem, CargoBuild, Compile, LogLevel},
        qemu::Qemu,
        ProjectConfig,
    },
    shell::{get_cargo_packages, get_rustup_targets},
    ui::shell_select,
};

pub mod sparreal;

pub trait OsConfig {
    fn new_config(&self) -> ProjectConfig;
}

pub fn new_config(workdir: &Path) -> ProjectConfig {
    let os = Sparreal::new_box(workdir).unwrap_or_else(|| Custom::new_box(workdir));
    os.new_config()
}

pub struct Custom {
    workdir: PathBuf,
}

impl Custom {
    fn new_box(workdir: &Path) -> Box<dyn OsConfig> {
        Box::new(Self {
            workdir: workdir.to_path_buf(),
        })
    }
}

impl OsConfig for Custom {
    fn new_config(&self) -> ProjectConfig {
        let targets = get_rustup_targets().unwrap();
        let select = shell_select("select target:", &targets);
        let target = targets[select].clone();

        let packages = get_cargo_packages(&self.workdir);
        let package = packages[shell_select("select package:", &packages)].clone();

        let mut cpu = None;

        let arch = target.split("-").next().unwrap().to_string();
        if arch == "aarch64" {
            cpu = Some("cortex-a53".to_string());
        }

        ProjectConfig {
            compile: Compile {
                target,
                build: BuildSystem::Cargo(CargoBuild {
                    kernel_bin_name: None,
                    package,
                    log_level: LogLevel::Debug,
                    rust_flags: String::new(),
                    env: BTreeMap::new(),
                    features: Vec::new(),
                    kernel_is_bin: true,
                }),
            },
            qemu: Qemu {
                machine: Some("virt".to_string()),
                cpu,
                graphic: false,
                args: String::new(),
            },
            uboot: None,
        }
    }
}
