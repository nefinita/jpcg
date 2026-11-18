use jpcg_api::ComputeBackendDTO;

/// 列举本机可用的计算平台：CPU（完整 AVX-512 支持或标量回退）在前，其后是 wgpu 适配器。
#[tauri::command]
pub async fn list_compute_backends() -> Result<Vec<ComputeBackendDTO>, String> {
    #[cfg(feature = "static")]
    {
        tauri::async_runtime::spawn_blocking(jpcg_compute::list_backends)
            .await
            .map_err(|e| e.to_string())
    }
    #[cfg(not(feature = "static"))]
    {
        tauri::async_runtime::spawn_blocking(|| {
            super::gear_bridge::call("compute_backends", &serde_json::json!({}))
        })
        .await
        .map_err(|e| e.to_string())?
    }
}
