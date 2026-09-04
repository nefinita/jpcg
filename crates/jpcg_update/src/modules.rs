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
    /// 产生该 dll 的 crate 版本（如 core="2.1.0"、const="130.3.20260602"）
    pub version: String,
    pub hash: String,
    #[serde(rename = "hash_type")]
    pub hash_type: String,
    pub size: u64,
}

impl ModulesFileEntry {
    /// 该 dll 是否属于本机平台（按扩展名判定：mac=.dylib linux=.so windows=.dll）
    fn matches_local_platform(name: &str) -> bool {
        let ext = Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        match std::env::consts::OS {
            "macos" => ext == "dylib",
            "linux" => ext == "so",
            "windows" => ext == "dll",
            _ => true,
        }
    }

    /// 是否应更新：本地快照缺该条目 / 版本不同 / 哈希不同
    fn needs_update(&self, local_manifest: &ModulesManifest, local_dir: &Path) -> bool {
        if !local_dir.join(&self.name).exists() {
            return true;
        }
        if let Some(local) = local_manifest.files.iter().find(|f| f.name == self.name) {
            if !local.version.is_empty() && local.version != self.version {
                return true;
            }
            local.hash != self.hash
        } else {
            true
        }
    }
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

/// 本地模块快照路径 = modules/modules_manifest.toml（记录上次应用的各 dll 版本+哈希）
pub fn local_snapshot_path(local_dir: &Path) -> PathBuf {
    local_dir.join(MODULES_MANIFEST_FILENAME)
}

fn load_local_snapshot(local_dir: &Path) -> ModulesManifest {
    let path = local_snapshot_path(local_dir);
    match std::fs::read_to_string(&path) {
        Ok(s) => toml::from_str(&s).unwrap_or_else(|_| ModulesManifest {
            modules_version: String::new(),
            platform: String::new(),
            files: vec![],
        }),
        Err(_) => ModulesManifest {
            modules_version: String::new(),
            platform: String::new(),
            files: vec![],
        },
    }
}

fn save_local_snapshot(local_dir: &Path, manifest: &ModulesManifest) {
    if let Ok(s) = toml::to_string_pretty(manifest) {
        let _ = std::fs::write(local_snapshot_path(local_dir), s);
    }
}

/// 对比本地 modules/ 目录，返回需要更新的模块文件
/// 逐 dll 比较：本地快照缺该条目 / 版本不同 / 哈希不同 → 需更新
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
    let local_manifest = load_local_snapshot(&local_dir);

    let mut needed = Vec::new();
    for entry in &manifest.files {
        // 跳过其他平台的 dll（服务器 modules_manifest 为三平台合并，platform="multi"）
        if !ModulesFileEntry::matches_local_platform(&entry.name) {
            continue;
        }
        if force || entry.needs_update(&local_manifest, &local_dir) {
            needed.push(entry.clone());
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

    // 更新本地快照：合并本次安装的条目
    let mut snapshot = load_local_snapshot(&dest_dir);
    snapshot.modules_version = app_version.to_string();
    for entry in files_to_update {
        if let Some(existing) = snapshot.files.iter_mut().find(|f| f.name == entry.name) {
            *existing = entry.clone();
        } else {
            snapshot.files.push(entry.clone());
        }
    }
    save_local_snapshot(&dest_dir, &snapshot);

    Ok(())
}

/// 兼容辅助：返回本地 modules 目录（供宿主上报路径）
pub fn modules_dir_for(base_path: &Path) -> PathBuf {
    let _ = base_path;
    modules_dir()
}
