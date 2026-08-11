// ============================================================================
// download — 下载与更新引擎
// 提供完整的更新流程支持：
//   1. 从服务器获取版本信息和清单文件
//   2. 对比本地文件哈希确定需要更新的文件
//   3. 带进度条/进度回调的文件下载
//   4. SHA256 哈希验证
//   5. 文件替换与本地版本信息持久化
//   6. 压缩包解压（通过 7z 外部工具）
// ============================================================================

use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

// ============================================================================
// 常量定义
// ============================================================================

/// 应用版本清单文件名（服务器上每个版本目录下的 manifest.toml）
pub const MANIFEST_FILENAME: &str = "manifest.toml";
/// 默认主程序名称
pub const DEFAULT_BINARY_NAME: &str = "JPCG";
/// 本地版本信息文件名（存储在当前工作目录）
const LOCAL_VERSION_FILE: &str = "local_update_info.toml";
/// 数据文件清单文件名（服务器上的 data_manifest.toml）
pub const DATA_MANIFEST_FILENAME: &str = "data_manifest.toml";

// ============================================================================
// 数据结构
// ============================================================================

/// 应用版本清单（对应服务器 manifest.toml 的顶层结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: Option<String>,       // 版本号（如 "v1.1.251222"）
    pub major_version: Option<u32>,    // 主版本号
    pub binaries: Vec<BinaryEntry>,    // 各平台的二进制入口列表
    pub files: Option<Vec<FileEntry>>, // 附带文件列表（可选）
    pub compressed_package: Option<CompressedPackageEntry>, // 压缩包入口（可选）
}

/// 二进制入口（对应 manifest.toml 中的 [[binaries]]）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryEntry {
    pub path: String, // 二进制文件路径
    pub os: String,   // 目标操作系统（如 "windows"、"linux"、"macos"）
    pub arch: String, // 目标架构（如 "x86_64"、"aarch64"）
    pub hash: String, // SHA256 哈希值
    #[serde(rename = "hash_type")]
    pub hash_type: String, // 哈希类型（当前仅支持 "SHA256"）
}

/// 附带文件入口（对应 manifest.toml 中的 [[files]]）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String, // 相对路径
    pub hash: String, // SHA256 哈希值
    #[serde(rename = "hash_type")]
    pub hash_type: String, // 哈希类型
}

/// 压缩包入口（对应 manifest.toml 中的 [compressed_package]）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedPackageEntry {
    pub path: String, // 压缩包文件路径
    pub hash: String, // SHA256 哈希值
    #[serde(rename = "hash_type")]
    pub hash_type: String, // 哈希类型
}

/// 服务器 update.toml 信息结构
/// 位于更新服务器根目录，记录最新版本号及相关元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTomlInfo {
    pub version: String,              // 最新版本号（如 "v1.1.251222"）
    pub major_version: Option<u32>,   // 最新主版本号
    pub data_version: Option<String>, // 最新数据版本号（如 "v2.0.2026050201"）
}

/// 本地版本信息结构
/// 持久化在 local_update_info.toml，记录本地安装版本和更新渠道
#[derive(Debug, Serialize, Deserialize)]
pub struct LocalVersionInfo {
    pub version: Option<String>,              // 本地安装的应用版本
    pub major_version: Option<u32>,           // 本地安装的主版本号
    pub channel: String,                      // 更新渠道（"stable" / "beta"）
    pub last_checked_version: Option<String>, // 上次检查到的服务器版本
    pub last_checked_major: Option<u32>,      // 上次检查到的服务器主版本
    pub data_version: Option<String>,         // 本地数据文件版本
}

/// 数据文件的清单入口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFileEntry {
    pub path: String, // 相对于 data/ 目录的路径
    pub hash: String, // SHA256 哈希值
    #[serde(rename = "hash_type")]
    pub hash_type: String, // 哈希类型
}

/// 数据文件清单（对应服务器 data_manifest.toml）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataManifest {
    pub data_version: String,      // 数据版本号
    pub files: Vec<DataFileEntry>, // 数据文件列表
}

/// 更新进度事件（用于 GUI 进度回调）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgressEvent {
    pub stage: String, // 阶段标识: "checking" / "downloading" / "verifying" / "installing" / "done" / "error"
    pub message: String, // 可读的描述信息
    pub progress: f64, // 进度值 [0.0, 1.0]
    pub file: Option<String>, // 当前处理的文件名（若有）
}

