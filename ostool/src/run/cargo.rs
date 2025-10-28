use std::path::PathBuf;

use crate::{build::config::Cargo, ctx::AppContext};

pub struct CargoRunner {
    pub cmd: String,
    pub args: Vec<String>,
}

impl CargoRunner {
    pub fn new(cmd: &str) -> Self {
        Self {
            cmd: cmd.to_string(),
            args: vec![],
        }
    }

    pub fn arg(&mut self, arg: impl Into<String>) {
        self.args.push(arg.into());
        self
    }

    pub async fn run(&mut self, ctx: &mut AppContext, config: Cargo) -> anyhow::Result<()> {
        Ok(())
    }
}
