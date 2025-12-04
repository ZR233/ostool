use std::path::PathBuf;

use anyhow::Context;

use crate::{
    build::{
        cargo_builder::CargoBuilder,
        config::{Cargo, Custom}
    },
    ctx::AppContext,
};

pub mod cargo_builder;
pub mod config;

pub enum CargoRunnerKind {
    Qemu {
        qemu_config: Option<PathBuf>,
        debug: bool,
        dtb_dump: bool,
    },
    Uboot {
        uboot_config: Option<PathBuf>,
    },
}

impl AppContext {
    pub async fn build_with_config(&mut self, config: &config::BuildConfig) -> anyhow::Result<()> {
        match &config.system {
            config::BuildSystem::Custom(custom) => self.build_custom(custom)?,
            config::BuildSystem::Cargo(cargo) => {
                self.cargo_build(cargo).await?;
            }
        }
        Ok(())
    }

    pub async fn build(&mut self, config_path: Option<PathBuf>) -> anyhow::Result<()> {
        let build_config = self.prepare_build_config(config_path, false).await?;
        println!("Build configuration: {:?}", build_config);
        self.build_with_config(&build_config).await
    }

    pub fn build_custom(&mut self, config: &Custom) -> anyhow::Result<()> {
        self.shell_run_cmd(&config.build_cmd)?;
        Ok(())
    }

    pub async fn cargo_build(&mut self, config: &Cargo) -> anyhow::Result<()> {
        cargo_builder::CargoBuilder::build_auto(self, config)
            .execute()
            .await
    }

    pub async fn cargo_run(
        &mut self,
        config: &Cargo,
        runner: &CargoRunnerKind,
    ) -> anyhow::Result<()> {
        let build_config_path = self.build_config_path.clone();

        let normalize = |dir: &PathBuf| -> anyhow::Result<PathBuf> {
            let bin_path = if dir.is_relative() {
                self.paths.manifest.join(dir)
            } else {
                dir.clone()
            };

            bin_path.canonicalize()
                .or_else(|_| {
                    if let Some(parent) = bin_path.parent() {
                        parent.canonicalize()
                            .map(|p| p.join(bin_path.file_name().unwrap()))
                    } else {
                        Ok(bin_path.clone())
                    }
                })
                .context("Failed to normalize path")
        };

        let build_dir = self.paths.config.build_dir
            .as_ref()
            .map(|d| normalize(d))
            .transpose()?;

        let bin_dir = self.paths.config.bin_dir
            .as_ref()
            .map(|d| normalize(d))
            .transpose()?;

        let mut builder = CargoBuilder::run(self, config, build_config_path);

        builder = builder.arg("--");

        if let Some(build_dir) = build_dir {
            builder = builder
                .arg("--build-dir")
                .arg(build_dir.display().to_string())
        }

        if let Some(bin_dir) = bin_dir {
            builder = builder
                .arg("--bin-dir")
                .arg(bin_dir.display().to_string())
        }

        match runner {
            CargoRunnerKind::Qemu {
                qemu_config,
                debug,
                dtb_dump,
            } => {
                if let Some(cfg) = qemu_config {
                    builder = builder
                        .arg("--config")
                        .arg(cfg.display().to_string());
                }

                builder = builder.debug(*debug);

                if *dtb_dump {
                    builder = builder.arg("--dtb-dump");
                }
                builder = builder.arg("qemu");
            }
            CargoRunnerKind::Uboot { uboot_config } => {
                if let Some(cfg) = uboot_config {
                    builder = builder
                        .arg("--config")
                        .arg(cfg.display().to_string());
                }
                builder = builder.arg("uboot");
            }
        }

        builder.execute().await
    }
}
