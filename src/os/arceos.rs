use std::{
    env::temp_dir,
    fs::{self},
    path::Path,
};

use colored::Colorize;

use crate::config::{
    compile::{Compile, CustomBuild},
    qemu::Qemu,
    ProjectConfig,
};

use super::OsConfig;

pub struct ArceOS {}

impl ArceOS {
    pub fn new_box(workdir: &Path) -> Option<Box<dyn OsConfig>> {
        let readme = fs::read_to_string(workdir.join("README.md")).ok()?;
        if readme.lines().next()?.contains("# ArceOS") {
            Some(Box::new(Self {}))
        } else {
            None
        }
    }

    fn _windows_shell(&self) -> Vec<String> {
        let tmp_dir = temp_dir().join("arceos_docker_cache");
        let tmp_git_dir = tmp_dir.join("git");
        let tmp_registry_dir = tmp_dir.join("registry");


        vec![
            "docker build -t arceos -f Dockerfile .".to_string(),
            format!("docker run --rm -it -v .:/arceos -v \"{}:/usr/local/cargo/git\" -v \"{}:/usr/local/cargo/registry\" -w /arceos arceos make A=examples/helloworld ARCH=aarch64", 
            tmp_git_dir.display(), tmp_registry_dir.display()),
            ]
    }
}

impl OsConfig for ArceOS {
    fn new_config(&self) -> ProjectConfig {
        println!("{}", "ArceOS detected.".green());

        let shell = if cfg!(windows) {
            self._windows_shell()
        } else {
            vec!["make A=examples/helloworld ARCH=aarch64".to_string()]
        };

        let elf = "examples/helloworld/helloworld_aarch64-qemu-virt.elf".to_string();

        ProjectConfig {
            compile: Compile {
                cargo: None,
                custom: Some(CustomBuild { shell, elf }),
                target: "aarch64-unknown-none-softfloat".to_string(),
            },
            qemu: Qemu {
                machine: Some("virt".to_string()),
                cpu: Some("cortex-a53".into()),
                graphic: false,
                args: "-smp 2".to_string(),
            },
            uboot: None,
        }
    }
}
