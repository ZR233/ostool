use std::path::PathBuf;

use anyhow::anyhow;
use jkconfig::data::app_data::default_schema_by_init;
use tokio::fs;

use crate::{ctx::AppContext, utils::ShellRunner};

pub mod config;

pub async fn run_build(
    ctx: AppContext,
    config_path: Option<PathBuf>,
) -> anyhow::Result<AppContext> {
    // Build logic will be implemented here
    let config_path = match config_path {
        Some(path) => path,
        None => ctx.workdir.join(".config.toml"),
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
            let ctx = CtxCustom {
                ctx,
                config: custom,
            };
            ctx.run().await
        }
        config::BuildSystem::Cargo(cargo) => {
            let ctx = CtxCargo { ctx, config: cargo };
            ctx.run().await
        }
    }
}

struct CtxCustom {
    ctx: AppContext,
    config: config::Custom,
}

impl CtxCustom {
    async fn run(self) -> anyhow::Result<AppContext> {
        self.ctx.shell_run_cmd(&self.config.build_cmd)?;
        Ok(self.ctx)
    }
}

struct CtxCargo {
    ctx: AppContext,
    config: config::Cargo,
}

impl CtxCargo {
    async fn run(mut self) -> anyhow::Result<AppContext> {
        for cmd in &self.config.pre_build_cmds {
            self.ctx.shell_run_cmd(cmd)?;
        }

        let mut features = self.config.features.clone();
        if let Some(log_level) = &self.log_level_feature() {
            features.push(log_level.to_string());
        }

        let mut cmd = self.ctx.command("cargo");
        cmd.arg("build");
        cmd.arg("-p");
        cmd.arg(&self.config.package);
        cmd.arg("--target");
        cmd.arg(&self.config.target);
        cmd.arg("-Z");
        cmd.arg("unstable-options");
        if !features.is_empty() {
            cmd.arg("--features");
            cmd.arg(features.join(","));
        }
        for arg in &self.config.args {
            cmd.arg(arg);
        }
        if !self.ctx.debug {
            cmd.arg("--release");
        }

        cmd.run()?;

        let elf_path = self
            .ctx
            .workdir
            .join("target")
            .join(&self.config.target)
            .join(if self.ctx.debug { "debug" } else { "release" })
            .join(&self.config.package);

        self.ctx.set_elf_path(elf_path.clone()).await;

        if self.config.to_bin {
            let bin_path = elf_path.with_extension("bin");
            let mut objcopy_cmd = self.ctx.command("rust-objcopy");
            objcopy_cmd.arg("--strip-all");
            objcopy_cmd.arg("-O");
            objcopy_cmd.arg("binary");
            objcopy_cmd.arg(elf_path);
            objcopy_cmd.arg(&bin_path);
            objcopy_cmd.run()?;
            self.ctx.bin_path = Some(bin_path);
        }

        for cmd in &self.config.post_build_cmds {
            self.ctx.shell_run_cmd(cmd)?;
        }

        Ok(self.ctx)
    }

    fn log_level_feature(&self) -> Option<String> {
        let level = self.config.log.clone()?;

        let meta = self.ctx.metadata().ok()?;
        let pkg = meta
            .packages
            .iter()
            .find(|p| p.name == self.config.package)?;
        let mut has_log = false;
        for dep in &pkg.dependencies {
            if dep.name == "log" {
                has_log = true;
                break;
            }
        }
        if has_log {
            Some(format!(
                "log/{}max_level_{}",
                if self.ctx.debug { "" } else { "release_" },
                format!("{:?}", level).to_lowercase()
            ))
        } else {
            None
        }
    }
}