impl UpdateProgressEvent {
    /// 创建进度事件
    pub fn new(stage: &str, message: &str, progress: f64, file: Option<&str>) -> Self {
        Self {
            stage: stage.to_string(),
            message: message.to_string(),
            progress,
            file: file.map(|s| s.to_string()),
        }
    }
}

/// 进度回调接口
/// 实现者可向 GUI 发射事件或向 CLI 打印进度
pub trait ProgressCallback: Send + Sync {
    fn on_progress(&self, event: &UpdateProgressEvent);
}

/// 更新检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub current_app_version: Option<String>,  // 本地应用版本
    pub latest_app_version: Option<String>,   // 服务器最新应用版本
    pub has_app_update: bool,                 // 是否有应用更新
    pub current_data_version: Option<String>, // 本地数据版本
    pub latest_data_version: Option<String>,  // 服务器最新数据版本
    pub has_data_update: bool,                // 是否有数据更新
    pub data_files_to_update: Vec<String>,    // 需要更新的数据文件列表
    pub has_modules_update: bool,             // 是否有模块库（dll）更新
    pub modules_version: Option<String>,      // 服务器模块版本
    pub modules_files_to_update: Vec<crate::modules::ModulesFileEntry>, // 需要更新的模块文件列表
}

/// 应用更新信息（由 fetch_app_update_info 返回）
#[derive(Debug, Clone, Serialize)]
pub struct AppUpdateInfo {
    pub download_url: String,  // 二进制文件下载 URL
    pub expected_hash: String, // 期望的 SHA256 哈希
    pub binary_path: String,   // manifest 中的 relative path
    pub version: String,       // 目标版本号（如 "v2.1.0"）
}

/// 版本目录信息
/// 由服务器目录列表解析得出（每个版本号对应一个目录，内含 manifest.toml）
#[derive(Debug)]
pub struct VersionDirectory {
    pub dir_name: String,   // 目录名（如 "v1.1.251222"）
    pub manifest: Manifest, // 该版本的清单
}

// ============================================================================
// 工具函数: 网络请求与服务器通信
// ============================================================================

/// 从服务器获取最新版本信息（update.toml）
pub async fn fetch_latest_version_info(
    base_url: &str,
) -> Result<Option<UpdateTomlInfo>, Box<dyn std::error::Error + Send + Sync>> {
    let update_info_url = format!("{}/{}", base_url.trim_end_matches('/'), "update.toml");

    let client = reqwest::Client::new();
    let response = client.get(&update_info_url).send().await?;

    // 非 2xx 状态码视为无可用的版本信息（不视为错误）
    if !response.status().is_success() {
        eprintln!("警告: 无法获取 update.toml: HTTP {}", response.status());
        return Ok(None);
    }

    let toml_text = response.text().await?;
    let update_info: UpdateTomlInfo = toml::from_str(&toml_text)?;
    Ok(Some(update_info))
}

/// 从服务器目录列表抓取所有版本目录
/// 通过正则解析 HTML 页面中的超链接，筛选出版本号格式的目录
pub async fn fetch_all_version_directories(
    base_url: &str,
) -> Result<Vec<VersionDirectory>, Box<dyn std::error::Error + Send + Sync>> {
    let body = reqwest::get(base_url).await?.text().await?;
    // 匹配 <a href="..."> 中的链接文本
    let re = Regex::new(r#"(?i)<a\s+[^>]*href\s*=\s*["']([^"'/\s>]+)[^>]*>"#)?;
    let mut version_dirs = Vec::new();

    for cap in re.captures_iter(&body) {
        if let Some(version_match) = cap.get(1) {
            let dir_name = version_match.as_str();
            // 过滤出版本号格式: v{major}.{minor}.{patch}
            if dir_name.starts_with('v')
                && Regex::new(r"^v\d+\.\d+\.\d+$").unwrap().is_match(dir_name)
            {
                let manifest_url = format!(
                    "{}/{}/{}",
                    base_url.trim_end_matches('/'),
                    dir_name,
                    MANIFEST_FILENAME
                );
                match download_and_parse_manifest(&manifest_url).await {
                    Ok(manifest) => {
                        version_dirs.push(VersionDirectory {
                            dir_name: dir_name.to_string(),
                            manifest,
                        });
                    }
                    Err(e) => {
                        eprintln!("警告: 无法加载版本目录 '{}' 的清单: {}", dir_name, e);
                    }
                }
            }
        }
    }

    // 按版本号降序排列（最新的在前）
    version_dirs.sort_by(|a, b| b.dir_name.cmp(&a.dir_name));
    Ok(version_dirs)
}

/// 在已拉取的版本列表中查找指定主版本的最新版本
pub fn find_latest_version_in_major(
    versions: &[VersionDirectory],
    target_major: u32,
) -> Result<Option<&VersionDirectory>, Box<dyn std::error::Error + Send + Sync>> {
    let filtered: Vec<&VersionDirectory> = versions
        .iter()
        .filter(|v| v.manifest.major_version == Some(target_major))
        .collect();

    if filtered.is_empty() {
        Ok(None)
    } else {
        Ok(filtered.first().copied())
    }
}

/// 从可用的二进制列表中选择匹配当前平台的二进制
pub fn select_target_binary<'a>(
    binaries: &'a [BinaryEntry],
    target_os: &str,
    target_arch: &str,
) -> Result<&'a BinaryEntry, Box<dyn std::error::Error + Send + Sync>> {
    for entry in binaries {
        if entry.os == target_os && entry.arch == target_arch {
            return Ok(entry);
        }
    }
    Err(format!("未找到适用于 {} {} 的二进制文件。", target_os, target_arch).into())
}

