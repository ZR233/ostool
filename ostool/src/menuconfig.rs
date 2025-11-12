use anyhow::Result;
use clap::ValueEnum;
use log::info;
use tokio::fs;

use crate::ctx::AppContext;
use crate::run::qemu::QemuConfig;
use crate::run::uboot::UbootConfig;

#[derive(ValueEnum, Clone, Debug)]
pub enum MenuConfigMode {
    Qemu,
    Uboot,
}

pub struct MenuConfigHandler;

impl MenuConfigHandler {
    pub async fn handle_menuconfig(
        ctx: &mut AppContext,
        mode: Option<MenuConfigMode>,
    ) -> Result<()> {
        match mode {
            Some(MenuConfigMode::Qemu) => {
                Self::handle_qemu_config(ctx).await?;
            }
            Some(MenuConfigMode::Uboot) => {
                Self::handle_uboot_config(ctx).await?;
            }
            None => {
                // 默认模式：显示当前构建配置
                Self::handle_default_config(ctx).await?;
            }
        }
        Ok(())
    }

    async fn handle_default_config(ctx: &mut AppContext) -> Result<()> {
        ctx.perpare_build_config(None, true).await?;

        Ok(())
    }

    async fn handle_qemu_config(ctx: &mut AppContext) -> Result<()> {
        info!("配置 QEMU 运行参数");
        let config_path = ctx.workspace_folder.join(".qemu.toml");
        if config_path.exists() {
            println!("\n当前 U-Boot 配置文件: {}", config_path.display());
            // 这里可以读取并显示当前的 U-Boot 配置
        } else {
            println!("\n未找到 U-Boot 配置文件，将使用默认配置");
        }

        let config = jkconfig::run::<QemuConfig>(config_path, true, &[]).await?;

        if let Some(c) = config {
            fs::write(
                ctx.value_replace_with_var(ctx.workspace_folder.join(".qemu.toml")),
                toml::to_string_pretty(&c)?,
            )
            .await?;
            println!("\nQEMU 配置已保存到 .qemu.toml");
        } else {
            println!("\n未更改 QEMU 配置");
        }

        Ok(())
    }

    async fn handle_uboot_config(ctx: &mut AppContext) -> Result<()> {
        info!("配置 U-Boot 运行参数");

        println!("=== U-Boot 配置模式 ===");

        // 检查是否存在 U-Boot 配置文件
        let uboot_config_path = ctx.workspace_folder.join(".uboot.toml");
        if uboot_config_path.exists() {
            println!("\n当前 U-Boot 配置文件: {}", uboot_config_path.display());
            // 这里可以读取并显示当前的 U-Boot 配置
        } else {
            println!("\n未找到 U-Boot 配置文件，将使用默认配置");
        }
        let config = jkconfig::run::<UbootConfig>(uboot_config_path, true, &[]).await?;
        if let Some(c) = config {
            fs::write(
                ctx.value_replace_with_var(ctx.workspace_folder.join(".uboot.toml")),
                toml::to_string_pretty(&c)?,
            )
            .await?;
            println!("\nU-Boot 配置已保存到 .uboot.toml");
        } else {
            println!("\n未更改 U-Boot 配置");
        }

        Ok(())
    }
}
