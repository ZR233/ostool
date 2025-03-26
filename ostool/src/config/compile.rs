use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CargoBuild {
    pub kernel_bin_name: Option<String>,
    pub package: String,
    pub log_level: LogLevel,
    pub rust_flags: String,
    pub features: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub kernel_is_bin: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomBuild {
    pub shell: Vec<String>,
    pub elf: Option<String>,
    pub kernel: String,
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