// ============================================================================
// 工具函数: 更新检查与哈希验证
// ============================================================================

/// 检查本地二进制是否需要更新（不存在或哈希不匹配）
pub async fn check_binary_update_needed(
    base_path: &Path,
    target_binary_entry: &BinaryEntry,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    // 确定本地二进制文件的最终名称（考虑不同平台的扩展名差异）
    let final_binary_name =
        determine_final_binary_name(&target_binary_entry.path, env::consts::OS)?;
    let local_binary_path = base_path.join(final_binary_name);

    // 本地文件不存在，需要下载
    if !local_binary_path.exists() {
        return Ok(true);
    }

    // 本地文件存在但哈希不匹配，需要更新
    let local_hash = calculate_file_sha256(&local_binary_path).await?;
    if local_hash != target_binary_entry.hash {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 通过哈希对比检查附带文件是否需要更新
pub async fn determine_other_updates_by_hash(
    base_path: &Path,
    manifest: &Manifest,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut updates_needed = HashMap::new();

    if let Some(files_list) = &manifest.files {
        for file_entry in files_list {
            let local_file_path = base_path.join(&file_entry.path);

            if !local_file_path.exists() {
                updates_needed.insert(file_entry.path.clone(), file_entry.hash.clone());
                continue;
            }

            let local_hash = calculate_file_sha256(&local_file_path).await?;
            if local_hash != file_entry.hash {
                updates_needed.insert(file_entry.path.clone(), file_entry.hash.clone());
            }
        }
    }

    Ok(updates_needed)
}

/// 检查本地数据文件是否需要更新（不存在或哈希不匹配）
pub async fn check_data_updates(
    base_path: &Path,
    manifest: &DataManifest,
) -> Result<Vec<DataFileEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let data_dir = base_path.join("data");
    let mut needed = Vec::new();

    for file_entry in &manifest.files {
        let local_path = data_dir.join(&file_entry.path);

        if !local_path.exists() {
            needed.push(file_entry.clone());
            continue;
        }

        let local_hash = calculate_file_sha256(&local_path).await?;
        if local_hash != file_entry.hash {
            needed.push(file_entry.clone());
        }
    }

    Ok(needed)
}

// ============================================================================
// 工具函数: 文件下载
// ============================================================================

/// 下载文件（CLI 风格，使用 indicatif 进度条）
pub async fn download_file(
    url: &str,
    file_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let mut response = client
        .get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("下载文件失败，HTTP 状态码: {}", response.status()).into());
    }

    let total_size = response.content_length().unwrap_or(0);

    // 创建 CLI 进度条
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
            .progress_chars("#>-"),
    );
    pb.set_message(format!("正在下载 {}", file_name));

    // 下载到临时文件
    let temp_file = tempfile::NamedTempFile::new()?;
    let temp_path = temp_file.into_temp_path();
    let mut file_handle = tokio::fs::File::create(&temp_path).await?;
    use tokio::io::AsyncWriteExt;

    let mut downloaded: u64 = 0;
    while let Some(chunk) = response.chunk().await? {
        file_handle.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    file_handle.flush().await?;
    pb.finish_with_message(format!("已下载 {}", file_name));

    // 检查下载完整性
    let final_size = tokio::fs::metadata(&temp_path).await?.len();
    if final_size == 0 {
        std::fs::remove_file(&temp_path)?;
        return Err(format!("下载的文件 {} 为空。", file_name).into());
    }

    // 保留临时文件（防止被自动删除），返回路径
    let kept_path = temp_path
        .keep()
        .map_err(|e| format!("无法保留临时文件: {}", e.error))?;
    Ok(kept_path)
}

