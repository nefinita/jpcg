use std::path::Path;
use tauri::Emitter;

struct TauriProgress {
    app_handle: tauri::AppHandle,
}

impl jpcg_update::ProgressCallback for TauriProgress {
    fn on_progress(&self, event: &jpcg_update::UpdateProgressEvent) {
        let _ = self.app_handle.emit("update-progress", event);
    }
}

#[tauri::command]
pub async fn check_update(
    _app_handle: tauri::AppHandle,
    beta: bool,
    force: bool,
) -> Result<jpcg_update::UpdateCheckResult, String> {
    let base_path = Path::new(".");
    let base_path = base_path.canonicalize().map_err(|e| e.to_string())?;

    let result = jpcg_update::check_updates(&base_path, beta, force)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result)
}

#[tauri::command]
pub async fn perform_update(
    app_handle: tauri::AppHandle,
    beta: bool,
    has_data_update: bool,
    latest_data_version: Option<String>,
    data_files_to_update: Vec<String>,
) -> Result<String, String> {
    let base_path = Path::new(".");
    let base_path = base_path.canonicalize().map_err(|e| e.to_string())?;

    let progress = TauriProgress { app_handle };

    if has_data_update {
        let check_result = jpcg_update::UpdateCheckResult {
            current_app_version: None,
            latest_app_version: None,
            has_app_update: false,
            current_data_version: None,
            latest_data_version,
            has_data_update: true,
            data_files_to_update,
        };

        jpcg_update::download_updates(&base_path, beta, &check_result, &progress)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok("更新完成".to_string())
}
