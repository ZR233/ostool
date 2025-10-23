use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use crate::data::menu::MenuRoot;

pub struct AppData {
    pub root: MenuRoot,
    pub current_key: String,
}

const DEFAULT_INIT_PATH: &str = ".project.toml";

fn default_schema_by_init(init_path: &Path) -> Option<PathBuf> {
  
}

impl AppData {
    pub fn new(init: Option<impl AsRef<Path>>, schema: Option<impl AsRef<Path>>) -> Self {
        let mut init_value_path = PathBuf::from(DEFAULT_INIT_PATH);

        let root = if let Some(path) = init {
            MenuRoot::load_from_file(path).unwrap_or_default()
        } else {
            MenuRoot::default()
        };
        let current_key = root.first_key().unwrap_or_default();
        AppData { root, current_key }
    }
}