/// 带进度回调的文件下载（替代 indicatif 版本，用于 GUI）
pub async fn download_file_with_progress(
    url: &str,
    file_name: &str,
    progress: &dyn ProgressCallback,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let mut response = client
        .get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("下载文件失败，HTTP 状态码: {}", response.status()).into());
    }

    let total_size = response.content_length().unwrap_or(0);

    // 发送初始进度
    progress.on_progress(&UpdateProgressEvent::new(
        "downloading",
        &format!("正在下载 {}", file_name),
        0.0,
        Some(file_name),
    ));

    // 下载到临时文件，逐 chunk 报告进度
    let temp_file = tempfile::NamedTempFile::new()?;
    let temp_path = temp_file.into_temp_path();
    let mut file_handle = tokio::fs::File::create(&temp_path).await?;
    use tokio::io::AsyncWriteExt;

    let mut downloaded: u64 = 0;
    while let Some(chunk) = response.chunk().await? {
        file_handle.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            progress.on_progress(&UpdateProgressEvent::new(
                "downloading",
                &format!("正在下载 {}", file_name),
                downloaded as f64 / total_size as f64,
                Some(file_name),
            ));
        }
    }

    file_handle.flush().await?;

    // 完整性检查
    let final_size = tokio::fs::metadata(&temp_path).await?.len();
    if final_size == 0 {
        std::fs::remove_file(&temp_path)?;
        return Err(format!("下载的文件 {} 为空。", file_name).into());
    }

    let kept_path = temp_path
        .keep()
        .map_err(|e| format!("无法保留临时文件: {}", e.error))?;
    Ok(kept_path)
}

// ============================================================================
// 工具函数: 文件操作（替换、哈希、解压等）
// ============================================================================

/// 用下载的临时文件替换目标文件（先创建父目录）
pub async fn replace_file_or_prompt(
    from: &Path,
    to: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(debug_assertions)]
    if to.exists() {
        println!("正在替换现有文件: {}", to.display());
    } else {
        println!("正在创建新文件: {}", to.display());
    }

    // 确保目标父目录存在
    if let Some(parent) = to.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("创建目录 '{}' 失败: {}", parent.display(), e))?;
    }

    // 复制文件
    tokio::fs::copy(from, to).await.map_err(|e| {
        format!(
            "复制文件 '{}' 到 '{}' 失败: {}",
            from.display(),
            to.display(),
            e
        )
    })?;

    #[cfg(debug_assertions)]
    println!("文件已复制/替换: {}", to.display());
    Ok(())
}

/// 计算文件的 SHA256 哈希值
pub async fn calculate_file_sha256(
    file_path: &Path,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let n = tokio::io::AsyncReadExt::read(&mut file, &mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let hash_result = hasher.finalize();
    Ok(hex::encode(hash_result))
}

/// 下载并解析服务器上的 manifest.toml
pub async fn download_and_parse_manifest(
    manifest_url: &str,
) -> Result<Manifest, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let response = client.get(manifest_url).send().await?;
    if !response.status().is_success() {
        return Err(format!("下载清单失败，HTTP 状态码: {}", response.status()).into());
    }
    let toml_text = response.text().await?;
    let manifest: Manifest = toml::from_str(&toml_text)?;

    // 验证所有哈希类型是否均为 SHA256
    #[cfg(debug_assertions)]
    {
        if let Some(pkg) = &manifest.compressed_package
            && pkg.hash_type != "SHA256"
        {
            eprintln!(
                "警告: 压缩包 '{}' 的哈希类型 '{}' 不受支持。期望 SHA256。",
                pkg.path, pkg.hash_type
            );
        }
        for binary_entry in &manifest.binaries {
            if binary_entry.hash_type != "SHA256" {
                eprintln!(
                    "警告: 二进制文件 '{}' 的哈希类型 '{}' 不受支持。期望 SHA256。",
                    binary_entry.path, binary_entry.hash_type
                );
            }
        }
        if let Some(files_list) = &manifest.files {
            for file_entry in files_list {
                if file_entry.hash_type != "SHA256" {
                    eprintln!(
                        "警告: 文件 '{}' 的哈希类型 '{}' 不受支持。期望 SHA256。",
                        file_entry.path, file_entry.hash_type
                    );
                }
            }
        }
    }
    Ok(manifest)
}

