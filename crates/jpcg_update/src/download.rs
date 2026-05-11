// src/download.rs
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

// --- 常量定义 ---
pub const MANIFEST_FILENAME: &str = "manifest.toml";
pub const DEFAULT_BINARY_NAME: &str = "JPCG";
// 修改：本地版本信息文件名改为 local_update_info.toml
const LOCAL_VERSION_FILE: &str = "local_update_info.toml";

// --- 数据结构 ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: Option<String>,
    pub major_version: Option<u32>,
    pub binaries: Vec<BinaryEntry>,
    pub files: Option<Vec<FileEntry>>,
    pub compressed_package: Option<CompressedPackageEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryEntry {
    pub path: String,
    pub os: String,
    pub arch: String,
    pub hash: String,
    #[serde(rename = "hash_type")]
    pub hash_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub hash: String,
    #[serde(rename = "hash_type")]
    pub hash_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedPackageEntry {
    pub path: String,
    pub hash: String,
    #[serde(rename = "hash_type")]
    pub hash_type: String,
}

// 服务器 update.toml 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTomlInfo {
    pub version: String,           // 例如 "v1.1.251222"
    pub major_version: Option<u32>, // 例如 Some(1)
    // 可以添加其他字段，如 release_date, download_url 等
}

// 本地版本信息结构，可能包含更多细节
#[derive(Debug, Serialize, Deserialize)]
pub struct LocalVersionInfo {
    pub version: Option<String>,      // 本地安装的版本
    pub major_version: Option<u32>,   // 本地安装的主版本号
    pub channel: String,              // 更新渠道 ("stable", "beta")
    pub last_checked_version: Option<String>, // 上次检查到的服务器版本 (可选)
    pub last_checked_major: Option<u32>,      // 上次检查到的服务器主版本 (可选)
}

#[derive(Debug)]
pub struct VersionDirectory {
    pub dir_name: String,
    pub manifest: Manifest,
}

// --- 共享功能函数 ---

pub async fn fetch_latest_version_info(base_url: &str) -> Result<Option<UpdateTomlInfo>, Box<dyn std::error::Error>> {
    let update_info_url = format!("{}/{}", base_url.trim_end_matches('/'), "update.toml");

    let client = reqwest::Client::new();
    let response = client.get(&update_info_url).send().await?;

    if !response.status().is_success() {
        eprintln!("警告: 无法获取 update.toml: HTTP {}", response.status());
        return Ok(None);
    }

    let toml_text = response.text().await?;
    let update_info: UpdateTomlInfo = toml::from_str(&toml_text)?;
    Ok(Some(update_info))
}

pub async fn fetch_all_version_directories(base_url: &str) -> Result<Vec<VersionDirectory>, Box<dyn std::error::Error>> {
    let body = reqwest::get(base_url).await?.text().await?;
    let re = Regex::new(r#"(?i)<a\s+[^>]*href\s*=\s*["']([^"'/\s>]+)[^>]*>"#)?;
    let mut version_dirs = Vec::new();

    for cap in re.captures_iter(&body) {
        if let Some(version_match) = cap.get(1) {
            let dir_name = version_match.as_str();
            println!("Debug: Found potential link text: {}", dir_name);
            if dir_name.starts_with('v') && Regex::new(r"^v\d+\.\d+\.\d+$").unwrap().is_match(dir_name) {
                let manifest_url = format!("{}/{}/{}", base_url.trim_end_matches('/'), dir_name, MANIFEST_FILENAME);
                match download_and_parse_manifest(&manifest_url).await {
                    Ok(manifest) => {
                        version_dirs.push(VersionDirectory { dir_name: dir_name.to_string(), manifest });
                    }
                    Err(e) => {
                        eprintln!("警告: 无法加载版本目录 '{}' 的清单: {}", dir_name, e);
                    }
                }
            } else {
                println!("Debug: Link text '{}' does not match version format.", dir_name);
            }
        }
    }

    version_dirs.sort_by(|a, b| b.dir_name.cmp(&a.dir_name));
    Ok(version_dirs)
}

pub fn find_latest_version_in_major(versions: &[VersionDirectory], target_major: u32) -> Result<Option<&VersionDirectory>, Box<dyn std::error::Error>> {
    let filtered: Vec<&VersionDirectory> = versions.iter()
        .filter(|v| v.manifest.major_version == Some(target_major))
        .collect();

    if filtered.is_empty() { Ok(None) } else { Ok(filtered.first().copied()) }
}

pub fn select_target_binary<'a>(binaries: &'a [BinaryEntry], target_os: &str, target_arch: &str) -> Result<&'a BinaryEntry, Box<dyn std::error::Error>> {
    for entry in binaries {
        if entry.os == target_os && entry.arch == target_arch {
            return Ok(entry);
        }
    }
    Err(format!("未找到适用于 {} {} 的二进制文件。", target_os, target_arch).into())
}

