use std::{
    collections::BTreeMap,
    fs::{self, read_dir},
    path::{Path, PathBuf},
};

use colored::Colorize;

use crate::{
    config::{
        compile::{Compile, LogLevel},
        qemu::Qemu,
        ProjectConfig,
    },
    shell::get_cargo_packages,
    ui::shell_select,
};

use super::OsConfig;

pub struct ArceOS {
    workdir: PathBuf,
}

impl ArceOS {
    pub fn new_box(workdir: &Path) -> Option<Box<dyn OsConfig>> {
        let readme = fs::read_to_string(workdir.join("README.md")).ok()?;
        if readme.lines().next()?.contains("# ArceOS") {
            Some(Box::new(Self {
                workdir: workdir.to_path_buf(),
            }))
        } else {
            None
        }
    }
}

impl OsConfig for ArceOS {
    fn new_config(&self) -> ProjectConfig {
        println!("{}", "ArceOS detected.".green());
        let mut platforms = Vec::new();
        for dir in read_dir(self.workdir.join("platforms")).unwrap() {
            let dir = dir.unwrap();
            let f = PathBuf::from(dir.file_name());
            if let Some(name) = f.file_name() {
                if !name.to_str().unwrap().ends_with(".toml") {
                    continue;
                }
                if let Some(name) = f.file_stem() {
                    platforms.push(name.to_str().unwrap().to_string());
                }
            }
        }
        let platform = platforms[shell_select("select platform", &platforms)].clone();
        let arch = platform.split("-").next().unwrap().to_string();
        let mut target = format!("{}-unknown-none", arch);
        let mut cpu = None;

        if arch == "aarch64" {
            cpu = Some("cortex-a53".to_string());
            target = "aarch64-unknown-none-softfloat".to_string();
        }

        let mut env = BTreeMap::new();
        env.insert("AX_PLATFORM".to_string(), platform.clone());

        let packages = get_cargo_packages(&self.workdir);
        let package = packages[shell_select("select package:", &packages)].clone();

        let ld_script_name = format!("linker_{}.lds", &platform);

        let rust_flags = format!(
            "-C link-arg=-T{} -C link-arg=-no-pie -C link-arg=-znostart-stop-gc",
            self.workdir
                .join("target")
                .join(&target)
                .join("release")
                .join(ld_script_name)
                .display()
        );

        ProjectConfig {
            compile: Compile {
                target,
                kernel_bin_name: None,
                package,
                log_level: LogLevel::Debug,
                rust_flags,
                custom_shell: None,
                env,
                features: ["axstd/log-level-info", "axstd/smp"]
                    .iter()
                    .map(|o| o.to_string())
                    .collect(),
            },
            qemu: Qemu {
                machine: Some("virt".to_string()),
                cpu,
                graphic: false,
                args: "-smp 4".to_string(),
            },
        }
    }
}
