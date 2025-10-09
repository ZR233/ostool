// Integration tests for ostool
//
// This module contains comprehensive unit and integration tests for the ostool CLI tool.
// Tests cover:
// - Configuration loading and management
// - Project structure and workspace handling
// - Step system (build, test, run operations)
// - CLI argument parsing and validation
// - Integration scenarios and error handling

pub mod config_loader_tests;
pub mod project_tests;
pub mod step_tests;
pub mod cli_tests;
pub mod integration_tests;

// Re-export test utilities for convenience
pub use config_loader_tests::*;
pub use project_tests::*;
pub use step_tests::*;
pub use cli_tests::*;
pub use integration_tests::*;