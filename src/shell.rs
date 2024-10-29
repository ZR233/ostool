use std::{
    ffi::{OsStr, OsString},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::Result;
use cargo_metadata::Metadata;
use colored::Colorize;

pub trait Shell {
    fn exec(&mut self, is_print_cmd: bool) -> Result<()>;
}

impl Shell for Command {
    fn exec(&mut self, is_print_cmd: bool) -> Result<()> {
        if is_print_cmd {
            let mut cmd_str = self.get_program().to_string_lossy().to_string();

            for arg in self.get_args() {
                cmd_str += " ";
                cmd_str += arg.to_string_lossy().as_ref();
            }

            println!("{}", cmd_str.purple().bold());
        }

        let mut child = self.stdout(Stdio::piped()).spawn()?;

        let stdout = BufReader::new(child.stdout.take().unwrap());
        for line in stdout.lines() {
            let line = line.expect("Failed to read line");
            // 解析输出为UTF-8
            println!("{}", line);
        }

        let out = child.wait_with_output()?;

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
pub(crate) fn metadata(workdir: &Path) -> Metadata {
    let mut mainifest = workdir.join("Cargo.toml");
    mainifest = PathBuf::from(format!("{}", mainifest.display()).trim_start_matches("\\\\?\\"));
    println!("manifest: {}", mainifest.display());
    let mut cmd = cargo_metadata::MetadataCommand::new();
    cmd.manifest_path(mainifest);
    cmd.no_deps();
    cmd.exec().unwrap()
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
    let meta = metadata(workdir);

    meta.packages.into_iter().map(|p| p.name).collect()
}

pub(crate) fn check_porgram(program: &str) -> bool {
    Command::new(program).arg("--version").output().is_ok()
}
