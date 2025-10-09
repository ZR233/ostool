use crate::{
    config::qemu::Qemu,
    project::{Arch, Project},
    shell::Shell,
};
use object::{Architecture, Object};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use super::{Step, UbootConfig};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    qemu: Option<Qemu>,
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

/// Unified test preparation step that handles both cargo test and manual testing
pub struct TestPrepare {
    pub elf: Option<String>,
    pub uboot: bool,
}

impl TestPrepare {
    pub fn new_boxed(elf: Option<String>, uboot: bool, _is_cargo_test: bool) -> Box<Self> {
        Box::new(Self {
            elf,
            uboot,
        })
    }

    fn prepare_cargo_test(&mut self, project: &mut Project, elf_path: &str) -> anyhow::Result<()> {
        let binary_data = fs::read(elf_path)?;
        let file = object::File::parse(&*binary_data)?;
        let arch = file.architecture();
        project.arch = Some(arch.into());
        project.out_dir = PathBuf::from(elf_path).parent().map(|p| p.to_path_buf());

        let target_dir = project.workspace_root().join("target");
        let test_name = Path::new(elf_path).file_stem().unwrap();

        let bin_path = project
            .out_dir()
            .join(format!("{}.bin", test_name.to_string_lossy()));

        let elf_path_for_kernel = project.out_dir().join("test.elf");
        let target_elf = target_dir.join("kernel.elf");

        // Copy ELF files
        let _ = std::fs::remove_file(&target_elf);
        let _ = fs::copy(elf_path, &target_elf);

        let _ = fs::remove_file(&elf_path_for_kernel);
        let _ = fs::copy(elf_path, &elf_path_for_kernel);

        // Convert to binary if needed
        let _ = fs::remove_file(&bin_path);
        project
            .shell("rust-objcopy")
            .args(["--strip-all", "-O", "binary"])
            .arg(elf_path)
            .arg(&bin_path)
            .exec(project.is_print_cmd)?;

        // Setup kernel path
        let kernel = if matches!(project.arch, Some(Arch::X86_64)) {
            elf_path_for_kernel.clone()
        } else {
            bin_path.clone()
        };

        project.kernel = Some(kernel);

        // Handle QEMU configuration from cargo.toml or test configs
        self.setup_cargo_qemu_config(project)?;

        Ok(())
    }

    fn setup_cargo_qemu_config(&mut self, project: &mut Project) -> anyhow::Result<()> {
        // Try to read QEMU config from Cargo.toml
        let cargo_toml = project.workdir().join("Cargo.toml");
        if cargo_toml.exists() {
            let cargo_toml_content = fs::read_to_string(&cargo_toml)?;
            if let Ok(cargo_toml_value) = toml::from_str::<CargoToml>(&cargo_toml_content) {
                if let Some(arch_map) = cargo_toml_value.test_qemu {
                    let k = project.arch.unwrap().qemu_arch();
                    if let Some(qemu) = arch_map.get(&k) {
                        if let Some(ref mut config) = project.config {
                            config.qemu = qemu.clone();
                        }
                    }
                }
            }
        }

        // Try to read from bare-test.toml for legacy compatibility
        let config_path = project.workdir().join("bare-test.toml");
        if config_path.exists() {
            let test_config: Config = toml::from_str(&fs::read_to_string(&config_path)?)?;
            if let Some(q) = test_config.qemu {
                if let Some(ref mut config) = project.config {
                    config.qemu = q;
                }
            }
        }

        // Handle U-Boot config for cargo tests
        if self.uboot {
            let config_user = project.workdir().join(".bare-test.toml");
            if config_user.exists() {
                let uboot_config: UbootConfig = toml::from_str(&fs::read_to_string(&config_user)?)?;
                if let Some(ref mut config) = project.config {
                    config.uboot = Some(uboot_config);
                }
            }
        }

        Ok(())
    }
}

impl Step for TestPrepare {
    fn run(&mut self, project: &mut Project) -> anyhow::Result<()> {
        if let Some(elf_path) = self.elf.clone() {
            self.prepare_cargo_test(project, &elf_path)?;
        } else {
            // For non-cargo tests, ensure project has proper configuration
            if project.config.is_none() {
                return Err(anyhow::anyhow!("Project configuration not loaded"));
            }
        }
        Ok(())
    }
}