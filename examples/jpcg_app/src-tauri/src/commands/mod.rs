// ============================================================================
// mod.rs — Tauri 命令处理器
// 所有前端通过 invoke 调用的命令定义在此。
// 包含伤害计算、配置读写、版本更新、论坛下载四大类功能。
// ============================================================================

pub mod types;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::Emitter;
use types::*;

// ============================================================================
// 伤害计算
// ============================================================================

/// 执行伤害计算
/// - 接收前端提交的完整配置（玩家/目标/心法）
/// - 转换为核心库类型后调用 jpcg_core::calculate::start
/// - 返回每个技能的 7 段伤害结果
#[tauri::command]
pub async fn calculate_damage(req: CalculateRequest) -> Result<Vec<SkillResultDTO>, String> {
    // 将前端 DTO 转换为核心库类型
    let player = req.player.into_core();
    let hostile = req.hostile.into_core();
    let xinfa = req.xinfa_config.into_core();

    // 调用核心计算引擎
    let results = jpcg_core::calculate::start(player, hostile, xinfa);

    // 将核心计算结果转换为前端友好格式
    Ok(results.into_iter().flatten().map(SkillResultDTO::from).collect())
}

// ============================================================================
// 配置持久化
// ============================================================================

/// 保存玩家配置到本地文件
#[tauri::command]
pub fn save_config_cmd(
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
) -> Result<(), String> {
    jpcg_core::save_config::save(player.into_core(), hostile.into_core(), xinfa.into_core());
    Ok(())
}

/// 从本地文件加载默认配置
#[tauri::command]
pub fn load_config_cmd() -> Result<types::CalculateRequest, String> {
    let saved = jpcg_core::load_config::default_load();

    // 将核心库类型转换回前端 DTO
    Ok(types::CalculateRequest {
        player: PlayerConfigDTO {
            jcsx: saved.player.jcsx,
            jichu_shuxing: saved.player.jichu_shuxing,
            jichu_gongji: saved.player.jichu_gongji,
            huixin_dengji: saved.player.huixin_dengji,
            huixin_xiaoguo: saved.player.huixin_xiaoguo,
            pofang_dengji: saved.player.pofang_dengji,
            wuqi_shanghai: saved.player.wuqi_shanghai,
        },
        hostile: HostileConfigDTO {
            waigong_fangyu: saved.hostilepile.waigong_fangyu,
            neigong_fangyu: saved.hostilepile.neigong_fangyu,
            yujin_dengji: saved.hostilepile.yujin_dengji,
            huajin_dengji: saved.hostilepile.huajin_dengji,
            jianshang_bili: saved.hostilepile.jianshang_bili,
        },
        xinfa_config: XinfaConfigDTO {
            xinfa_name: saved.xinfa.xinfa_name,
            xinfa_nom: saved.xinfa.xinfa_nom,
            atk_up: saved.xinfa.atk_up,
            pofang_up: saved.xinfa.pofang_up,
            huixin_up: saved.xinfa.huixin_up,
        },
    })
}

/// 按心法名称加载特定配置
#[tauri::command]
pub fn load_profession_config(profession: String) -> Result<types::XinfaConfigDTO, String> {
    let toml_cfg = jpcg_core::load_config::show_config(&profession);

    Ok(types::XinfaConfigDTO {
        xinfa_name: toml_cfg.xinfa.xinfa_name,
        xinfa_nom: toml_cfg.xinfa.xinfa_nom,
        atk_up: toml_cfg.xinfa.atk_up,
        pofang_up: toml_cfg.xinfa.pofang_up,
        huixin_up: toml_cfg.xinfa.huixin_up,
    })
}

// ============================================================================
// 自动更新
// ============================================================================

/// Tauri 进度回调适配器
/// 将 jpcg_update 的 ProgressCallback 桥接到 Tauri 事件系统，
/// 通过 app_handle.emit 向 JS 前端发送 "update-progress" 事件。
struct TauriProgress {
    app_handle: tauri::AppHandle,
}

impl jpcg_update::ProgressCallback for TauriProgress {
    fn on_progress(&self, event: &jpcg_update::UpdateProgressEvent) {
        let _ = self.app_handle.emit("update-progress", event);
    }
}

/// 检查更新（仅检查，不下载）
/// - `beta`: 是否使用 Beta 通道
/// - `force`: 是否强制重新检查
/// - 返回: UpdateCheckResult（是否有更新、待更新文件列表等）
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

/// 执行更新下载
/// - 需要先调用 check_update 获取检查结果
/// - 通过 TauriProgress 实时发射下载进度事件
/// - 下载完成后自动更新本地版本信息
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
        // 从 check_update 的结果中提取信息，构造下载所需参数
        let check_result = jpcg_update::UpdateCheckResult {
            current_app_version: None,
            latest_app_version: None,
            has_app_update: false,
            current_data_version: None,
            latest_data_version,
            has_data_update: true,
            data_files_to_update,
        };

        // 执行下载（通过 progress 回调实时报告进度）
        jpcg_update::download_updates(&base_path, beta, &check_result, &progress)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok("更新完成".to_string())
}

// ============================================================================
// 论坛数据共享
// ============================================================================

/// 论坛文件条目（与 forum 服务端 /api/files 返回格式一致）
#[derive(Serialize, Deserialize)]
pub(crate) struct ForumFileInfo {
    name: String,
    size: u64,
    modified: String,
}

/// 获取论坛上所有可下载的 .toml 文件列表
/// - `forum_url`: 论坛服务器地址（如 "http://localhost:8080"）
/// - 返回: Vec<ForumFileInfo>（文件名、大小、修改时间）
#[tauri::command]
pub async fn forum_list_files(forum_url: String) -> Result<Vec<ForumFileInfo>, String> {
    let url = format!("{}/api/files", forum_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("连接论坛失败: {}", e))?;
    let files: Vec<ForumFileInfo> = resp
        .json()
        .await
        .map_err(|e| format!("解析文件列表失败: {}", e))?;
    Ok(files)
}

/// 从论坛下载 .toml 文件并保存到 data/pvp36500/ 目录
/// - `forum_url`: 论坛服务器地址
/// - `filename`: 要下载的文件名（如 "mowen.toml"）
/// - 返回: 成功消息
#[tauri::command]
pub async fn forum_download_file(
    forum_url: String,
    filename: String,
) -> Result<String, String> {
    if !filename.ends_with(".toml") {
        return Err("仅支持下载 .toml 文件".to_string());
    }

    let url = format!(
        "{}/download/{}",
        forum_url.trim_end_matches('/'),
        filename
    );
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("下载文件失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("服务器返回错误: {}", resp.status()));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("读取文件数据失败: {}", e))?;

    // 定位 data/pvp36500/ 目录（与核心库路径规则一致）
    let current_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();
    let dest_dir = current_dir.join("data").join("pvp36500");
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    let dest_path = dest_dir.join(&filename);
    std::fs::write(&dest_path, &bytes)
        .map_err(|e| format!("保存文件失败: {}", e))?;

    Ok(format!("已下载: {}", filename))
}
