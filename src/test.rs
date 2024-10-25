use crate::{
    project::{Arch, Project},
    shell::Shell,
};
use object::{Architecture, Object};
use std::{fs, path::PathBuf};

pub struct CargoTest {}

impl CargoTest {
    pub fn run(project: &mut Project, elf: String) {
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
            .exec()
            .unwrap();

        project.bin_path = Some(bin_path);
    }
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
