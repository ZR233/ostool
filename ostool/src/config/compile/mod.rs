use std::path::Path;

use serde::{Deserialize, Serialize};

mod cargo;
mod custom;

use crate::ui;
pub use cargo::CargoBuild;
pub use custom::CustomBuild;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Compile {
    pub target: String,
    pub build: BuildSystem,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BuildSystem {
    Cargo(CargoBuild),
    Custom(CustomBuild),
}

impl BuildSystem {
    pub fn new_by_ui(workdir: &Path) -> Self {
        let options = vec!["Cargo", "Custom"];
        let idx = ui::shell_select("请选择构建系统：", &options);

        match idx {
            0 => BuildSystem::Cargo(CargoBuild::new_by_ui(workdir)),
            1 => BuildSystem::Custom(CustomBuild::new_by_ui()),
            _ => panic!("Invalid input"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Debug
    }
}
