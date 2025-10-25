//! Web服务器模块
//!
//! 提供基于axum的Web服务器功能，用于替代TUI界面提供配置编辑功能

#[cfg(feature = "web")]
pub mod handlers;
#[cfg(feature = "web")]
pub mod routes;
#[cfg(feature = "web")]
pub mod server;

// 重新导出server模块的run_server函数
#[cfg(feature = "web")]
pub use server::run_server;