/// 从服务器下载并解析 data_manifest.toml
/// 稳定版 URL 模式: {file_base_url}/{data_version}/data_manifest.toml
/// Beta 版 URL 模式: {file_base_url}/data/data_manifest.toml
pub async fn fetch_data_manifest(
    file_base_url: &str,
    data_version: &str,
    channel: &str,
) -> Result<DataManifest, Box<dyn std::error::Error + Send + Sync>> {
    let manifest_url = if channel == "beta" {
        format!(
            "{}/data/{}",
            file_base_url.trim_end_matches('/'),
            DATA_MANIFEST_FILENAME
        )
    } else {
        format!(
            "{}/{}/data/{}",
            file_base_url.trim_end_matches('/'),
            data_version,
            DATA_MANIFEST_FILENAME
        )
    };

    let client = reqwest::Client::new();
    let response = client.get(&manifest_url).send().await?;
    if !response.status().is_success() {
        return Err(format!("获取数据清单失败，HTTP 状态码: {}", response.status()).into());
    }

    let toml_text = response.text().await?;
    let manifest: DataManifest = toml::from_str(&toml_text)?;
    Ok(manifest)
}

// ============================================================================
// 工具函数: 数据文件同步
// ============================================================================

/// 下载并安装所有需要更新的数据文件
/// 每下载一个文件前报告进度，下载后验证哈希，最后安装到目标路径。
/// 全部更新完成后更新本地 data_version。
pub async fn download_and_install_data(
    files_to_update: &[DataFileEntry],
    base_path: &Path,
    data_version: &str,
    file_base_url: &str,
    channel: &str,
    progress: &dyn ProgressCallback,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let data_dir = base_path.join("data");
    let total = files_to_update.len();

    progress.on_progress(&UpdateProgressEvent::new(
        "installing_data",
        &format!("开始更新数据文件 (共 {} 个)", total),
        0.0,
        None,
    ));

    for (i, file_entry) in files_to_update.iter().enumerate() {
        // 构建下载 URL
        let file_url = if channel == "beta" {
            format!(
                "{}/data/{}",
                file_base_url.trim_end_matches('/'),
                file_entry.path
            )
        } else {
            format!(
                "{}/{}/data/{}",
                file_base_url.trim_end_matches('/'),
                data_version,
                file_entry.path
            )
        };

        // 报告即将下载
        let msg = format!("({}/{}) {}", i + 1, total, file_entry.path);
        progress.on_progress(&UpdateProgressEvent::new(
            "downloading",
            &msg,
            i as f64 / total as f64,
            Some(&file_entry.path),
        ));

        // 下载到临时文件
        let temp_path = download_file_with_progress(&file_url, &file_entry.path, progress).await?;

        // 验证哈希
        let downloaded_hash = calculate_file_sha256(&temp_path).await?;
        if downloaded_hash != file_entry.hash {
            return Err(format!("数据文件 {} 哈希验证失败。", file_entry.path).into());
        }

        // 安装到目标路径
        let target_path = data_dir.join(&file_entry.path);
        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        replace_file_or_prompt(temp_path.as_path(), target_path.as_path()).await?;

        // 报告安装完成
        progress.on_progress(&UpdateProgressEvent::new(
            "installing",
            &format!("已更新: {}", file_entry.path),
            (i + 1) as f64 / total as f64,
            Some(&file_entry.path),
        ));
    }

    // 全部完成后更新本地 data_version
    progress.on_progress(&UpdateProgressEvent::new("done", "数据更新完成", 1.0, None));

    let mut local_info = load_local_version_info()?;
    local_info.data_version = Some(data_version.to_string());
    save_local_version_info(&local_info)?;

    Ok(())
}

// ============================================================================
// 工具函数: 文件名适配
// ============================================================================

