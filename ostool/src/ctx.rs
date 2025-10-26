use std::{path::PathBuf, process::Command};

use anyhow::anyhow;
use cargo_metadata::Metadata;
use object::{Architecture, Object};
use tokio::fs;

use crate::utils::ShellRunner;

#[derive(Default)]
pub struct AppContext {
    pub workdir: PathBuf,
    pub debug: bool,
    pub elf_path: Option<PathBuf>,
    pub bin_path: Option<PathBuf>,
    pub arch: Option<Architecture>,
}

impl AppContext {
    pub fn shell_run_cmd(&self, cmd: &str) -> anyhow::Result<()> {
        let mut parts = cmd.split_whitespace();
        let mut command = self.command(parts.next().unwrap());
        for arg in parts {
            command.arg(arg);
        }
        if let Some(elf) = &self.elf_path {
            command.env("KERNEL_ELF", elf.display().to_string());
        }

        command.run()?;
        Ok(())
    }

    pub fn command(&self, program: &str) -> Command {
        let mut command = Command::new(program);
        command.current_dir(&self.workdir);
        command
    }

    pub fn metadata(&self) -> anyhow::Result<Metadata> {
        let res = cargo_metadata::MetadataCommand::new()
            .current_dir(&self.workdir)
            .no_deps()
            .exec()?;
        Ok(res)
    }

    pub async fn set_elf_path(&mut self, path: PathBuf) {
        self.elf_path = Some(path.clone());
        let binary_data = match fs::read(path).await {
            Ok(data) => data,
            Err(e) => {
                println!("Failed to read ELF file: {e}");
                return;
            }
        };
        let file = match object::File::parse(binary_data.as_slice()) {
            Ok(f) => f,
            Err(e) => {
                println!("Failed to parse ELF file: {e}");
                return;
            }
        };
        self.arch = Some(file.architecture())
    }

    pub fn objcopy_output_bin(&mut self) -> anyhow::Result<PathBuf> {
        let elf_path = self.elf_path.as_ref().ok_or(anyhow!("elf not exist"))?;
        let bin_path = elf_path.with_extension("bin");
        let mut objcopy_cmd = self.command("rust-objcopy");
        objcopy_cmd.arg("--strip-all");
        objcopy_cmd.arg("-O");
        objcopy_cmd.arg("binary");
        objcopy_cmd.arg(elf_path);
        objcopy_cmd.arg(&bin_path);
        objcopy_cmd.run()?;
        self.bin_path = Some(bin_path.clone());
        Ok(bin_path)
    }
}
