// src/lib.rs
use axum::{Json, Router, extract::State, routing::post};
use jpcg_core::{cal::CalculateResult, calculate};
use std::sync::Arc;
use tower_http::services::ServeDir;

// 应用状态 (如果需要共享状态，如缓存)
#[derive(Clone)]
struct AppState {
    data_dir: String,
}

#[tokio::main]
async fn main() {
    // 设置数据目录
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    let state = AppState { data_dir };

    // 构建路由
    let app = Router::new()
        // API 路由
        .route("/api/calculate", post(api_calculate))
        .route("/api/xinfa", get(api_get_xinfa_list))
        // 共享状态
        .with_state(Arc::new(state))
        // 静态文件服务 (前端页面)
        .nest_service("/", ServeDir::new("frontend/dist"));

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("🚀 服务启动于 http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn api_calculate(Json(req): Json<CalculateRequest>) -> Json<CalculateResponse> {
    // 调用核心逻辑
    match calculate(req) {
        Ok(resp) => Json(resp),
        Err(e) => {
            // 实际生产中应返回合适的 HTTP 状态码
            Json(CalculateResponse {
                result: format!("错误：{}", e),
                damage: 0.0,
            })
        }
    }
}
