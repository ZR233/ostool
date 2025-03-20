use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Compile {
    pub target: String,
    pub cargo: Option<CargoBuild>,
    pub custom: Option<CustomBuild>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CargoBuild {
    pub kernel_bin_name: Option<String>,
    pub package: String,
    pub log_level: LogLevel,
    pub rust_flags: String,
    pub features: Vec<String>,
    pub env: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomBuild {
    pub shell: Vec<Vec<String>>,
    pub elf: String,
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
