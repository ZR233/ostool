use std::{
    ffi::{OsStr, OsString},
    path::Path,
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
            .stdin(Stdio::inherit())
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

pub(crate) fn get_rustup_targets() -> Result<Vec<String>> {
    let output = Command::new("rustup").args(["target", "list"]).output()?;

    let stdout = unsafe { OsStr::from_encoded_bytes_unchecked(&output.stdout) };
    let targets: Vec<_> = stdout
        .to_str()
        .unwrap()
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(|line| line.trim_end_matches("(installed)").trim().to_string())
        .filter(|line| line.contains("-none"))
        .collect();

    Ok(targets)
}

pub(crate) fn get_cargo_packages(workdir: &Path) -> Vec<String> {
    let output = Command::new("cargo")
        .current_dir(workdir)
        .args(["metadata", "--format-version=1", "--no-deps"])
        .output()
        .unwrap();
    let stdout = unsafe { OsStr::from_encoded_bytes_unchecked(&output.stdout) };
    let data = stdout.to_str().unwrap();

    let v: serde_json::Value = serde_json::from_str(data).unwrap();
    let packages = v["packages"].as_array().unwrap();

    packages
        .iter()
        .map(|p| p["name"].as_str().unwrap().to_string())
        .collect()
}
