// ============================================================================
// jpcg_update — 自动更新核心库
// 提供应用版本检查、数据文件（./data/）同步、下载安装等功能。
// 支持稳定版（stable）和测试版（beta）两个更新通道。
// ============================================================================

mod download;

/// 更新检查服务器基础 URL（稳定版）
const UPDATE_BASE_URL: &str = "https://nefinita-ai.com/updates/JPCG/";
/// 更新检查服务器基础 URL（Beta 版）
const BETA_BASE_URL: &str = "https://nefinita-ai.com/updates/JPCG_beta/";
/// 应用根目录（相对于工作目录的路径）
const CURRENT_DIR: &str = ".";

use std::env;
use std::path::Path;

// 将 download 模块的所有公有类型和函数重新导出
pub use download::*;

// ============================================================================
// check_updates — 检查更新（只检查不下载）
// 1. 从服务器获取 latest update.toml，解析版本号
// 2. 对比本地版本号判断应用是否有更新
// 3. 若有 data_version，进一步对比本地 data 版本并获取 data_manifest.toml
// 4. 返回完整的检查结果（UpdateCheckResult）
// ============================================================================

/// 检查应用版本和数据更新
/// - `base_path`: 应用根目录路径
/// - `beta`: 是否使用 Beta 通道
/// - `force`: 是否强制检查（忽略本地版本比较）
/// - 返回: UpdateCheckResult 包含所有检查结果
pub async fn check_updates(
    base_path: &Path,
    beta: bool,
    force: bool,
) -> Result<UpdateCheckResult, Box<dyn std::error::Error + Send + Sync>> {
    // 加载本地版本信息
    let local_info = load_local_version_info()?;
    // 判断更新通道：参数指定优先，否则使用上次记录的通道
    let use_beta = beta || local_info.channel == "beta";
    let base_url = if use_beta { BETA_BASE_URL } else { UPDATE_BASE_URL };
    let channel = if use_beta { "beta" } else { "stable" };

    // 从服务器获取最新版本信息
    let latest = fetch_latest_version_info(base_url).await?;
    let latest_info = match latest {
        Some(info) => info,
        None => {
            // 服务器不可用时，返回无可用版本信息
            return Ok(UpdateCheckResult {
                current_app_version: local_info.version.clone(),
                latest_app_version: None,
                has_app_update: false,
                current_data_version: local_info.data_version.clone(),
                latest_data_version: None,
                has_data_update: false,
                data_files_to_update: vec![],
            });
        }
    };

    // 判断应用是否有新版本
    let has_app_update = force
        || local_info.version.as_deref() != Some(&latest_info.version);

    // ---- 检查 data 文件更新 ----
    let mut has_data_update = false;
    let mut data_files_to_update = vec![];

    if let Some(ref remote_data_ver) = latest_info.data_version {
        let local_data_ver = local_info.data_version.as_deref().unwrap_or("");
        // 版本不同或强制检查时，获取 data_manifest 并与本地对比
        if force || remote_data_ver.as_str() != local_data_ver {
            let file_base_url = if use_beta {
                "https://nefinita-ai.com/files/JPCG_beta/".to_string()
            } else {
                "https://nefinita-ai.com/files/JPCG/".to_string()
            };

            match fetch_data_manifest(&file_base_url, remote_data_ver, channel).await {
                Ok(manifest) => {
                    // 对比本地 data 文件哈希，找出需要更新的文件
                    let needed = check_data_updates(base_path, &manifest).await?;
                    if !needed.is_empty() {
                        has_data_update = true;
                        data_files_to_update = needed.iter().map(|f| f.path.clone()).collect();
                    }
                }
                Err(e) => {
                    // 获取 data_manifest 失败不影响应用更新检查
                    eprintln!("获取数据清单失败: {}", e);
                }
            }
        }
    }

    Ok(UpdateCheckResult {
        current_app_version: local_info.version,
        latest_app_version: Some(latest_info.version),
        has_app_update,
        current_data_version: local_info.data_version,
        latest_data_version: latest_info.data_version,
        has_data_update,
        data_files_to_update,
    })
}

// ============================================================================
// download_updates — 根据检查结果执行下载和安装
// 接收 check_updates 返回的检查结果，遍历需要更新的文件逐一下载。
// 通过 ProgressCallback 回调向前端/CLI 报告进度。
// ============================================================================

/// 根据检查结果执行下载和安装
/// - `base_path`: 应用根目录
/// - `beta`: 是否 Beta 通道
/// - `check_result`: 检查结果（由 check_updates 返回）
/// - `progress`: 进度回调接口（用于 GUI 或 CLI 显示）
pub async fn download_updates(
    base_path: &Path,
    beta: bool,
    check_result: &UpdateCheckResult,
    progress: &dyn ProgressCallback,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let use_beta = beta;
    let channel = if use_beta { "beta" } else { "stable" };
    let file_base_url = if use_beta {
        "https://nefinita-ai.com/files/JPCG_beta/".to_string()
    } else {
        "https://nefinita-ai.com/files/JPCG/".to_string()
    };

    // 仅当有数据更新时执行下载
    if let Some(ref data_ver) = check_result.latest_data_version {
        if check_result.has_data_update {
            // 重新获取 data_manifest（因为需要文件哈希进行验证）
            let manifest = fetch_data_manifest(&file_base_url, data_ver, channel).await?;
            let needed = check_data_updates(base_path, &manifest).await?;
            if !needed.is_empty() {
                // 下载并安装所有需要更新的 data 文件
                download_and_install_data(&needed, base_path, data_ver, &file_base_url, channel, progress).await?;
            }
        }
    }

    Ok(())
}

