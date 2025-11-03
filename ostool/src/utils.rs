use std::{path::PathBuf, process::Command};

use crate::ctx::AppContext;
use anyhow::bail;
use colored::Colorize;
use jkconfig::data::app_data::default_schema_by_init;
use schemars::JsonSchema;
use tokio::fs;

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

pub async fn prepare_config<C: JsonSchema>(
    ctx: &mut AppContext,
    config_path: Option<PathBuf>,
    config_name: &str,
) -> anyhow::Result<String> {
    // Implementation here
    // Build logic will be implemented here
    let config_path = match config_path {
        Some(path) => path,
        None => ctx.workdir.join(config_name),
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
