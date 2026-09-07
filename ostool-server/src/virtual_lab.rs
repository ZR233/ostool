use std::{fs, os::unix::ffi::OsStrExt as _, path::PathBuf, time::Duration};

use anyhow::{Context, bail};
use tokio::process::Command;

use crate::config::VirtualQemuConfig;

const HOST_VETH: &str = "ostool-host0";
const NAMESPACE_VETH: &str = "ostool-ns0";
const SERVER_CIDR: &str = "10.77.0.1/24";
const DHCP_CIDR: &str = "10.77.0.254/24";
const DHCP_RANGE: &str = "10.77.0.100,10.77.0.200,255.255.255.0,12h";

#[derive(Debug, Clone, Copy)]
pub enum VirtualLabAction {
    Up,
    Status,
    Down,
}

pub async fn run_virtual_lab(
    config: &VirtualQemuConfig,
    action: VirtualLabAction,
) -> anyhow::Result<()> {
    let _lock = LabLock::acquire(config).await?;
    match action {
        VirtualLabAction::Up => up(config).await,
        VirtualLabAction::Status => status(config).await,
        VirtualLabAction::Down => down(config).await,
    }
}

async fn up(config: &VirtualQemuConfig) -> anyhow::Result<()> {
    if !config.enabled {
        bail!("virtual_qemu.enabled must be true before starting the virtual lab");
    }
    if namespace_exists(&config.network_namespace).await? {
        if status(config).await.is_ok() {
            return Ok(());
        }
        down(config).await?;
    }

    let result = create_lab(config).await;
    if result.is_err() {
        let _ = down(config).await;
    }
    result?;
    status(config).await
}

