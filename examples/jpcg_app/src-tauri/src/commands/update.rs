use std::path::Path;
use std::process::{Command, Stdio};
use tauri::Emitter;
use jpcg_update::ProgressCallback;

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

#[tauri::command]
pub async fn perform_app_update(
    app_handle: tauri::AppHandle,
    beta: bool,
) -> Result<String, String> {
    let base_path = Path::new(".");
    let base_path = base_path.canonicalize().map_err(|e| e.to_string())?;
    let progress = TauriProgress { app_handle: app_handle.clone() };

    // 1. 获取应用更新信息
    progress.on_progress(&jpcg_update::UpdateProgressEvent::new(
        "checking", "正在获取更新信息...", 0.0, None,
    ));
    let info = jpcg_update::fetch_app_update_info(&base_path, beta, false)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "没有可用的应用更新".to_string())?;

    // 2. 下载新二进制
    progress.on_progress(&jpcg_update::UpdateProgressEvent::new(
        "downloading", &format!("正在下载 {}...", info.version), 0.1, Some(&info.binary_path),
    ));
    let temp_path = jpcg_update::download_file_with_progress(
        &info.download_url, &info.binary_path, &progress,
    )
    .await
    .map_err(|e| format!("下载失败: {}", e))?;

    // 3. 验证哈希
    progress.on_progress(&jpcg_update::UpdateProgressEvent::new(
        "verifying", "正在验证文件...", 0.85, Some(&info.binary_path),
    ));
    let downloaded_hash = jpcg_update::calculate_file_sha256(&temp_path)
        .await
        .map_err(|e| e.to_string())?;
    if downloaded_hash != info.expected_hash {
        return Err("下载文件哈希验证失败，更新已取消。".to_string());
    }

    // 4. 获取路径信息
    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_dir = current_exe.parent().ok_or("无法获取程序目录".to_string())?;
    let workdir = std::env::current_dir().map_err(|e| e.to_string())?;

    // 5. 查找更新器路径
    let updater_name = if cfg!(windows) { "jpcg_updater.exe" } else { "jpcg_updater" };
    let mut updater_path = exe_dir.join(updater_name);
    if !updater_path.exists() {
        // 开发模式：从 target/debug 目录查找
        updater_path = workdir.join("target").join("debug").join(updater_name);
    }
    if !updater_path.exists() {
        // 尝试 target/release
        updater_path = workdir.join("target").join("release").join(updater_name);
    }
    if !updater_path.exists() {
        return Err("找不到更新器程序 (jpcg_updater)，请确认已编译。".to_string());
    }

    // 6. 启动更新器（异步等待主进程退出后替换二进制）
    let parent_pid = std::process::id();
    Command::new(&updater_path)
        .arg(parent_pid.to_string())
        .arg(current_exe.to_str().unwrap_or(""))
        .arg(temp_path.to_str().unwrap_or(""))
        .arg(workdir.to_str().unwrap_or(""))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("启动更新器失败: {}", e))?;

    // 7. 发送最终进度后退出应用
    let _ = app_handle.emit("update-progress", &jpcg_update::UpdateProgressEvent::new(
        "done", "更新完成，正在重启...", 1.0, None,
    ));

    // 延迟退出，确保事件发送完成
    std::thread::sleep(std::time::Duration::from_millis(300));
    app_handle.exit(0);

    Ok("重启中...".to_string())
}
