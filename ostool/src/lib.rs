//! ostool - A tool for operating system development
//!
//! This library provides the core functionality for ostool, including:
//! - Configuration management
//! - Project workspace handling
//! - Build and test step execution
//! - CLI argument parsing

pub mod cmd;
pub mod config;
pub mod env;
pub mod project;
pub mod shell;
pub mod step;
pub mod ui;

// Re-export commonly used types for easier access in tests
pub use config::{ProjectConfig, loader::{ConfigLoader, ConfigMode}};
pub use project::{Project, Arch};
pub use step::{Step, Compile, TestPrepare};
pub use cmd::{Cli, SubCommands, RunArgs, RunSubCommands, TestArgs, QemuArgs};