use crate::{
    config::qemu::Qemu,
    project::{Arch, Project},
    shell::Shell,
};
use object::{Architecture, Object};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

pub struct CargoTest {}

impl CargoTest {
    pub fn run(project: &mut Project, elf: String) {
        project.config.qemu.machine = Some("virt".to_string());

        let binary_data = fs::read(&elf).unwrap();
        let file = object::File::parse(&*binary_data).unwrap();
        let arch = file.architecture();
        project.arch = arch.into();

        let mut bin_path = PathBuf::from(&elf);
        bin_path = bin_path.parent().unwrap().join("test.bin");

        let _ = fs::remove_file(&bin_path);
        project
            .shell("rust-objcopy")
            .args(["--strip-all", "-O", "binary"])
            .arg(&elf)
            .arg(&bin_path)
            .exec(project.is_print_cmd)
            .unwrap();

        project.config.qemu = Qemu::new_default(project.arch);
        let cargo_toml = project.workdir().join("Cargo.toml");
        let cargo_toml_content = fs::read_to_string(cargo_toml).unwrap();
        let cargo_toml_value: CargoToml = toml::from_str(&cargo_toml_content).unwrap();

        if let Some(test_qemu) = cargo_toml_value.test_qemu {
            project.config.qemu = test_qemu;
        }

        project.bin_path = Some(bin_path);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CargoToml {
    #[serde(rename = "test-qemu")]
    pub test_qemu: Option<Qemu>,
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
