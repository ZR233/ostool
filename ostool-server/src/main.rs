use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::Context;
use clap::{Parser, Subcommand};
use log::info;
use ostool_server::{
    ServerConfig, build_app_state, build_router,
    loader::start_udp_discovery,
    tftp::service::{BuiltinTftpManager, SystemTftpdHpaManager, TftpManager},
    virtual_lab::{VirtualLabAction, run_virtual_lab},
};

#[derive(Parser, Debug)]
#[command(version, about = "ostool board server")]
struct Cli {
    #[arg(short, long, default_value = ".ostool-server.toml")]
    config: PathBuf,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Manage the isolated QEMU network namespace and TAP pool.
    VirtualLab {
        #[command(subcommand)]
        action: VirtualLabCommand,
    },
}

#[derive(Subcommand, Debug)]
enum VirtualLabCommand {
    Up,
    Status,
    Down,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let config = ServerConfig::load_or_create(&cli.config).await?;
    if let Some(Command::VirtualLab { action }) = cli.command {
        let action = match action {
            VirtualLabCommand::Up => VirtualLabAction::Up,
            VirtualLabCommand::Status => VirtualLabAction::Status,
            VirtualLabCommand::Down => VirtualLabAction::Down,
        };
        return run_virtual_lab(&config.virtual_qemu, action).await;
    }
    let tftp_manager: Arc<dyn TftpManager> = match &config.tftp {
        ostool_server::TftpConfig::Builtin(cfg) => Arc::new(BuiltinTftpManager::new(cfg.clone())),
        ostool_server::TftpConfig::SystemTftpdHpa(cfg) => {
            Arc::new(SystemTftpdHpaManager::new(cfg.clone()))
        }
    };

    let state = build_app_state(cli.config.clone(), config, tftp_manager.clone()).await?;
    state.ensure_data_dirs().await?;
    for (board_id, err) in state.power_off_all_boards_on_startup().await {
        log::warn!(
            "failed to power off board `{board_id}` during server startup; marking it disabled for this process: {err}"
        );
    }
    let discovery_task = start_udp_discovery(state.clone()).await?;
    tftp_manager.start_if_needed().await?;
    if let ostool_server::TftpConfig::SystemTftpdHpa(cfg) = &state.config.read().await.tftp
        && cfg.reconcile_on_start
    {
        tftp_manager.reconcile().await?;
    }
    let gc_state = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if let Err(err) = gc_state.cleanup_expired_sessions().await {
                log::warn!("failed to cleanup expired sessions: {err:#}");
            }
        }
    });

    #[cfg(target_os = "linux")]
    let _admin_monitors = ostool_server::admin_monitor::start(&state)?;
    let app = build_router(state.clone());
    let listen_addr = state.config.read().await.listen_addr;
    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .with_context(|| format!("failed to bind {listen_addr}"))?;
    info!("ostoold listening on {listen_addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    if let Some(task) = discovery_task {
        task.abort();
        let _ = task.await;
    }
    state.virtual_boards.shutdown().await;
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut terminate = match signal(SignalKind::terminate()) {
            Ok(signal) => signal,
            Err(error) => {
                log::warn!("failed to install SIGTERM handler: {error}");
                let _ = tokio::signal::ctrl_c().await;
                return;
            }
        };
        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                if let Err(error) = result {
                    log::warn!("failed to wait for Ctrl-C: {error}");
                }
            }
            _ = terminate.recv() => {}
        }
    }

    #[cfg(not(unix))]
    if let Err(error) = tokio::signal::ctrl_c().await {
        log::warn!("failed to wait for Ctrl-C: {error}");
    }
}
