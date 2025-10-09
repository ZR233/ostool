use clap::*;

pub mod defconfig;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub workdir: Option<String>,
    #[command(subcommand)]
    pub command: SubCommands,
}

#[derive(Subcommand)]
pub enum SubCommands {
    Build,
    Run(RunArgs),
    Test(TestArgs),
    Defconfig(defconfig::Cmd),
}

#[derive(Args, Debug)]
pub struct RunArgs {
    #[command(subcommand)]
    pub command: RunSubCommands,
}

#[derive(Subcommand, Debug)]
pub enum RunSubCommands {
    Qemu(QemuArgs),
    Uboot,
    Tftp,
}

#[derive(Args, Debug, Default)]
pub struct TestArgs {
    /// ELF file to test (for cargo test compatibility)
    #[arg(long)]
    pub elf: Option<String>,
    /// Test mode (board, qemu, uboot)
    pub mode: Option<String>,
    #[arg(long)]
    pub show_output: bool,
    /// Use U-Boot for testing
    #[arg(long)]
    pub uboot: bool,
    /// Don't run the binary, just build it
    #[arg(long)]
    pub no_run: bool,
    /// Board test mode (uses .board.toml configuration)
    #[arg(long)]
    pub board: bool,
    /// Additional arguments to pass to cargo test
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub trailing: Vec<String>,
}

#[derive(Args, Debug, Default)]
pub struct QemuArgs {
    #[arg(short, long)]
    pub debug: bool,
    #[arg(long)]
    pub dtb: bool,
}
