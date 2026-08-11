// ============================================================================
// modules — 模块库（dll）更新能力
// 服务端布局（与 data 对齐）：
//   stable: {files_url}/{app_version}/modules/modules_manifest.toml + dll
//   beta:   {files_url}/modules/modules_manifest.toml + dll
// 本地布局：exe 同目录下的 modules/ 子目录（动态模式 app 优先从此加载 dll）
// ============================================================================

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::download::{self, ProgressCallback, UpdateProgressEvent};

pub const MODULES_MANIFEST_FILENAME: &str = "modules_manifest.toml";

/// 模块文件条目（对应 modules_manifest.toml 的 files 数组）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesFileEntry {
    pub name: String,
    pub hash: String,
    #[serde(rename = "hash_type")]
    pub hash_type: String,
    pub size: u64,
}

/// 模块清单（对应服务器 modules_manifest.toml）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesManifest {
    pub modules_version: String,
    pub platform: String,
    pub files: Vec<ModulesFileEntry>,
}

/// 模块更新检查结果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModulesCheckResult {
    pub modules_version: Option<String>,
    pub has_modules_update: bool,
    pub modules_files_to_update: Vec<ModulesFileEntry>,
}

/// 本地模块目录 = exe 同目录 / modules
pub fn modules_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("modules")
}

/// 拉取模块清单
pub async fn fetch_modules_manifest(
    file_base_url: &str,
    app_version: &str,
    channel: &str,
) -> Result<ModulesManifest, Box<dyn std::error::Error + Send + Sync>> {
    let manifest_url = if channel == "beta" {
        format!(
            "{}/modules/{}",
            file_base_url.trim_end_matches('/'),
            MODULES_MANIFEST_FILENAME
        )
    } else {
        format!(
            "{}/{}/modules/{}",
            file_base_url.trim_end_matches('/'),
            app_version,
            MODULES_MANIFEST_FILENAME
        )
    };
    let client = reqwest::Client::new();
    let response = client.get(&manifest_url).send().await?;
    if !response.status().is_success() {
        return Err(format!("获取模块清单失败，HTTP 状态码: {}", response.status()).into());
    }
    let toml_text = response.text().await?;
    let manifest: ModulesManifest = toml::from_str(&toml_text)?;
    Ok(manifest)
}

/// 对比本地 modules/ 目录，返回需要更新的模块文件
pub async fn check_modules_update(
    beta: bool,
    force: bool,
) -> Result<ModulesCheckResult, Box<dyn std::error::Error + Send + Sync>> {
    let use_beta = beta;
    let channel = if use_beta { "beta" } else { "stable" };
    let base_url = if use_beta {
        crate::BETA_BASE_URL
    } else {
        crate::UPDATE_BASE_URL
    };
    let file_base_url = if use_beta {
        "https://nefinita-ai.com/files/JPCG_beta/".to_string()
    } else {
        "https://nefinita-ai.com/files/JPCG/".to_string()
    };

    let mut result = ModulesCheckResult::default();

    let latest = download::fetch_latest_version_info(base_url).await?;
    let Some(info) = latest else {
        return Ok(result);
    };

    let manifest = match fetch_modules_manifest(&file_base_url, &info.version, channel).await {
        Ok(m) => m,
        Err(e) => {
            // 服务器未提供模块清单（如静态分发）时视为无需更新
            eprintln!("获取模块清单失败: {}", e);
            return Ok(result);
        }
    };

    let local_dir = modules_dir();
    let mut needed = Vec::new();
    for entry in &manifest.files {
        let local = local_dir.join(&entry.name);
        if force || !local.exists() {
            needed.push(entry.clone());
            continue;
        }
        match download::calculate_file_sha256(&local).await {
            Ok(hash) if hash == entry.hash => {}
            _ => needed.push(entry.clone()),
        }
    }

    if !needed.is_empty() {
        result.has_modules_update = true;
        result.modules_version = Some(manifest.modules_version);
        result.modules_files_to_update = needed;
    }
    Ok(result)
}

/// 下载并安装模块库到 exe 同目录 modules/（每文件下载 → 校验哈希 → 原子替换）
pub async fn download_and_install_modules(
    files_to_update: &[ModulesFileEntry],
    app_version: &str,
    file_base_url: &str,
    channel: &str,
    progress: &dyn ProgressCallback,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let dest_dir = modules_dir();
    std::fs::create_dir_all(&dest_dir)?;
    let total = files_to_update.len();

    progress.on_progress(&UpdateProgressEvent::new(
        "installing_modules",
        &format!("开始更新模块库 (共 {} 个)", total),
        0.0,
        None,
    ));

    for (idx, entry) in files_to_update.iter().enumerate() {
        let base = idx as f64 / total as f64;
        progress.on_progress(&UpdateProgressEvent::new(
            "downloading",
            &format!("正在下载模块 {}", entry.name),
            base,
            Some(&entry.name),
        ));

        let file_url = if channel == "beta" {
            format!(
                "{}/modules/{}",
                file_base_url.trim_end_matches('/'),
                entry.name
            )
        } else {
            format!(
                "{}/{}/modules/{}",
                file_base_url.trim_end_matches('/'),
                app_version,
                entry.name
            )
        };

        let tmp_path = dest_dir.join(format!("{}.tmp", entry.name));
        let tmp_str = tmp_path.to_string_lossy().into_owned();
        let _ = download::download_file_with_progress(&file_url, &tmp_str, progress).await?;

        // 校验哈希
        let downloaded_hash = download::calculate_file_sha256(&tmp_path).await?;
        if downloaded_hash != entry.hash {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("模块 {} 哈希校验失败，更新已取消。", entry.name).into());
        }

        // 原子替换
        let final_path = dest_dir.join(&entry.name);
        std::fs::rename(&tmp_path, &final_path)?;

        progress.on_progress(&UpdateProgressEvent::new(
            "installing_modules",
            &format!("已安装模块 {}", entry.name),
            (idx + 1) as f64 / total as f64,
            Some(&entry.name),
        ));
    }

    Ok(())
}

/// 兼容辅助：返回本地 modules 目录（供宿主上报路径）
pub fn modules_dir_for(base_path: &Path) -> PathBuf {
    let _ = base_path;
    modules_dir()
}
