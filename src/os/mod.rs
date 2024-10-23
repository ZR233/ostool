use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use arceos::ArceOS;
use sparreal::Sparreal;

use crate::{
    config::{
        compile::{Compile, LogLevel},
        ProjectConfig,
    },
    shell::{get_cargo_packages, get_rustup_targets},
    ui::shell_select,
};

pub mod arceos;
pub mod sparreal;

pub trait OsConfig {
    fn new_config(&self) -> ProjectConfig;
}

pub fn new_config(workdir: &Path) -> ProjectConfig {
    let os = ArceOS::new_box(workdir)
        .or_else(|| Sparreal::new_box(workdir))
        .unwrap_or_else(|| Custom::new_box(workdir));

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

        ProjectConfig {
            compile: Compile {
                target,
                kernel_bin_name: None,
                package,
                log_level: LogLevel::Debug,
                rust_flags: String::new(),
                custom_shell: None,
                env: BTreeMap::new(),
                features: Vec::new(),
            },
        }
    }
}
