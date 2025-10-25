use std::process::Command;

use anyhow::bail;
use colored::Colorize;

pub trait ShellRunner {
    fn run(&mut self) -> anyhow::Result<()>;
}

impl ShellRunner for Command {
    fn run(&mut self) -> anyhow::Result<()> {
        let mut cmd_str = self.get_program().to_string_lossy().to_string();

        for arg in self.get_args() {
            cmd_str += " ";
            cmd_str += arg.to_string_lossy().as_ref();
        }

        println!("{}", cmd_str.purple().bold());
        let status = self.status()?;
        if !status.success() {
            bail!("failed with status: {status}");
        }
        Ok(())
    }
}
