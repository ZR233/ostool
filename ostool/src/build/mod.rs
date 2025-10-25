use std::path::{Path, PathBuf};

use anyhow::anyhow;
use jkconfig::data::app_data::default_schema_by_init;
use tokio::{fs, process::Command};

pub mod config;

pub async fn run_build(workdir: &Path, config_path: Option<PathBuf>) -> anyhow::Result<()> {
    // Build logic will be implemented here
    let config_path = match config_path {
        Some(path) => path,
        None => workdir.join(".config.toml"),
    };

    let schema_path = default_schema_by_init(&config_path);

    let schema = schemars::schema_for!(config::BuildConfig);
    let schema_json = serde_json::to_value(&schema)?;
    let schema_content = serde_json::to_string_pretty(&schema_json)?;
    fs::write(&schema_path, schema_content).await?;

    // 初始化AppData
    // let app_data = AppData::new(Some(&config_path), Some(schema_path))?;

    let config_content = fs::read_to_string(&config_path)
        .await
        .map_err(|_| anyhow!("can not open config file: {}", config_path.display()))?;

    let build_config: config::BuildConfig = toml::from_str(&config_content)?;

    println!("Build configuration: {:?}", build_config);

    match build_config.system {
        config::BuildSystem::Custom(custom) => {
            let mut ctx = CtxCustom {
                ctx: BuildContext {
                    workdir: workdir.to_path_buf(),
                },
                config: custom,
            };
            ctx.run().await?;
        }
        config::BuildSystem::Cargo(cargo) => todo!(),
    }

    Ok(())
}

struct BuildContext {
    workdir: PathBuf,
}

struct CtxCustom {
    ctx: BuildContext,
    config: config::Custom,
}

impl CtxCustom {
    async fn run(&mut self) -> anyhow::Result<()> {
        self.run_pre_build_cmd().await?;

        Ok(())
    }

    async fn run_pre_build_cmd(&self) -> anyhow::Result<()> {
        println!("Running pre-build command: {}", self.config.build_cmd);
        let mut cmd = self.config.build_cmd.split_whitespace();
        let mut command = Command::new(cmd.next().unwrap());
        for arg in cmd {
            command.arg(arg);
        }
        command.current_dir(&self.ctx.workdir);
        let status = command.status().await?;
        if !status.success() {
            anyhow::bail!("Pre-build command failed with status: {}", status);
        }

        Ok(())
    }
}
