// #[macro_use]
// extern crate log;

#[macro_use]
mod log;

pub mod data;
// UI模块暂时注释掉，使用主程序中的 MenuView
pub mod ui;

// Web服务器模块（需要web feature）
#[cfg(feature = "web")]
pub mod web;

pub use serde_json::Value;
