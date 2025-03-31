use std::{fs, path::Path};

use compile::{BuildSystem, Compile, CustomBuild};
use qemu::Qemu;
use serde::{Deserialize, Serialize};

use crate::{project::Arch, shell::get_rustup_targets, step::UbootConfig, ui::shell_select};

pub mod compile;
pub mod qemu;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectConfig {
    pub compile: Compile,
    pub qemu: Qemu,
    pub uboot: Option<UbootConfig>,
}

impl ProjectConfig {
    pub fn new(arch: Arch) -> Self {
        Self {
            compile: Compile {
                target: String::new(),
                build: BuildSystem::Custom(CustomBuild {
                    shell: vec![],
                    elf: None,
                    kernel: String::new(),
                }),
            },
            qemu: Qemu::new_default(arch),
            uboot: None,
        }
    }

    pub fn new_by_ui(workdir: &Path) -> Self {
        let targets = get_rustup_targets().unwrap();
        let select = shell_select("select target:", &targets);
        let target = targets[select].clone();
        let build = BuildSystem::new_by_ui(workdir);
        let arch = Arch::from_target(&target).unwrap();

        Self {
            compile: Compile { target, build },
            qemu: Qemu::new_default(arch),
            uboot: None,
        }
    }

    pub fn save(&self, path: &Path) {
        fs::write(path, toml::to_string(self).unwrap()).unwrap();
    }
}
