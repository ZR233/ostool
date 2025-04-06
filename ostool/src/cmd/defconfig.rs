use clap::*;

use crate::{
    config::ProjectConfig,
    project::{Arch, Project},
};

#[derive(Parser)]
pub struct Cmd {
    #[arg(long, short)]
    pub r#type: Option<ConfigKind>,
}

#[derive(ValueEnum, Clone)]
pub enum ConfigKind {
    Test,
}

impl Cmd {
    pub fn run(&self, project: &mut Project) {
        let mut config = ProjectConfig::new(Arch::Aarch64);
        config.compile.target = "aarch64-unknown-none".to_string();
        project.config = Some(config.clone());
        let config_path = project.workdir().join(".project.toml");

        println!("config save at {}", project.workdir().display());
        config.save(&config_path);
    }
}
