//! Main entry point for the mkimage CLI tool

use clap::Parser;
use mkimage::cli::{run_cli, Args};

fn main() {
    let args = Args::parse();

    if let Err(e) = run_cli(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
