use std::{path::PathBuf, process::Command};

use cargo_metadata::Metadata;

use crate::utils::ShellRunner;

pub struct AppContext {
    pub workdir: PathBuf,
    pub debug: bool,
}

impl AppContext {
    pub fn shell_run_cmd(&self, cmd: &str) -> anyhow::Result<()> {
        let mut parts = cmd.split_whitespace();
        let mut command = self.command(parts.next().unwrap());
        for arg in parts {
            command.arg(arg);
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
}
