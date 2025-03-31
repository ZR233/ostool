use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::{shell::get_cargo_packages, ui::shell_select};

use super::LogLevel;

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

impl CargoBuild {
    pub fn new_by_ui(workdir: &Path) -> Self {
        let packages = get_cargo_packages(workdir);
        let package = packages[shell_select("select package:", &packages)].clone();

        Self {
            kernel_bin_name: None,
            package,
            log_level: LogLevel::Info,
            rust_flags: "".to_string(),
            features: vec![],
            env: BTreeMap::new(),
            kernel_is_bin: false,
        }
    }
}