async fn create_lab(config: &VirtualQemuConfig) -> anyhow::Result<()> {
    let username = invoking_username().await?;
    let groupname = command_output("id", &["-gn", &username])
        .await
        .context("failed to resolve the virtual lab owner group")?;
    tokio::fs::create_dir_all(lab_runtime_dir(config)).await?;

    run("ip", &["netns", "add", &config.network_namespace]).await?;
    run(
        "ip",
        &[
            "link",
            "add",
            HOST_VETH,
            "type",
            "veth",
            "peer",
            "name",
            NAMESPACE_VETH,
        ],
    )
    .await?;
    run(
        "ip",
        &[
            "link",
            "set",
            NAMESPACE_VETH,
            "netns",
            &config.network_namespace,
        ],
    )
    .await?;
    run("ip", &["addr", "add", SERVER_CIDR, "dev", HOST_VETH]).await?;
    run("ip", &["link", "set", HOST_VETH, "up"]).await?;

    run_in_namespace(config, &["link", "add", &config.bridge, "type", "bridge"]).await?;
    run_in_namespace(config, &["addr", "add", DHCP_CIDR, "dev", &config.bridge]).await?;
    run_in_namespace(config, &["link", "set", &config.bridge, "up"]).await?;
    run_in_namespace(
        config,
        &["link", "set", NAMESPACE_VETH, "master", &config.bridge],
    )
    .await?;
    run_in_namespace(config, &["link", "set", NAMESPACE_VETH, "up"]).await?;

    for tap in &config.tap_pool {
        run_in_namespace(
            config,
            &[
                "tuntap", "add", "dev", tap, "mode", "tap", "user", &username,
            ],
        )
        .await?;
        run_in_namespace(config, &["link", "set", tap, "master", &config.bridge]).await?;
        run_in_namespace(config, &["link", "set", tap, "up"]).await?;
    }

    let pid_file = dnsmasq_pid_file(config);
    let lease_file = dnsmasq_lease_file(config);
    let status = Command::new("ip")
        .args(["netns", "exec", &config.network_namespace, "dnsmasq"])
        .arg("--conf-file=")
        .arg(format!("--interface={}", config.bridge))
        .arg("--bind-interfaces")
        .arg(format!("--user={username}"))
        .arg(format!("--group={groupname}"))
        .arg("--listen-address=10.77.0.254")
        .arg(format!("--dhcp-range={DHCP_RANGE}"))
        .arg("--dhcp-option=3,10.77.0.1")
        .arg("--dhcp-option=6,10.77.0.254")
        .arg(format!("--pid-file={}", pid_file.display()))
        .arg(format!("--dhcp-leasefile={}", lease_file.display()))
        .status()
        .await
        .context("failed to execute dnsmasq")?;
    if !status.success() {
        bail!("dnsmasq exited with {status}");
    }
    for _ in 0..20 {
        if dnsmasq_process(config).await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    dnsmasq_process(config).await?;
    Ok(())
}

async fn invoking_username() -> anyhow::Result<String> {
    if let Some(username) = std::env::var_os("SUDO_USER")
        .and_then(|username| username.into_string().ok())
        .filter(|username| !username.trim().is_empty() && username != "root")
    {
        command_output("id", &["-u", &username])
            .await
            .context("SUDO_USER does not name a valid account")?;
        return Ok(username);
    }
    command_output("id", &["-un"]).await
}

async fn status(config: &VirtualQemuConfig) -> anyhow::Result<()> {
    if !namespace_exists(&config.network_namespace).await? {
        bail!("virtual lab `{}` is down", config.network_namespace);
    }
    run("ip", &["link", "show", HOST_VETH]).await?;
    run_in_namespace(config, &["link", "show", &config.bridge]).await?;
    for tap in &config.tap_pool {
        run_in_namespace(config, &["link", "show", tap]).await?;
    }
    dnsmasq_process(config).await?;
    println!(
        "virtual lab {} is up; server=10.77.0.1, bridge={}, taps={}",
        config.network_namespace,
        config.bridge,
        config.tap_pool.join(",")
    );
    Ok(())
}

async fn down(config: &VirtualQemuConfig) -> anyhow::Result<()> {
    let pid_file = dnsmasq_pid_file(config);
    if let Ok(pid) = dnsmasq_process(config).await {
        let _ = Command::new("kill").arg(pid.to_string()).status().await;
    }
    match tokio::fs::remove_file(&pid_file).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    match tokio::fs::remove_file(dnsmasq_lease_file(config)).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    if namespace_exists(&config.network_namespace).await? {
        run("ip", &["netns", "delete", &config.network_namespace]).await?;
    }
    if link_exists(HOST_VETH).await? {
        run("ip", &["link", "delete", HOST_VETH]).await?;
    }
    println!("virtual lab {} is down", config.network_namespace);
    Ok(())
}

async fn namespace_exists(namespace: &str) -> anyhow::Result<bool> {
    Ok(Command::new("ip")
        .args(["netns", "exec", namespace, "true"])
        .status()
        .await
        .context("failed to execute ip")?
        .success())
}

async fn link_exists(link: &str) -> anyhow::Result<bool> {
    Ok(Command::new("ip")
        .args(["link", "show", link])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .context("failed to execute ip")?
        .success())
}

async fn run_in_namespace(config: &VirtualQemuConfig, args: &[&str]) -> anyhow::Result<()> {
    let mut full_args = vec!["netns", "exec", config.network_namespace.as_str(), "ip"];
    full_args.extend_from_slice(args);
    run("ip", &full_args).await
}

async fn run(program: &str, args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new(program)
        .args(args)
        .status()
        .await
        .with_context(|| format!("failed to execute {program}"))?;
    if !status.success() {
        bail!("{program} {} exited with {status}", args.join(" "));
    }
    Ok(())
}

async fn command_output(program: &str, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new(program).args(args).output().await?;
    if !output.status.success() {
        bail!("{program} {} exited with {}", args.join(" "), output.status);
    }
    String::from_utf8(output.stdout)
        .context("command output was not UTF-8")
        .map(|output| output.trim().to_string())
}

fn lab_runtime_dir(config: &VirtualQemuConfig) -> PathBuf {
    config.runtime_dir.join("lab")
}

fn dnsmasq_pid_file(config: &VirtualQemuConfig) -> PathBuf {
    lab_runtime_dir(config).join("dnsmasq.pid")
}

fn dnsmasq_lease_file(config: &VirtualQemuConfig) -> PathBuf {
    lab_runtime_dir(config).join("dnsmasq.leases")
}

async fn dnsmasq_process(config: &VirtualQemuConfig) -> anyhow::Result<u32> {
    let pid_file = dnsmasq_pid_file(config);
    let pid = tokio::fs::read_to_string(&pid_file)
        .await
        .with_context(|| format!("failed to read {}", pid_file.display()))?
        .trim()
        .parse::<u32>()
        .context("dnsmasq pid file did not contain a process ID")?;
    let cmdline = tokio::fs::read(format!("/proc/{pid}/cmdline"))
        .await
        .context("dnsmasq process is not running")?;
    if !is_expected_dnsmasq_cmdline(&cmdline, &pid_file) {
        bail!("process {pid} from the virtual lab pid file is not its dnsmasq");
    }
    let namespace_pids =
        command_output("ip", &["netns", "pids", config.network_namespace.as_str()]).await?;
    if !namespace_pids
        .lines()
        .filter_map(|line| line.trim().parse::<u32>().ok())
        .any(|namespace_pid| namespace_pid == pid)
    {
        bail!(
            "dnsmasq process {pid} is not in network namespace `{}`",
            config.network_namespace
        );
    }
    Ok(pid)
}

fn is_expected_dnsmasq_cmdline(cmdline: &[u8], pid_file: &std::path::Path) -> bool {
    let expected_pid_file = format!("--pid-file={}", pid_file.display());
    let mut arguments = cmdline.split(|byte| *byte == 0);
    let executable_is_dnsmasq = arguments
        .next()
        .and_then(|argument| {
            std::path::Path::new(std::ffi::OsStr::from_bytes(argument)).file_name()
        })
        .is_some_and(|name| name == "dnsmasq");
    executable_is_dnsmasq && arguments.any(|argument| argument == expected_pid_file.as_bytes())
}

struct LabLock(fs::File);

impl LabLock {
    async fn acquire(config: &VirtualQemuConfig) -> anyhow::Result<Self> {
        let path = lab_runtime_dir(config).join("lifecycle.lock");
        tokio::task::spawn_blocking(move || {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            let file = fs::OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .truncate(false)
                .open(&path)
                .with_context(|| format!("failed to open {}", path.display()))?;
            fs4::FileExt::lock(&file)
                .with_context(|| format!("failed to lock {}", path.display()))?;
            Ok::<_, anyhow::Error>(Self(file))
        })
        .await
        .context("virtual lab lock task failed")?
    }
}

impl Drop for LabLock {
    fn drop(&mut self) {
        let _ = fs4::FileExt::unlock(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::is_expected_dnsmasq_cmdline;

    #[test]
    fn dnsmasq_pid_must_belong_to_the_expected_command() {
        let pid_file = std::path::Path::new("/run/ostool/dnsmasq.pid");
        assert!(is_expected_dnsmasq_cmdline(
            b"/usr/sbin/dnsmasq\0--pid-file=/run/ostool/dnsmasq.pid\0",
            pid_file
        ));
        assert!(!is_expected_dnsmasq_cmdline(
            b"/usr/bin/unrelated\0--pid-file=/run/ostool/dnsmasq.pid\0",
            pid_file
        ));
        assert!(!is_expected_dnsmasq_cmdline(
            b"/usr/sbin/dnsmasq\0--pid-file=/run/other.pid\0",
            pid_file
        ));
    }
}
