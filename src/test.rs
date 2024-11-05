use crate::{
    config::{compile::LogLevel, qemu::Qemu, ProjectConfig},
    project::{Arch, Project},
    shell::Shell,
    uboot::{self, UbootConfig},
};
use object::{Architecture, Object};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    qemu: Option<Qemu>,
}

pub struct CargoTest {}

impl CargoTest {
    pub fn run(project: &mut Project, elf: String, uboot: bool) {
        let binary_data = fs::read(&elf).unwrap();
        let file = object::File::parse(&*binary_data).unwrap();
        let arch = file.architecture();
        project.arch = Some(arch.into());
        project.out_dir = PathBuf::from(&elf).parent().map(|p| p.to_path_buf());

        let mut config = ProjectConfig::new(project.arch.unwrap());
        config.qemu.machine = Some("virt".to_string());
        config.compile.log_level = LogLevel::Error;

        let bin_path = project.out_dir().join("test.bin");

        let _ = fs::remove_file(&bin_path);
        project
            .shell("rust-objcopy")
            .args(["--strip-all", "-O", "binary"])
            .arg(&elf)
            .arg(&bin_path)
            .exec(project.is_print_cmd)
            .unwrap();

        config.qemu = Qemu::new_default(project.arch.unwrap());

        let config_path = project.workdir().join("bare-test.toml");

        if config_path.exists() {
            let test_config: Config =
                toml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
            if let Some(q) = test_config.qemu.clone() {
                config.qemu = q;
            }
        }

        let config_user = project.workdir().join(".bare-test.toml");

        if uboot {
            let uboot_config;
            if !config_user.exists() {
                uboot_config = UbootConfig::config_by_select();
                let s = toml::to_string(&uboot_config).unwrap();
                fs::write(&config_user, s).unwrap();
            } else {
                uboot_config = toml::from_str(&fs::read_to_string(&config_user).unwrap()).unwrap();
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
        project.bin_path = Some(bin_path);
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
