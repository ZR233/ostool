use std::{
    env,
    ffi::{OsStr, OsString},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::Result;
use cargo_metadata::Metadata;
use colored::Colorize;

use crate::env::get_extra_path;

pub trait Shell {
    fn exec_with_lines(
        &mut self,
        is_print_cmd: bool,
        on_line: impl Fn(&str) -> Result<()>,
    ) -> Result<()>;
    fn exec(&mut self, is_print_cmd: bool) -> Result<()> {
        self.exec_with_lines(is_print_cmd, |_| Ok(()))
    }
}

impl Shell for Command {
    fn exec_with_lines(
        &mut self,
        is_print_cmd: bool,
        on_line: impl Fn(&str) -> Result<()>,
    ) -> Result<()> {
        let mut paths = vec![];

        if let Some(path) = env::var_os("PATH") {
            paths = env::split_paths(&path).collect::<Vec<_>>();
        }

        for p in get_extra_path() {
            paths.push(p);
        }
        let new_path = env::join_paths(paths)?;

        self.env("PATH", new_path);

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
            on_line(&line)?;
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
    match Command::new(program).arg("--version").output() {
        Ok(out) => out.stderr.is_empty(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_porgram() {
        let is_installed = check_porgram("rust-objcopy");

        println!("is installed: {}", is_installed);
    }
}