/// 根据目标操作系统确定本地二进制文件的最终名称
/// - Windows: 确保以 .exe 结尾
/// - Linux/macOS: 去除可能的 .exe 后缀，保留原名
pub fn determine_final_binary_name(
    server_path: &str,
    os: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let final_name = match os {
        "windows" => {
            if server_path.ends_with(".exe") {
                server_path.to_string()
            } else {
                format!("{}.exe", server_path)
            }
        }
        _ => {
            // Linux, macOS
            if server_path.ends_with(".exe") {
                server_path
                    .strip_suffix(".exe")
                    .unwrap_or(server_path)
                    .to_string()
            } else {
                server_path.to_string()
            }
        }
    };

    Ok(final_name)
}

// ============================================================================
// 工具函数: 本地版本信息持久化
// ============================================================================

/// 从 local_update_info.toml 加载本地版本信息
pub fn load_local_version_info()
-> Result<LocalVersionInfo, Box<dyn std::error::Error + Send + Sync>> {
    let path = Path::new(LOCAL_VERSION_FILE);
    if path.exists() {
        let contents = fs::read_to_string(path)?;
        let info: LocalVersionInfo = toml::from_str(&contents)?;
        Ok(info)
    } else {
        // 文件不存在时返回默认值（稳定版通道，无版本信息）
        Ok(LocalVersionInfo {
            version: None,
            major_version: None,
            channel: "stable".to_string(),
            last_checked_version: None,
            last_checked_major: None,
            data_version: None,
        })
    }
}

/// 将本地版本信息保存到 local_update_info.toml
pub fn save_local_version_info(
    info: &LocalVersionInfo,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let contents = toml::to_string_pretty(info)?;
    fs::write(LOCAL_VERSION_FILE, contents)?;
    Ok(())
}

/// 仅更新本地更新渠道（不修改版本号）
pub fn update_local_channel_only(
    channel: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut local_info = load_local_version_info()?;
    local_info.channel = channel.to_string();
    save_local_version_info(&local_info)?;
    Ok(())
}

// ============================================================================
// 交互式更新流程（CLI 模式使用）
// ============================================================================

