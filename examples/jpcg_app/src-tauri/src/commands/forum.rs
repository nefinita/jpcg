use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct ForumFileInfo {
    name: String,
    size: u64,
    modified: String,
}

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
