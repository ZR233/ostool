use std::{
    ffi::OsStr,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use crate::ctx::AppContext;
use anyhow::bail;
use colored::Colorize;
use jkconfig::data::app_data::default_schema_by_init;
use schemars::JsonSchema;
use tokio::fs;

pub struct Command {
    inner: std::process::Command,
    value_replace: Box<dyn Fn(&OsStr) -> String>,
}

impl Deref for Command {
    type Target = std::process::Command;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Command {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Command {
    pub fn new<S>(
        program: S,
        workdir: &Path,
        value_replace: impl Fn(&OsStr) -> String + 'static,
    ) -> Command
    where
        S: AsRef<OsStr>,
    {
        let mut cmd = std::process::Command::new(program);
        cmd.current_dir(workdir);
        cmd.env("WORKSPACE_FOLDER", workdir.display().to_string());

        Self {
            inner: cmd,
            value_replace: Box::new(value_replace),
        }
    }

    pub fn print_cmd(&self) {
        let mut cmd_str = self.get_program().to_string_lossy().to_string();

        for arg in self.get_args() {
            cmd_str += " ";
            cmd_str += arg.to_string_lossy().as_ref();
        }

        println!("{}", cmd_str.purple().bold());
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.print_cmd();
        let status = self.status()?;
        if !status.success() {
            bail!("failed with status: {status}");
        }
        Ok(())
    }

    pub fn arg<S>(&mut self, arg: S) -> &mut Command
    where
        S: AsRef<OsStr>,
    {
        let value = (self.value_replace)(arg.as_ref());
        self.inner.arg(value);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Command
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for arg in args {
            self.arg(arg.as_ref());
        }
        self
    }

    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Command
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        let value = (self.value_replace)(val.as_ref());
        self.inner.env(key, value);
        self
    }
}

pub async fn prepare_config<C: JsonSchema>(
    ctx: &mut AppContext,
    config_path: Option<PathBuf>,
    config_name: &str,
) -> anyhow::Result<String> {
    // Implementation here
    // Build logic will be implemented here
    let config_path = match config_path {
        Some(path) => path,
        None => ctx.manifest_dir.join(config_name),
    };
    ctx.build_config_path = Some(config_path.clone());

    let schema_path = default_schema_by_init(&config_path);

    let schema = schemars::schema_for!(C);
    let schema_json = serde_json::to_value(&schema)?;
    let schema_content = serde_json::to_string_pretty(&schema_json)?;
    fs::write(&schema_path, schema_content).await?;

    // 初始化AppData
    // let app_data = AppData::new(Some(&config_path), Some(schema_path))?;

    let config_content = fs::read_to_string(&config_path)
        .await
        .map_err(|_| anyhow!("can not open config file: {}", config_path.display()))?;

    Ok(config_content)
}