/// 交互式更新执行函数（含用户确认）
/// 1. 显示待更新文件列表
/// 2. 询问用户是否继续
/// 3. 逐文件下载、验证哈希、替换
/// 4. 对不支持 OS/Arch 的场景尝试压缩包解压
pub async fn prompt_and_perform_update(
    all_updates_needed: HashMap<String, String>,
    base_path: &Path,
    base_url_prefix_for_download: &str,
    file_base_url: &str,
    target_binary: Option<&BinaryEntry>,
    detected_os: &str,
    detected_arch: &str,
    manifest: &Manifest,
    target_version_str: &str,
    channel: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let os_arch_supported = target_binary.is_some();

    if os_arch_supported {
        // 显示待更新列表
        println!(
            "\n检测到更新 (目标版本: {}, 渠道: {}, OS/Arch: {}):",
            target_version_str,
            channel,
            if os_arch_supported {
                "支持"
            } else {
                "不支持"
            }
        );
        if let Some(bin) = target_binary {
            if all_updates_needed.contains_key(&bin.path) {
                println!(" - 应用程序: {}", bin.path);
            }
        }
        for (rel_path, _) in &all_updates_needed {
            if let Some(bin) = target_binary {
                if rel_path != &bin.path {
                    println!(" - 其他文件: {}", rel_path);
                }
            } else {
                println!(" - 文件: {}", rel_path);
            }
        }

        // 询问用户确认
        print!("\n是否要下载并安装这些更新? (y/N): ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().to_lowercase().starts_with('y') {
            println!("用户取消了更新。");
            return Ok(());
        }

        // 下载所有需要更新的文件
        let mut downloaded_files: HashMap<String, PathBuf> = HashMap::new();
        for (rel_path, expected_hash) in &all_updates_needed {
            // 构建文件下载 URL（区分稳定版和 Beta 版）
            let version_part = base_url_prefix_for_download
                .strip_suffix('/')
                .and_then(|s| s.rsplit_once('/'))
                .map(|(_, ver)| ver);

            let file_url = if channel == "beta" {
                format!("{}/{}", file_base_url.trim_end_matches('/'), rel_path)
            } else if let Some(ver) = version_part {
                format!(
                    "{}/{}/{}",
                    file_base_url.trim_end_matches('/'),
                    ver,
                    rel_path
                )
            } else {
                format!(
                    "{}/{}/{}",
                    file_base_url.trim_end_matches('/'),
                    target_version_str,
                    rel_path
                )
            };

            println!("正在下载 {}...", rel_path);
            let temp_file_path = download_file(&file_url, rel_path).await?;

            // 验证哈希
            let downloaded_hash = calculate_file_sha256(&temp_file_path).await?;
            if downloaded_hash != *expected_hash {
                return Err(format!(
                    "下载的文件 {} 哈希值验证失败。本地: {}, 清单: {}",
                    rel_path, downloaded_hash, expected_hash
                )
                .into());
            }
            downloaded_files.insert(rel_path.clone(), temp_file_path);
        }

        // 安装所有下载的文件
        for (rel_path, temp_path) in downloaded_files {
            let target_path = base_path.join(&rel_path);
            if let Some(parent) = target_path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| format!("创建目录 '{}' 失败: {}", parent.display(), e))?;
            }
            if let Some(bin) = target_binary {
                if rel_path == bin.path {
                    // 特殊处理二进制文件名的平台适配
                    let final_binary_name = determine_final_binary_name(&bin.path, detected_os)?;
                    let final_target_path = base_path.join(&final_binary_name);
                    replace_file_or_prompt(temp_path.as_path(), final_target_path.as_path())
                        .await?;
                } else {
                    replace_file_or_prompt(temp_path.as_path(), target_path.as_path()).await?;
                }
            } else {
                replace_file_or_prompt(temp_path.as_path(), target_path.as_path()).await?;
            }
        }
    } else {
        // 没有匹配的二进制文件，尝试使用压缩包更新
        println!(
            "\n检测到更新 (目标版本: {}, 渠道: {}, OS/Arch: {}):",
            target_version_str,
            channel,
            if os_arch_supported {
                "支持"
            } else {
                "不支持"
            }
        );
        println!(
            " - 无法为当前系统 ({} {}) 找到专用的二进制文件。",
            detected_os, detected_arch
        );
        if let Some(ref pkg) = manifest.compressed_package {
            println!(" - 发现压缩包: {} (将尝试下载并解压)", pkg.path);
        } else {
            return Err("当前系统不受支持，且服务器上未找到可用的压缩包。".into());
        }

        print!("\n是否要下载并解压该压缩包以进行更新? (y/N): ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().to_lowercase().starts_with('y') {
            println!("用户取消了更新。");
            return Ok(());
        }

        if let Some(ref pkg) = manifest.compressed_package {
            let version_part = base_url_prefix_for_download
                .strip_suffix('/')
                .and_then(|s| s.rsplit_once('/'))
                .map(|(_, ver)| ver);

            let file_url = if channel == "beta" {
                format!("{}/{}", file_base_url.trim_end_matches('/'), pkg.path)
            } else if let Some(ver) = version_part {
                format!(
                    "{}/{}/{}",
                    file_base_url.trim_end_matches('/'),
                    ver,
                    pkg.path
                )
            } else {
                format!(
                    "{}/{}/{}",
                    file_base_url.trim_end_matches('/'),
                    target_version_str,
                    pkg.path
                )
            };

            println!("正在下载压缩包 {}...", pkg.path);
            let temp_pkg_path = download_file(&file_url, &pkg.path).await?;

            // 验证压缩包哈希
            let downloaded_hash = calculate_file_sha256(&temp_pkg_path).await?;
            if downloaded_hash != pkg.hash {
                return Err(format!(
                    "下载的压缩包 {} 哈希值验证失败。本地: {}, 清单: {}",
                    pkg.path, downloaded_hash, pkg.hash
                )
                .into());
            }

            // 使用外部 7z 工具解压
            decompress_package_with_external_tool(&temp_pkg_path, base_path).await?;
            std::fs::remove_file(&temp_pkg_path)?;
        }
    }

    // 更新本地版本信息
    if os_arch_supported {
        save_local_version_info(&LocalVersionInfo {
            version: Some(target_version_str.to_string()),
            major_version: None,
            channel: channel.to_string(),
            last_checked_version: Some(target_version_str.to_string()),
            last_checked_major: manifest.major_version,
            data_version: None, // data 版本由独立流程维护
        })?;
    }

    Ok(())
}

/// 使用外部 7z 工具解压压缩包
async fn decompress_package_with_external_tool(
    package_path: &Path,
    target_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let output = Command::new("7z")
        .arg("x")
        .arg("-y")
        .arg("-o")
        .arg(target_dir)
        .arg(package_path)
        .output()?;

    if !output.status.success() {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        return Err(format!("解压命令失败: {}", stderr_str).into());
    }

    Ok(())
}
