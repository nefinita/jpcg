use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub(crate) struct ForumFileInfo {
    name: String,
    size: u64,
    modified: String,
}

fn download_dir(category: &str) -> PathBuf {
    let exe = std::env::current_exe().ok();
    let exe_dir = exe.as_ref().and_then(|p| p.parent());
    let base_dir = exe_dir.map(|p| p.to_path_buf()).unwrap_or_default();

    if exe_dir.map_or(false, |d| d.ends_with("MacOS")) {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("com.qinthirteen.jpcg")
            .join(category)
    } else {
        base_dir.join("data").join(category)
    }
}

#[tauri::command]
pub async fn forum_list_files(
    forum_url: String,
    category: Option<String>,
) -> Result<Vec<ForumFileInfo>, String> {
    let cat = category.unwrap_or_else(|| "shuxing".to_string());
    let url = format!("{}/api/files/{}", forum_url.trim_end_matches('/'), cat);
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

#[tauri::command]
pub async fn forum_list_categories(forum_url: String) -> Result<Vec<String>, String> {
    let url = format!("{}/api/categories", forum_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("连接论坛失败: {}", e))?;
    let categories: Vec<String> = resp
        .json()
        .await
        .map_err(|e| format!("解析分类列表失败: {}", e))?;
    Ok(categories)
}

#[tauri::command]
pub async fn forum_download_file(
    forum_url: String,
    filename: String,
    category: Option<String>,
) -> Result<String, String> {
    if !filename.ends_with(".toml") {
        return Err("仅支持下载 .toml 文件".to_string());
    }

    let cat = category.unwrap_or_else(|| "shuxing".to_string());
    let url = format!(
        "{}/download/{}/{}",
        forum_url.trim_end_matches('/'),
        cat,
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

    let dest_dir = download_dir(&cat);
    std::fs::create_dir_all(&dest_dir).map_err(|e| format!("创建目录失败: {}", e))?;

    let dest_path = dest_dir.join(&filename);
    std::fs::write(&dest_path, &bytes).map_err(|e| format!("保存文件失败: {}", e))?;

    Ok(format!("已下载: {}", filename))
}

#[tauri::command]
pub fn forum_list_downloaded(category: Option<String>) -> Result<Vec<String>, String> {
    let cat = category.unwrap_or_else(|| "shuxing".to_string());
    let dir = download_dir(&cat);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|e| format!("读取目录失败: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".toml") {
                    files.push(name.to_string());
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

#[tauri::command]
pub fn forum_delete_downloaded(
    filename: String,
    category: Option<String>,
) -> Result<String, String> {
    if !filename.ends_with(".toml") {
        return Err("仅支持删除 .toml 文件".to_string());
    }
    let cat = category.unwrap_or_else(|| "shuxing".to_string());
    let path = download_dir(&cat).join(&filename);
    if !path.exists() {
        return Err(format!("文件不存在: {}", filename));
    }
    fs::remove_file(&path).map_err(|e| format!("删除文件失败: {}", e))?;
    Ok(format!("已删除: {}", filename))
}
