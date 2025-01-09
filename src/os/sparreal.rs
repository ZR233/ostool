use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use colored::Colorize;

use crate::{
    config::{
        compile::{CargoBuild, Compile, LogLevel},
        qemu::Qemu,
        ProjectConfig,
    },
    os::{get_cargo_packages, get_rustup_targets},
    ui::shell_select,
};

use super::OsConfig;

pub struct Sparreal {
    workdir: PathBuf,
}

impl Sparreal {
    pub fn new_box(workdir: &Path) -> Option<Box<dyn OsConfig>> {
        let readme = fs::read_to_string(workdir.join("README.md")).ok()?;
        if readme.lines().next()?.contains("# 雀实操作系统 Sparreal") {
            Some(Box::new(Self {
                workdir: workdir.to_path_buf(),
            }))
        } else {
            None
        }
    }
}

impl OsConfig for Sparreal {
    fn new_config(&self) -> crate::config::ProjectConfig {
        println!("{}", "Sparreal OS detected.".green());
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
                cargo: Some(CargoBuild {
                    kernel_bin_name: None,
                    package,
                    log_level: LogLevel::Debug,
                    rust_flags:
                        "-C link-arg=-Tlink.x -C link-arg=-no-pie -C link-arg=-znostart-stop-gc"
                            .to_string(),
                    env: BTreeMap::new(),
                    features: Vec::new(),
                }),
                custom: None,
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