pub async fn check_binary_update_needed(base_path: &Path, target_binary_entry: &BinaryEntry) -> Result<bool, Box<dyn std::error::Error>> {
    let final_binary_name = determine_final_binary_name(&target_binary_entry.path, env::consts::OS)?;
    let local_binary_path = base_path.join(final_binary_name);

    if !local_binary_path.exists() {
        println!("本地可执行文件 {} 不存在，需要下载。", local_binary_path.display());
        return Ok(true);
    }

    let local_hash = calculate_file_sha256(&local_binary_path).await?;
    if local_hash != target_binary_entry.hash {
        println!("本地可执行文件 {} 哈希值不匹配，需要更新。", local_binary_path.display());
        Ok(true)
    } else { Ok(false) }
}

// 在 download.rs 文件中找到 determine_other_updates_by_hash 函数
pub async fn determine_other_updates_by_hash(
    base_path: &Path,
    manifest: &Manifest,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut updates_needed = HashMap::new();

    // 修正：检查 files 是否为 Some(Vec) 再进行迭代
    if let Some(files_list) = &manifest.files {
        for file_entry in files_list {
            let local_file_path = base_path.join(&file_entry.path);

            if !local_file_path.exists() {
                println!("本地文件 {} 不存在，添加到更新列表。", local_file_path.display());
                updates_needed.insert(file_entry.path.clone(), file_entry.hash.clone());
                continue;
            }

            let local_hash = calculate_file_sha256(&local_file_path).await?;
            if local_hash != file_entry.hash {
                println!("文件 {} 哈希值不匹配。本地: {}, 期望: {}。添加到更新列表。",
                         local_file_path.display(), local_hash, file_entry.hash);
                updates_needed.insert(file_entry.path.clone(), file_entry.hash.clone());
            }
        }
    } else {
        // 如果清单中没有 files 字段，打印一条信息
        println!("清单中未定义额外文件列表。");
    }

    Ok(updates_needed)
}

pub async fn download_and_parse_manifest(manifest_url: &str) -> Result<Manifest, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(manifest_url).send().await?;
    if !response.status().is_success() {
        return Err(format!("下载清单失败，HTTP 状态码: {}", response.status()).into());
    }
    let toml_text = response.text().await?;
    let manifest: Manifest = toml::from_str(&toml_text)?;

    if let Some(pkg) = &manifest.compressed_package
        && pkg.hash_type != "SHA256" {
            eprintln!("警告: 压缩包 '{}' 的哈希类型 '{}' 不受支持。期望 SHA256。", pkg.path, pkg.hash_type);
        }
    for binary_entry in &manifest.binaries {
        if binary_entry.hash_type != "SHA256" {
            eprintln!("警告: 二进制文件 '{}' 的哈希类型 '{}' 不受支持。期望 SHA256。", binary_entry.path, binary_entry.hash_type);
        }
    }
    if let Some(files_list) = &manifest.files {
        for file_entry in files_list {
            if file_entry.hash_type != "SHA256" {
                eprintln!("警告: 文件 '{}' 的哈希类型 '{}' 不受支持。期望 SHA256。", file_entry.path, file_entry.hash_type);
            }
        }
    }
    Ok(manifest)
}

pub async fn calculate_file_sha256(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let n = tokio::io::AsyncReadExt::read(&mut file, &mut buffer).await?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }

    let hash_result = hasher.finalize();
    Ok(hex::encode(hash_result))
}

// ... (其他代码不变) ...

