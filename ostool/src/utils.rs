use std::process::Command;

use anyhow::bail;
use colored::Colorize;

pub trait ShellRunner {
    fn print_cmd(&self);
    fn run(&mut self) -> anyhow::Result<()>;
}

impl ShellRunner for Command {
    fn print_cmd(&self) {
        let mut cmd_str = self.get_program().to_string_lossy().to_string();

        for arg in self.get_args() {
            cmd_str += " ";
            cmd_str += arg.to_string_lossy().as_ref();
        }

        println!("{}", cmd_str.purple().bold());
    }

    fn run(&mut self) -> anyhow::Result<()> {
        self.print_cmd();
        let status = self.status()?;
        if !status.success() {
            bail!("failed with status: {status}");
        }
        Ok(())
    }
}
