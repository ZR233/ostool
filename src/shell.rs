use std::{
    ffi::OsString,
    process::{Command, Stdio},
};

use anyhow::Result;
use colored::Colorize;

pub trait Shell {
    fn exec(&mut self) -> Result<()>;
}

impl Shell for Command {
    fn exec(&mut self) -> Result<()> {
        let mut cmd_str = self.get_program().to_string_lossy().to_string();

        for arg in self.get_args() {
            cmd_str += " ";
            cmd_str += arg.to_string_lossy().as_ref();
        }

        println!("{}", cmd_str.purple().bold());

        let out = self
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait_with_output()?;

        if !out.status.success() {
            unsafe {
                return Err(anyhow::anyhow!(
                    "{}",
                    OsString::from_encoded_bytes_unchecked(out.stderr).to_string_lossy()
                ));
            }
        }

        Ok(())
    }
}
