//! OS notifications feed the management projection; no periodic list polling.
#[cfg(target_os = "linux")]
mod linux {
    use crate::{AppState, admin_events::AdminEvents};
    use anyhow::Context;
    use futures_util::StreamExt;
    use notify::Watcher;
    use std::{path::Path, time::Duration};
    use tokio::{io::unix::AsyncFd, task::JoinHandle};

    pub struct Monitors {
        tasks: Vec<JoinHandle<()>>,
        _files: notify::RecommendedWatcher,
    }
    impl Drop for Monitors {
        fn drop(&mut self) {
            for task in &self.tasks {
                task.abort();
            }
        }
    }
    pub fn start(state: &AppState) -> anyhow::Result<Monitors> {
        let events = state.admin_events.clone();
        let mut files =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| match event {
                Ok(event)
                    if matches!(
                        event.kind,
                        notify::EventKind::Create(_)
                            | notify::EventKind::Remove(_)
                            | notify::EventKind::Modify(
                                notify::event::ModifyKind::Name(_)
                                    | notify::event::ModifyKind::Metadata(_)
                            )
                    ) && event.paths.iter().any(|path| {
                        path.starts_with("/dev/serial")
                            || path
                                .file_name()
                                .is_some_and(|name| name.to_string_lossy().starts_with("tty"))
                    }) =>
                {
                    events.invalidate(&["serial"])
                }
                Err(error) => log::warn!("serial device watcher: {error}"),
                _ => {}
            })?;
        // udev creates by-path/by-id links after the tty node. Observe that
        // second step too; otherwise serials without an SN stay unresolved.
        files.watch(Path::new("/dev"), notify::RecursiveMode::Recursive)?;
        let mut tasks = vec![];
        let events = state.admin_events.clone();
        let mut socket = netlink_sys::Socket::new(netlink_sys::protocols::NETLINK_ROUTE)?;
        // RTMGRP_LINK | RTMGRP_IPV4_IFADDR: inventory and IPv4 configuration.
        socket.bind(&netlink_sys::SocketAddr::new(0, 1 | 0x10))?;
        socket.set_non_blocking(true)?;
        let socket = AsyncFd::new(socket)?;
        tasks.push(tokio::spawn(async move {
            loop {
                let Ok(mut ready) = socket.readable().await else {
                    break;
                };
                let result = ready.try_io(|fd| fd.get_ref().recv(&mut Vec::<u8>::new(), 0));
                match result {
                    Ok(Ok(_)) => events.invalidate(&["network"]),
                    Ok(Err(e)) => {
                        log::warn!("network watcher: {e}");
                        break;
                    }
                    Err(_) => {}
                }
            }
        }));
        let events = state.admin_events.clone();
        let mut child = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::child())?;
        tasks.push(tokio::spawn(async move {
            while child.recv().await.is_some() {
                events.invalidate(&["virtual"]);
            }
        }));
        let events = state.admin_events.clone();
        tasks.push(tokio::spawn(async move {
            loop {
                if let Err(error) = systemd(events.clone()).await {
                    log::warn!("systemd event connection unavailable: {error:#}");
                }
                // Connection retry, not status polling. Keep the last projection.
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        }));
        Ok(Monitors {
            tasks,
            _files: files,
        })
    }
    async fn systemd(events: AdminEvents) -> anyhow::Result<()> {
        let connection = zbus::Connection::system().await?;
        let rule = zbus::MatchRule::builder()
            .msg_type(zbus::message::Type::Signal)
            .sender("org.freedesktop.systemd1")?
            .interface("org.freedesktop.DBus.Properties")?
            .member("PropertiesChanged")?
            .build();
        let mut messages = zbus::MessageStream::for_match_rule(rule, &connection, Some(64)).await?;
        let manager = zbus::Proxy::new(
            &connection,
            "org.freedesktop.systemd1",
            "/org/freedesktop/systemd1",
            "org.freedesktop.systemd1.Manager",
        )
        .await?;
        manager.call::<_, _, ()>("Subscribe", &()).await?;
        events.invalidate(&["tftp_status"]);
        while let Some(message) = messages.next().await {
            message.context("systemd event stream")?;
            events.invalidate(&["tftp_status"]);
        }
        Ok(())
    }
}
#[cfg(target_os = "linux")]
pub use linux::start;
