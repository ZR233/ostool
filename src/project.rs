use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs,
    io::Write,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;

use crate::{
    config::{
        compile::{Compile, LogLevel},
        ProjectConfig,
    },
    ui::shell_select,
};

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
            config = Self::new_config(&workdir);
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

    fn new_config(workdir: &Path) -> ProjectConfig {
        let targets = get_rustup_targets().unwrap();

        let select = shell_select("select target:", &targets);
        let target = targets[select].clone();

        let packages = get_cargo_packages(workdir);
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

    pub fn output_dir(&self, debug: bool) -> PathBuf {
        let pwd = self.workdir.clone();

        let target = &self.config.compile.target;

        pwd.join("target")
            .join(target)
            .join(if debug { "debug" } else { "release" })
    }

    pub fn package_all_features(&self) -> Vec<String> {
        let meta = self.cargo_meta();
        let packages = meta["packages"].as_array().unwrap();
        let package = packages
            .iter()
            .find(|one| one["name"] == self.config.compile.package)
            .unwrap();
        package["features"]
            .as_array()
            .unwrap()
            .iter()
            .map(|one| one.as_str().unwrap().to_string())
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

fn get_rustup_targets() -> Result<Vec<String>> {
    let output = Command::new("rustup").args(["target", "list"]).output()?;

    let stdout = OsStr::from_bytes(&output.stdout);
    let targets: Vec<_> = stdout
        .to_str()
        .unwrap()
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(|line| line.trim_end_matches("(installed)").trim().to_string())
        .filter(|line| line.contains("-none"))
        .collect();

    Ok(targets)
}

fn get_cargo_packages(workdir: &Path) -> Vec<String> {
    let output = Command::new("cargo")
        .current_dir(workdir)
        .args(["metadata", "--format-version=1", "--no-deps"])
        .output()
        .unwrap();
    let stdout = OsStr::from_bytes(&output.stdout);
    let data = stdout.to_str().unwrap();

    let v: serde_json::Value = serde_json::from_str(data).unwrap();
    let packages = v["packages"].as_array().unwrap();

    packages
        .iter()
        .map(|p| p["name"].as_str().unwrap().to_string())
        .collect()
}
