// ============================================================================
// host::update — 更新编排入口（net 特性）
// 全权编排更新流程：检查 → 下载 → 验证 → 发动 jpcg_updater → 请求宿主退出。
// 异步逻辑经内嵌 tokio runtime 同步化，供 FFI 与静态直调统一入口。
// 宿主平台行为通过 HostEvents trait 注入（Tauri 壳实现 / CLI 实现）。
// ============================================================================

use std::path::Path;
use std::process::{Command, Stdio};

use jpcg_update::{ProgressCallback, UpdateProgressEvent};

/// 宿主事件接口 — 由附加组件（Tauri 壳/CLI）实现
pub trait HostEvents: Send + Sync {
    /// 上报更新进度（等价于原 Tauri 的 emit "update-progress"）
    fn on_progress(&self, event: &UpdateProgressEvent);
    /// 请求宿主退出（updater 已启动后调用）
    fn request_exit(&self);
    /// 注入 updater 二进制路径；返回 None 时回退到标准查找
    fn updater_path(&self) -> Option<std::path::PathBuf>;
}

/// 适配器：HostEvents → jpcg_update::ProgressCallback
struct HostProgress<'a>(&'a dyn HostEvents);

impl ProgressCallback for HostProgress<'_> {
    fn on_progress(&self, event: &UpdateProgressEvent) {
        self.0.on_progress(event);
    }
}

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    let rt = tokio::runtime::Runtime::new().expect("创建 tokio runtime 失败");
    rt.block_on(f)
}

fn base_path() -> Result<std::path::PathBuf, String> {
    let p = Path::new(".").canonicalize().map_err(|e| e.to_string())?;
    Ok(p)
}

/// 检查更新（应用 + 数据）
pub fn check_update(
    beta: bool,
    force: bool,
) -> Result<jpcg_update::UpdateCheckResult, String> {
    let base_path = base_path()?;
    block_on(jpcg_update::check_updates(&base_path, beta, force)).map_err(|e| e.to_string())
}

/// 执行模块库（dll）更新：下载 → 校验 → 安装到 exe 同目录 modules/ → 请求宿主重启
pub fn perform_modules_update(
    events: &dyn HostEvents,
    beta: bool,
    modules_version: Option<String>,
    files_to_update: Vec<jpcg_update::modules::ModulesFileEntry>,
) -> Result<String, String> {
    let progress = HostProgress(events);

    if files_to_update.is_empty() {
        return Ok("无需更新模块".to_string());
    }

    let version = modules_version.unwrap_or_default();
    let file_base_url = if beta {
        "https://nefinita-ai.com/files/JPCG_beta/".to_string()
    } else {
        "https://nefinita-ai.com/files/JPCG/".to_string()
    };
    let channel = if beta { "beta" } else { "stable" };

    block_on(jpcg_update::modules::download_and_install_modules(
        &files_to_update,
        &version,
        &file_base_url,
        channel,
        &progress,
    ))
    .map_err(|e| e.to_string())?;

    let module_dir = jpcg_update::modules::modules_dir();
    progress.on_progress(&UpdateProgressEvent::new(
        "done",
        &format!("模块更新完成，正在重启以加载新模块（{}）", module_dir.display()),
        1.0,
        None,
    ));
    std::thread::sleep(std::time::Duration::from_millis(300));
    events.request_exit();

    Ok("重启中...".to_string())
}

/// 执行数据更新下载
pub fn perform_update(
    events: &dyn HostEvents,
    beta: bool,
    has_data_update: bool,
    latest_data_version: Option<String>,
    data_files_to_update: Vec<String>,
) -> Result<String, String> {
    let base_path = base_path()?;
    let progress = HostProgress(events);

    if has_data_update {
        let check_result = jpcg_update::UpdateCheckResult {
            current_app_version: None,
            latest_app_version: None,
            has_app_update: false,
            current_data_version: None,
            latest_data_version,
            has_data_update: true,
            data_files_to_update,
            has_modules_update: false,
            modules_version: None,
            modules_files_to_update: vec![],
        };
        block_on(jpcg_update::download_updates(&base_path, beta, &check_result, &progress))
            .map_err(|e| e.to_string())?;
    }

    Ok("更新完成".to_string())
}

/// 定位 updater 二进制（壳注入优先，回退 exe 目录 / target 目录）
fn locate_updater(events: &dyn HostEvents) -> Result<std::path::PathBuf, String> {
    let updater_name = if cfg!(windows) {
        "jpcg_updater.exe"
    } else {
        "jpcg_updater"
    };

    if let Some(path) = events.updater_path() {
        if path.exists() {
            return Ok(path);
        }
    }

    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_dir = current_exe.parent().ok_or("无法获取程序目录".to_string())?;
    let candidate = exe_dir.join(updater_name);
    if candidate.exists() {
        return Ok(candidate);
    }

    let workdir = std::env::current_dir().map_err(|e| e.to_string())?;
    for dir in ["debug", "release"] {
        let candidate = workdir.join("target").join(dir).join(updater_name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err("找不到更新器程序 (jpcg_updater)，请确认已编译。".to_string())
}

/// 执行整包应用更新：下载 → 验证 → 发动 updater → 请求宿主退出
pub fn perform_app_update(events: &dyn HostEvents, beta: bool) -> Result<String, String> {
    let base_path = base_path()?;
    let progress = HostProgress(events);

    // 1. 获取应用更新信息
    progress.on_progress(&UpdateProgressEvent::new(
        "checking",
        "正在获取更新信息...",
        0.0,
        None,
    ));
    let info = block_on(jpcg_update::fetch_app_update_info(&base_path, beta, false))
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "没有可用的应用更新".to_string())?;

    // 2. 下载新二进制
    progress.on_progress(&UpdateProgressEvent::new(
        "downloading",
        &format!("正在下载 {}...", info.version),
        0.1,
        Some(&info.binary_path),
    ));
    let temp_path =
        block_on(jpcg_update::download_file_with_progress(&info.download_url, &info.binary_path, &progress))
            .map_err(|e| format!("下载失败: {}", e))?;

    // 3. 验证哈希
    progress.on_progress(&UpdateProgressEvent::new(
        "verifying",
        "正在验证文件...",
        0.85,
        Some(&info.binary_path),
    ));
    let downloaded_hash = block_on(jpcg_update::calculate_file_sha256(&temp_path))
        .map_err(|e| e.to_string())?;
    if downloaded_hash != info.expected_hash {
        return Err("下载文件哈希验证失败，更新已取消。".to_string());
    }

    // 4. 获取路径信息
    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let workdir = std::env::current_dir().map_err(|e| e.to_string())?;

    // 5. 定位并启动更新器（异步等待主进程退出后替换二进制）
    let updater_path = locate_updater(events)?;
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

    // 6. 发送最终进度后请求宿主退出
    progress.on_progress(&UpdateProgressEvent::new(
        "done",
        "更新完成，正在重启...",
        1.0,
        None,
    ));
    std::thread::sleep(std::time::Duration::from_millis(300));
    events.request_exit();

    Ok("重启中...".to_string())
}