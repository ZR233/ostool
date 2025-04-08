use crate::{
    config::{ProjectConfig, qemu::Qemu},
    project::{Arch, Project},
    shell::Shell,
};
use colored::Colorize;
use object::{Architecture, Object};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{Step, UbootConfig};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    qemu: Option<Qemu>,
}

pub struct CargoTestPrepare {
    elf: String,
    uboot: bool,
}

impl CargoTestPrepare {
    pub fn new_boxed(elf: String, uboot: bool) -> Box<Self> {
        Box::new(Self { elf, uboot })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CargoToml {
    #[serde(rename = "test-qemu")]
    pub test_qemu: Option<BTreeMap<String, Qemu>>,
}

impl From<Architecture> for Arch {
    fn from(value: Architecture) -> Self {
        match value {
            Architecture::Aarch64 => Self::Aarch64,
            Architecture::X86_64 => Self::X86_64,
            Architecture::Riscv64 => Self::Riscv64,
            _ => panic!("{value:?} not support!"),
        }
    }
}

impl Step for CargoTestPrepare {
    fn run(&mut self, project: &mut Project) -> anyhow::Result<()> {
        let binary_data = fs::read(&self.elf).unwrap();
        let file = object::File::parse(&*binary_data).unwrap();
        let arch = file.architecture();
        project.arch = Some(arch.into());
        project.out_dir = PathBuf::from(&self.elf).parent().map(|p| p.to_path_buf());

        let target_dir = project.workspace_root().join("target");

        let test_name = Path::new(&self.elf).file_stem().unwrap();

        let mut config = ProjectConfig::new(project.arch.unwrap());
        config.qemu.machine = Some("virt".to_string());

        let bin_path = project
            .out_dir()
            .join(format!("{}.bin", test_name.to_string_lossy()));

        let elf_path = project.out_dir().join("test.elf");

        let target_elf = target_dir.join("kernel.elf");
        let _ = std::fs::remove_file(&target_elf);
        let _ = fs::copy(&self.elf, &target_elf);

        let _ = fs::remove_file(&elf_path);
        let _ = fs::copy(&self.elf, &elf_path);

        let _ = fs::remove_file(&bin_path);
        project
            .shell("rust-objcopy")
            .args(["--strip-all", "-O", "binary"])
            .arg(&self.elf)
            .arg(&bin_path)
            .exec(project.is_print_cmd)
            .unwrap();

        config.qemu = Qemu::default();

        let config_path = project.workdir().join("bare-test.toml");

        if config_path.exists() {
            let test_config: Config =
                toml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
            if let Some(q) = test_config.qemu.clone() {
                config.qemu = q;
            }
        } else {
            let test_config = Config {
                qemu: Some(config.qemu.clone()),
            };
            let s = toml::to_string_pretty(&test_config).unwrap();
            fs::write(&config_path, s).unwrap();
        }

        let config_user = project.workdir().join(".bare-test.toml");

        if self.uboot {
            let uboot_config;
            if !config_user.exists() {
                uboot_config = UbootConfig::config_by_select();
                let s = toml::to_string(&uboot_config).unwrap();
                fs::write(&config_user, s).unwrap();
            } else {
                uboot_config = match toml::from_str(&fs::read_to_string(&config_user).unwrap()) {
                    Ok(c) => c,
                    _ => {
                        let old = format!(
                            ".bare-test.toml.bk.{}",
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                        );
                        let old = project.workdir().join(old);
                        println!(
                            "{}",
                            format!("config error, generate new, save old to: {}", old.display())
                                .yellow()
                        );
                        let _ = fs::rename(&config_user, &old);

                        let config = UbootConfig::config_by_select();
                        let s = toml::to_string(&config).unwrap();
                        fs::write(&config_user, s).unwrap();
                        config
                    }
                };
            }
            config.uboot = Some(uboot_config);
        }

        let cargo_toml = project.workdir().join("Cargo.toml");
        let cargo_toml_content = fs::read_to_string(cargo_toml).unwrap();
        let cargo_toml_value: CargoToml = toml::from_str(&cargo_toml_content).unwrap();

        if let Some(arch_map) = cargo_toml_value.test_qemu {
            let k = project.arch.unwrap().qemu_arch();
            if let Some(qemu) = arch_map.get(&k) {
                config.qemu = qemu.clone();
            }
        }
        project.config = Some(config);
        project.kernel = Some(bin_path);

        Ok(())
    }
}
