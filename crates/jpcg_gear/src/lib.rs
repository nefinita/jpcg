//! #90 配装模板、合成与有界优化。API 独立于 core/combo 共享 DTO，不包含 UI 或时序公式。
pub mod attributes;
#[cfg(feature = "engine")]
mod catalog;
#[cfg(feature = "engine")]
pub mod ffi;
#[cfg(feature = "engine")]
pub mod host;
pub mod model;
#[cfg(feature = "engine")]
pub mod optimize;
pub use model::*;
#[cfg(feature = "engine")]
pub use optimize::{
    OptimizeProgress, optimize, optimize_joint, optimize_parallel, optimize_with, player,
};
use serde::{Deserialize, Serialize};

/// JSON 调用不读取任意路径，宿主显式加载并传入目录。版本由模板 schema 管理。
#[derive(Debug, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum ApiRequest {
    Validate {},
    Compose {
        loadout: Loadout,
    },
    Optimize {
        request: Box<OptimizeRequest>,
    },
    /// 时间轴模拟（调试用）：直接返回 TimelineResultDTO，便于比对窗口/事件。
    Timeline {
        request: Box<jpcg_api::TimelineRequestDTO>,
    },
    /// 由 loadout 合成并返回计算用 player 面板（调试/对拍用）。
    Player {
        loadout: Box<Loadout>,
    },
}
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ApiResponse {
    Ok { result: serde_json::Value },
    Error { message: String },
}
/// API 统一错误信封；CLI api 子命令可做无 GUI 的回归入口。
#[cfg(feature = "engine")]
pub fn call(catalog: &Catalog, input: &str) -> ApiResponse {
    fn run(catalog: &Catalog, input: &str) -> Result<serde_json::Value, String> {
        let req: ApiRequest = serde_json::from_str(input).map_err(|e| e.to_string())?;
        match req {
            ApiRequest::Validate {} => {
                catalog.validate()?;
                Ok(
                    serde_json::json!({"version": catalog.version, "templates": catalog.templates.len()}),
                )
            }
            ApiRequest::Compose { loadout } => {
                serde_json::to_value(catalog.compose(&loadout)?).map_err(|e| e.to_string())
            }
            ApiRequest::Optimize { request } => {
                serde_json::to_value(optimize(catalog, &request, |_| true)?)
                    .map_err(|e| e.to_string())
            }
            ApiRequest::Player { loadout } => {
                let composed = catalog.compose(&loadout)?;
                serde_json::to_value(player(&composed)).map_err(|e| e.to_string())
            }
            ApiRequest::Timeline { request } => jpcg_combo::timeline_host::calculate_timeline(
                jpcg_combo::timeline_host::prepare_timeline_from_store(*request)?,
            )
            .map_err(|e| e.to_string())
            .and_then(|r| serde_json::to_value(r).map_err(|e| e.to_string())),
        }
    }
    match run(catalog, input) {
        Ok(result) => ApiResponse::Ok { result },
        Err(message) => ApiResponse::Error { message },
    }
}
