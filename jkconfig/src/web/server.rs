//! Web服务器核心实现
//!
//! 负责启动和配置axum Web服务器

use std::net::SocketAddr;

use super::routes::create_routes;
use crate::data::AppData;

/// 运行Web服务器
pub async fn run_server(app_data: AppData, port: u16) -> anyhow::Result<()> {
    // 创建应用状态
    let state = AppState { app_data };

    // 创建路由
    let app = create_routes(state);

    // 绑定地址
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("🚀 Web服务器启动成功！");
    println!("📍 访问地址: http://localhost:{}", port);
    println!("⏹️  按 Ctrl+C 停止服务器");

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub app_data: AppData,
}
