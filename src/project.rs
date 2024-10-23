use std::{ffi::OsStr, fs, io::Write, os::unix::ffi::OsStrExt, path::PathBuf, process::Command};

use anyhow::Result;

use crate::{config::ProjectConfig, os::new_config};

pub struct Project {
    workdir: PathBuf,
    pub config: ProjectConfig,
    pub bin_path: Option<PathBuf>,
}

impl Project {
    pub fn new(workdir: PathBuf, config: Option<String>) -> Result<Self> {
        let config_path = config
            .map(PathBuf::from)
            .unwrap_or(workdir.join(".project.toml"));
        let config;
        if !fs::exists(&config_path)? {
            config = new_config(&workdir);
            let config_str = toml::to_string(&config).unwrap();
            let mut file = fs::File::create(&config_path).unwrap();
            file.write_all(config_str.as_bytes()).unwrap();
        } else {
            config = toml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
        }

        Ok(Self {
            workdir,
            config,
            bin_path: None,
        })
    }

    pub fn shell<S: AsRef<OsStr>>(&self, program: S) -> Command {
        let mut cmd = Command::new(program);
        cmd.current_dir(&self.workdir);
        cmd
    }

    pub fn output_dir(&self, debug: bool) -> PathBuf {
        let pwd = self.workdir.clone();

        let target = &self.config.compile.target;

        pwd.join("target")
            .join(target)
            .join(if debug { "debug" } else { "release" })
    }



    pub fn package_metadata(&self) -> serde_json::Value {
        let meta = self.cargo_meta();
        let packages = meta["packages"].as_array().unwrap();
        let package = packages
            .iter()
            .find(|one| one["name"] == self.config.compile.package)
            .unwrap();

        package.clone()
    }

    pub fn package_dependencies(&self) -> Vec<String> {
        let meta = self.package_metadata();

        meta["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|one| one["name"].as_str().unwrap().to_string())
            .collect()
    }

    fn cargo_meta(&self) -> serde_json::Value {
        let output = Command::new("cargo")
            .current_dir(&self.workdir)
            .args(["metadata", "--format-version=1", "--no-deps"])
            .output()
            .unwrap();
        let stdout = OsStr::from_bytes(&output.stdout);
        let data = stdout.to_str().unwrap();

        serde_json::from_str(data).unwrap()
    }
}