// ============================================================================
// all_updates — 旧版 CLI 入口（全量检查 + 交互式更新）
// 保留用于命令行测试和独立更新器场景。
// 包含应用二进制更新和 data 更新两阶段。
// ============================================================================

/// 命令行全量更新检查（含交互式确认）
/// 仅在 CLI 模式下使用，Tauri 集成应使用 check_updates / download_updates
pub async fn all_updates() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use clap::Parser;

    /// 命令行参数定义
    #[derive(clap::Parser, Debug)]
    #[command(author, version, about)]
    struct Args {
        #[arg(long)]
        force_check: bool,          // 强制检查（忽略本地版本记录）
        #[arg(long)]
        target_os: Option<String>,  // 指定目标操作系统
        #[arg(long)]
        target_arch: Option<String>,// 指定目标架构
        #[arg(short = 'b', long = "beta")]
        beta: bool,                 // 使用 Beta 通道
    }

    let args = Args::parse();
    let app_dir = Path::new(CURRENT_DIR);
    let base_path = app_dir.canonicalize()?;

    let detected_os = args.target_os.as_deref().unwrap_or(env::consts::OS);
    let detected_arch = args.target_arch.as_deref().unwrap_or(env::consts::ARCH);

    let local_info = load_local_version_info()?;
    let use_beta = args.beta || local_info.channel == "beta";
    let base_url = if use_beta { BETA_BASE_URL } else { UPDATE_BASE_URL };
    let channel = if use_beta { "beta" } else { "stable" };

    let latest = fetch_latest_version_info(base_url).await?;
    let latest_info = match latest {
        Some(info) => info,
        None => {
            eprintln!("无法获取最新版本信息，跳过更新检查。");
            return Ok(());
        }
    };

    println!("当前版本: {:?}, 最新版本: {}", local_info.version, latest_info.version);

    // ---- 第一阶段: 应用二进制更新 ----
    if !args.force_check && local_info.version.as_deref() == Some(&latest_info.version) {
        println!("已是最新版本 ({}), 无需更新。", latest_info.version);
    } else {
        let version_dirs = fetch_all_version_directories(base_url).await?;
        let target_version = if let Some(major) = latest_info.major_version {
            find_latest_version_in_major(&version_dirs, major)?
        } else {
            version_dirs.first()
        };

        let target_dir = match target_version {
            Some(dir) => dir,
            None => {
                eprintln!("未找到匹配的版本目录，跳过更新。");
                return Ok(());
            }
        };

        let manifest = &target_dir.manifest;
        let target_version_str = &target_dir.dir_name;

        let target_binary =
            select_target_binary(&manifest.binaries, detected_os, detected_arch).ok();

        let mut all_updates_needed: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        // 检查二进制是否需要更新（SHA256 对比）
        if let Some(bin) = target_binary {
            if check_binary_update_needed(&base_path, bin).await? {
                all_updates_needed.insert(bin.path.clone(), bin.hash.clone());
            }
        }

        // 检查其他附带文件是否需要更新
        let other_updates = determine_other_updates_by_hash(&base_path, manifest).await?;
        for (path, hash) in other_updates {
            all_updates_needed.entry(path).or_insert(hash);
        }

        if !all_updates_needed.is_empty() {
            let version_url =
                format!("{}{}/", base_url.trim_end_matches('/'), target_version_str);
            let file_base_url = if use_beta {
                "https://nefinita-ai.com/files/JPCG_beta/".to_string()
            } else {
                "https://nefinita-ai.com/files/JPCG/".to_string()
            };

            // 进入交互式更新流程
            prompt_and_perform_update(
                all_updates_needed,
                &base_path,
                &version_url,
                &file_base_url,
                target_binary,
                detected_os,
                detected_arch,
                manifest,
                target_version_str,
                channel,
            )
            .await?;
        } else {
            println!("应用程序文件已是最新。");
        }
    }

    // ---- 第二阶段: Data 文件更新 ----
    if let Some(ref remote_data_ver) = latest_info.data_version {
        let local_data_ver = local_info.data_version.as_deref().unwrap_or("");
        if args.force_check || remote_data_ver.as_str() != local_data_ver {
            let file_base_url = if use_beta {
                "https://nefinita-ai.com/files/JPCG_beta/".to_string()
            } else {
                "https://nefinita-ai.com/files/JPCG/".to_string()
            };

            match fetch_data_manifest(&file_base_url, remote_data_ver, channel).await {
                Ok(manifest) => {
                    let needed = check_data_updates(&base_path, &manifest).await?;
                    if !needed.is_empty() {
                        println!("\n检测到数据更新 (版本: {}), 共 {} 个文件需要更新。", remote_data_ver, needed.len());
                        // CLI 模式下使用简化的控制台进度回调
                        struct CliProgress;
                        impl ProgressCallback for CliProgress {
                            fn on_progress(&self, event: &UpdateProgressEvent) {
                                if event.stage == "downloading" {
                                    println!("  [{:.0}%] {}", event.progress * 100.0, event.message);
                                }
                            }
                        }
                        download_and_install_data(&needed, &base_path, remote_data_ver, &file_base_url, channel, &CliProgress).await?;
                    }
                }
                Err(e) => {
                    eprintln!("获取数据清单失败: {}", e);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