pub async fn download_file(url: &str, file_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    // 添加 User-Agent 头，模拟浏览器
    let mut response = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send().await?;
        
    if !response.status().is_success() {
        return Err(format!("下载文件失败，HTTP 状态码: {}", response.status()).into());
    }

    let total_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
        .progress_chars("#>-"));
    pb.set_message(format!("正在下载 {}", file_name));

    let temp_file = tempfile::NamedTempFile::new()?;
    // 获取 TempPath
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

    // 检查文件大小
    let final_size = tokio::fs::metadata(&temp_path).await?.len();

    if final_size == 0 {
        // 如果文件为空，删除临时文件并返回错误
        std::fs::remove_file(&temp_path)?;
        return Err(format!("下载的文件 {} 为空。", file_name).into());
    }

    // 检查大小是否匹配（仅当 header 中提供了 size 时）
    if total_size > 0 && final_size != total_size {
        println!("警告: 下载的文件 {} 大小与预期不符。预期: {} B, 实际: {} B", file_name, total_size, final_size);
        // 可以选择返回错误，或者继续（取决于你的需求）
        // std::fs::remove_file(&temp_path)?; // 删除文件并返回错误
        // return Err(format!("下载的文件 {} 大小与预期不符。预期: {} B, 实际: {} B", file_name, total_size, final_size).into());
    }

    // 重要：调用 keep()，防止临时文件在离开作用域时被删除
    // into_temp_path().keep() 会将临时文件保留在磁盘上，并返回其 PathBuf
    let kept_path = temp_path.keep().map_err(|e| format!("无法保留临时文件: {}", e.error))?;
    Ok(kept_path)
}

pub async fn replace_file_or_prompt(from: &Path, to: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if to.exists() {
        println!("正在替换现有文件: {}", to.display());
    } else {
        println!("正在创建新文件: {}", to.display());
    }

    // 确保目标文件的父目录存在
    if let Some(parent) = to.parent() {
        // 使用 tokio::fs::create_dir_all 是异步的
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            format!("创建目录 '{}' 失败: {}", parent.display(), e)
        })?;
    }

    // 使用 tokio::fs::copy 进行异步文件复制
    tokio::fs::copy(from, to).await.map_err(|e| {
        format!("复制文件 '{}' 到 '{}' 失败: {}", from.display(), to.display(), e)
    })?;

    println!("文件已复制/替换: {}", to.display());
    Ok(())
}

// 在 download.rs 文件中修改 determine_final_binary_name 函数
pub fn determine_final_binary_name(server_path: &str, os: &str) -> Result<String, Box<dyn std::error::Error>> {

    // 不再使用 Path::file_stem 和 Path::extension
    // 直接处理字符串

    let final_name = match os {
        "windows" => {
            // 检查是否已经以 .exe 结尾
            if server_path.ends_with(".exe") {
                server_path.to_string()
            } else {
                format!("{}.exe", server_path)
            }
        }
        _ => { // Linux, macOS
            // 对于 macOS/Linux，通常不带扩展名
            // 但我们要确保不截断包含多个 . 的文件名
            // 直接返回 server_path (没有扩展名的情况下)
            // 如果 server_path 有 .exe 扩展名（从 Windows 包复制过来），则移除它
            if server_path.ends_with(".exe") {
                server_path.strip_suffix(".exe").unwrap_or(server_path).to_string()
            } else {
                server_path.to_string() // 直接返回，例如 "JPCG-arm64-Darwin-macOS-v1.1.251222"
            }
        }
    };

    println!("Debug: determine_final_binary_name returning: '{}'", final_name);
    Ok(final_name)
}

// 修改：从 LOCAL_VERSION_FILE (local_update_info.toml) 加载本地版本信息
pub fn load_local_version_info() -> Result<LocalVersionInfo, Box<dyn std::error::Error>> {
    let path = Path::new(LOCAL_VERSION_FILE);
    if path.exists() {
        let contents = fs::read_to_string(path)?;
        let info: LocalVersionInfo = toml::from_str(&contents)?;
        Ok(info)
    } else {
        // 如果文件不存在，返回默认值
        Ok(LocalVersionInfo { version: None, major_version: None, channel: "stable".to_string(), last_checked_version: None, last_checked_major: None })
    }
}

// 修改：将本地版本信息保存到 LOCAL_VERSION_FILE (local_update_info.toml)
pub fn save_local_version_info(info: &LocalVersionInfo) -> Result<(), Box<dyn std::error::Error>> {
    let contents = toml::to_string_pretty(info)?;
    fs::write(LOCAL_VERSION_FILE, contents)?;
    Ok(())
}

