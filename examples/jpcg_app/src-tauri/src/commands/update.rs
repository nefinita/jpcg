#[cfg(feature = "static")]
use std::path::PathBuf;

#[cfg(feature = "static")]
use tauri::Emitter;

// ============================================================================
// 更新命令（双模式）
//   static  （默认）— 直调 jpcg_core::host::update，HostEvents 由本壳实现
//   dynamic        — 经 jpcg_call 调用 core dll，进度经 jpcg_set_host_events 回调
// ============================================================================

#[tauri::command]
pub async fn check_update(
    _app_handle: tauri::AppHandle,
    beta: bool,
    force: bool,
) -> Result<jpcg_update::UpdateCheckResult, String> {
    check_update_impl(beta, force)
}

#[cfg(feature = "static")]
fn check_update_impl(beta: bool, force: bool) -> Result<jpcg_update::UpdateCheckResult, String> {
    jpcg_core::host::update::check_update(beta, force)
}

#[cfg(feature = "dynamic")]
fn check_update_impl(beta: bool, force: bool) -> Result<jpcg_update::UpdateCheckResult, String> {
    let req = serde_json::json!({ "beta": beta, "force": force });
    crate::commands::ffi_bridge::call("update_check", &req)
}

#[tauri::command]
pub async fn perform_update(
    app_handle: tauri::AppHandle,
    beta: bool,
    has_data_update: bool,
    latest_data_version: Option<String>,
    data_files_to_update: Vec<String>,
) -> Result<String, String> {
    perform_update_impl(
        app_handle,
        beta,
        has_data_update,
        latest_data_version,
        data_files_to_update,
    )
}

#[cfg(feature = "static")]
fn perform_update_impl(
    app_handle: tauri::AppHandle,
    beta: bool,
    has_data_update: bool,
    latest_data_version: Option<String>,
    data_files_to_update: Vec<String>,
) -> Result<String, String> {
    let events = TauriEvents { app_handle };
    jpcg_core::host::update::perform_update(
        &events,
        beta,
        has_data_update,
        latest_data_version,
        data_files_to_update,
    )
}

#[cfg(feature = "dynamic")]
fn perform_update_impl(
    app_handle: tauri::AppHandle,
    beta: bool,
    has_data_update: bool,
    latest_data_version: Option<String>,
    data_files_to_update: Vec<String>,
) -> Result<String, String> {
    crate::commands::ffi_bridge::register_host_events(&app_handle)?;
    let req = serde_json::json!({
        "beta": beta,
        "has_data_update": has_data_update,
        "latest_data_version": latest_data_version,
        "data_files_to_update": data_files_to_update
    });
    crate::commands::ffi_bridge::call("update_perform", &req)
}

#[tauri::command]
pub async fn perform_app_update(
    app_handle: tauri::AppHandle,
    beta: bool,
) -> Result<String, String> {
    perform_app_update_impl(app_handle, beta)
}

#[cfg(feature = "static")]
fn perform_app_update_impl(app_handle: tauri::AppHandle, beta: bool) -> Result<String, String> {
    let events = TauriEvents {
        app_handle: app_handle.clone(),
    };
    jpcg_core::host::update::perform_app_update(&events, beta)
}

#[cfg(feature = "dynamic")]
fn perform_app_update_impl(app_handle: tauri::AppHandle, beta: bool) -> Result<String, String> {
    crate::commands::ffi_bridge::register_host_events(&app_handle)?;
    let req = serde_json::json!({ "beta": beta });
    crate::commands::ffi_bridge::call("update_app", &req)
}

/// 模块库（dll）增量更新：下载到 exe 同目录 modules/ 后请求重启
#[tauri::command]
pub async fn perform_modules_update(
    app_handle: tauri::AppHandle,
    beta: bool,
    modules_version: Option<String>,
    modules_files_to_update: Vec<jpcg_update::modules::ModulesFileEntry>,
) -> Result<String, String> {
    perform_modules_update_impl(app_handle, beta, modules_version, modules_files_to_update)
}

#[cfg(feature = "static")]
fn perform_modules_update_impl(
    app_handle: tauri::AppHandle,
    beta: bool,
    modules_version: Option<String>,
    modules_files_to_update: Vec<jpcg_update::modules::ModulesFileEntry>,
) -> Result<String, String> {
    let events = TauriEvents { app_handle };
    jpcg_core::host::update::perform_modules_update(
        &events,
        beta,
        modules_version,
        modules_files_to_update,
    )
}

#[cfg(feature = "dynamic")]
fn perform_modules_update_impl(
    app_handle: tauri::AppHandle,
    beta: bool,
    modules_version: Option<String>,
    modules_files_to_update: Vec<jpcg_update::modules::ModulesFileEntry>,
) -> Result<String, String> {
    crate::commands::ffi_bridge::register_host_events(&app_handle)?;
    let req = serde_json::json!({
        "beta": beta,
        "modules_version": modules_version,
        "modules_files_to_update": modules_files_to_update
    });
    crate::commands::ffi_bridge::call("update_modules", &req)
}

// ============================================================================
// 静态模式 HostEvents 实现（进度经 Tauri 事件通道，退出请求经 AppHandle）
// ============================================================================

#[cfg(feature = "static")]
struct TauriEvents {
    app_handle: tauri::AppHandle,
}

#[cfg(feature = "static")]
impl jpcg_core::host::update::HostEvents for TauriEvents {
    fn on_progress(&self, event: &jpcg_update::UpdateProgressEvent) {
        let _ = self.app_handle.emit("update-progress", event);
    }

    fn request_exit(&self) {
        self.app_handle.exit(0);
    }

    fn updater_path(&self) -> Option<PathBuf> {
        std::env::var("JPCG_UPDATER_PATH").ok().map(PathBuf::from)
    }
}
