use std::{path::PathBuf, process::Command};

use anyhow::anyhow;
use cargo_metadata::Metadata;
use colored::Colorize;
use object::{Architecture, Object, ObjectSegment};
use tokio::fs;

use crate::utils::ShellRunner;

#[derive(Default, Clone)]
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
        println!(
            "{}",
            format!(
                "Converting ELF to BIN format...\r\n  elf: {}\r\n  bin: {}",
                elf_path.display(),
                bin_path.display()
            )
            .bold()
            .purple()
        );

        // Read ELF file
        let binary_data =
            std::fs::read(elf_path).map_err(|e| anyhow!("Failed to read ELF file: {}", e))?;

        // Parse ELF file
        let obj_file = object::File::parse(binary_data.as_slice())
            .map_err(|e| anyhow!("Failed to parse ELF file: {}", e))?;

        // Extract loadable segments and write to binary file
        let mut binary_output = Vec::new();
        let mut min_addr = u64::MAX;
        let mut max_addr = 0u64;

        // First pass: find memory range
        for segment in obj_file.segments() {
            // Only include loadable segments
            if segment.size() > 0 {
                let addr = segment.address();
                min_addr = min_addr.min(addr);
                max_addr = max_addr.max(addr + segment.size());
            }
        }

        if min_addr == u64::MAX {
            return Err(anyhow!("No loadable segments found in ELF file"));
        }

        // Allocate buffer for binary output
        let total_size = (max_addr - min_addr) as usize;
        binary_output.resize(total_size, 0u8);

        // Second pass: copy segment data
        for segment in obj_file.segments() {
            if let Ok(data) = segment.data()
                && !data.is_empty()
            {
                let addr = segment.address();
                let offset = (addr - min_addr) as usize;
                if offset + data.len() <= binary_output.len() {
                    binary_output[offset..offset + data.len()].copy_from_slice(data);
                }
            }
        }

        // Write binary file
        std::fs::write(&bin_path, binary_output)
            .map_err(|e| anyhow!("Failed to write binary file: {}", e))?;

        self.bin_path = Some(bin_path.clone());
        Ok(bin_path)
    }
}