// 在 download.rs 文件中找到 prompt_and_perform_update 函数
// 在 download.rs 文件中找到 prompt_and_perform_update 函数
// 在 download.rs 文件中找到 prompt_and_perform_update 函数
pub async fn prompt_and_perform_update(
    all_updates_needed: HashMap<String, String>,
    base_path: &Path,
    base_url_prefix_for_download: &str, // 用于下载清单的 URL 前缀 (通常是版本目录，例如 https://.../updates/JPCG/v1.1.251222/ 或 https://.../updates/JPCG_beta/)
    file_base_url: &str, // 新增：用于下载具体文件的 URL 基础路径 (例如 https://nefinita-ai.com/files/JPCG/ 或 https://nefinita-ai.com/files/JPCG_beta/)
    target_binary: Option<&BinaryEntry>,
    detected_os: &str,
    detected_arch: &str,
    manifest: &Manifest,
    target_version_str: &str, // 这里是传入的 target_version_str
    channel: &str, // 这是本次更新的渠道 ("stable" 或 "beta")
) -> Result<(), Box<dyn std::error::Error>> {
    let os_arch_supported = target_binary.is_some();

    // 添加调试信息
    println!("Debug: prompt_and_perform_update received target_version_str: {}", target_version_str);
    println!("Debug: prompt_and_perform_update received channel: {}", channel);
    println!("Debug: prompt_and_perform_update received base_url_prefix_for_download: {}", base_url_prefix_for_download);
    println!("Debug: prompt_and_perform_update received file_base_url: {}", file_base_url);

    if os_arch_supported {
        // 修正：这里缺少了 os_arch_supported 参数
        println!("\n检测到更新 (目标版本: {}, 渠道: {}, OS/Arch supported: {}):", target_version_str, channel, if os_arch_supported { "Yes" } else { "No" });
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

        print!("\n是否要下载并安装这些更新? (y/N): ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().to_lowercase().starts_with('y') {
            println!("用户取消了更新。");
            return Ok(());
        }

        let mut downloaded_files: HashMap<String, PathBuf> = HashMap::new();
        for (rel_path, expected_hash) in &all_updates_needed {
            // 修正：使用 file_base_url 构建下载 URL
            // 文件 URL 格式:
            // - 稳定版: https://nefinita-ai.com/files/JPCG/{version}/{rel_path}
            // - Beta版: https://nefinita-ai.com/files/JPCG_beta/{rel_path} (注意没有版本号)
            // 从 base_url_prefix_for_download 中尝试提取版本号 (仅稳定版需要)
            let version_part = base_url_prefix_for_download.strip_suffix('/').and_then(|s| s.rsplit_once('/')).map(|(_, ver)| ver);

            // 添加调试信息
            println!("Debug: Extracted version_part from base_url_prefix: {:?}", version_part);
            println!("Debug: Using target_version_str as fallback: {}", target_version_str);

            // 修正：区分稳定版和 Beta 版的 URL 构建逻辑
            let file_url = if channel == "beta" {
                // Beta 版：file_base_url 已经包含了版本目录（JPCG_beta），直接拼接 rel_path
                format!("{}/{}", file_base_url.trim_end_matches('/'), rel_path)
            } else {
                // 稳定版：尝试从 base_url_prefix 提取版本号，否则使用 target_version_str
                if let Some(ver) = version_part {
                    format!("{}/{}/{}", file_base_url.trim_end_matches('/'), ver, rel_path)
                } else {
                    format!("{}/{}/{}", file_base_url.trim_end_matches('/'), target_version_str, rel_path)
                }
            };

            println!("正在下载 {}... (URL: {})", rel_path, file_url); // 添加 URL 调试信息
            let temp_file_path = download_file(&file_url, rel_path).await?;
            let downloaded_hash = calculate_file_sha256(&temp_file_path).await?;
            if downloaded_hash != *expected_hash {
                return Err(format!("下载的文件 {} 哈希值验证失败。本地: {}, 清单: {}", rel_path, downloaded_hash, expected_hash).into());
            } else {
                println!("文件 {} 下载并验证成功。", rel_path);
            }
            downloaded_files.insert(rel_path.clone(), temp_file_path);
        }

        for (rel_path, temp_path) in downloaded_files {
            let target_path = base_path.join(&rel_path);
            if let Some(parent) = target_path.parent() { 
                tokio::fs::create_dir_all(parent).await.map_err(|e| {
                    format!("创建目录 '{}' 失败: {}", parent.display(), e)
                })?;
            }
            if let Some(bin) = target_binary {
                if rel_path == bin.path {
                    let final_binary_name = determine_final_binary_name(&bin.path, detected_os)?;
                    let final_target_path = base_path.join(&final_binary_name);
                    replace_file_or_prompt(temp_path.as_path(), final_target_path.as_path()).await?;
                } else {
                    replace_file_or_prompt(temp_path.as_path(), target_path.as_path()).await?;
                }
            } else {
                replace_file_or_prompt(temp_path.as_path(), target_path.as_path()).await?;
            }
        }

    } else {
        // 修正：同样缺少 os_arch_supported 参数
        println!("\n检测到更新 (目标版本: {}, 渠道: {}, OS/Arch supported: {}):", target_version_str, channel, if os_arch_supported { "Yes" } else { "No" });
        println!(" - 无法为当前系统 ({} {}) 找到专用的二进制文件。", detected_os, detected_arch);
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
             // 修正：使用 file_base_url 构建压缩包下载 URL
             // 与文件下载逻辑类似，区分稳定版和 Beta 版
             let file_url = if channel == "beta" {
                 // Beta 版：file_base_url 已经包含了版本目录（JPCG_beta），直接拼接 pkg.path
                 format!("{}/{}", file_base_url.trim_end_matches('/'), pkg.path)
             } else {
                 // 稳定版：尝试从 base_url_prefix 提取版本号，否则使用 target_version_str
                 let version_part = base_url_prefix_for_download.strip_suffix('/').and_then(|s| s.rsplit_once('/')).map(|(_, ver)| ver);
                 if let Some(ver) = version_part {
                     format!("{}/{}/{}", file_base_url.trim_end_matches('/'), ver, pkg.path)
                 } else {
                     format!("{}/{}/{}", file_base_url.trim_end_matches('/'), target_version_str, pkg.path)
                 }
             };
             
             println!("正在下载压缩包 {}... (URL: {})", pkg.path, file_url); // 添加 URL 调试信息
             let temp_pkg_path = download_file(&file_url, &pkg.path).await?;
             let downloaded_hash = calculate_file_sha256(&temp_pkg_path).await?;
             if downloaded_hash != pkg.hash {
                 return Err(format!("下载的压缩包 {} 哈希值验证失败。本地: {}, 清单: {}", pkg.path, downloaded_hash, pkg.hash).into());
             } else {
                 println!("压缩包 {} 下载并验证成功。", pkg.path);
             }

             decompress_package_with_external_tool(&temp_pkg_path, base_path).await?;
             std::fs::remove_file(&temp_pkg_path)?;
        } else {
             return Err("清单中未定义压缩包，无法为不支持的系统提供更新。".into());
        }
    }

    if os_arch_supported {
        // 修正：在保存本地版本信息时，使用本次更新的 channel
        save_local_version_info(&LocalVersionInfo {
            version: Some(target_version_str.to_string()),
            major_version: None, // 或者从 manifest 中获取
            channel: channel.to_string(), // 使用本次更新的渠道
            last_checked_version: Some(target_version_str.to_string()),
            last_checked_major: manifest.major_version,
        })?;
    } else {
        println!("注意: 使用压缩包更新，本地版本信息未更新。");
        // 注意：使用压缩包时，本地版本信息可能没有更新，所以 channel 也不会改变。
        // 如果你想在使用压缩包更新后也改变 channel，需要在这里也调用 save_local_version_info
        // 但需要知道压缩包里的版本号
        // 例如，假设你知道压缩包的版本是 target_version_str
        // save_local_version_info(&LocalVersionInfo {
        //     version: Some(target_version_str.to_string()),
        //     major_version: None,
        //     channel: channel.to_string(), // 使用本次更新的渠道
        //     last_checked_version: Some(target_version_str.to_string()),
        //     last_checked_major: manifest.major_version, // manifest 里可能没有，需要从其他途径获取
        // })?;
    }

    Ok(())
}

async fn decompress_package_with_external_tool(package_path: &Path, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("正在使用外部工具解压 {} 到 {}...", package_path.display(), target_dir.display());

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

    println!("解压成功完成。");
    Ok(())
}

pub fn update_local_channel_only(channel: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut local_info = load_local_version_info()?; // 加载当前本地信息
    local_info.channel = channel.to_string(); // 更新渠道信息
    // 可以选择性地更新 last_checked_channel 或其他字段
    save_local_version_info(&local_info)?; // 保存修改后的本地信息
    Ok(())
}