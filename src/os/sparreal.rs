use std::path::{Path, PathBuf};

use super::OsConfig;

pub struct Sparreal {
    _workdir: PathBuf,
}

impl Sparreal {
    pub fn new_box(_workdir: &Path) -> Option<Box<dyn OsConfig>> {
        None
    }
}

impl OsConfig for Sparreal {
    fn new_config(&self) -> crate::config::ProjectConfig {
        todo!()
    }
}
