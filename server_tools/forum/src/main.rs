use axum::{
    Router,
    body::Body,
    extract::{Multipart, Path, State},
    http::{Response, StatusCode, header},
    response::{Html, Json},
    routing::{get, post},
};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tower_http::cors::CorsLayer;

struct AppState {
    data_dir: PathBuf,
}

#[derive(Serialize)]
struct FileInfo {
    name: String,
    size: u64,
    modified: String,
}

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>JX3 PVP 数据分享论坛</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #f5f5f5; color: #333; }
.container { max-width: 800px; margin: 0 auto; padding: 20px; }
.header { background: #1a1a2e; color: #fff; padding: 20px; border-radius: 8px 8px 0 0; text-align: center; }
.header h1 { font-size: 24px; }
.header p { font-size: 14px; opacity: 0.8; margin-top: 8px; }
.card { background: #fff; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); margin-top: 16px; overflow: hidden; }
.card-header { background: #f8f9fa; padding: 12px 16px; border-bottom: 1px solid #eee; font-weight: 600; font-size: 16px; }
.card-body { padding: 16px; }
.upload-area { border: 2px dashed #ccc; border-radius: 8px; padding: 30px; text-align: center; cursor: pointer; transition: border-color 0.3s; }
.upload-area:hover { border-color: #1a1a2e; }
.upload-area input[type="file"] { display: none; }
.upload-area label { display: block; cursor: pointer; }
.upload-area .icon { font-size: 40px; color: #999; margin-bottom: 10px; }
.upload-area .text { color: #666; font-size: 14px; }
.btn { display: inline-block; padding: 8px 20px; border: none; border-radius: 4px; cursor: pointer; font-size: 14px; transition: background 0.2s; }
.btn-primary { background: #1a1a2e; color: #fff; margin-top: 12px; }
.btn-primary:hover { background: #2d2d5e; }
.btn-download { background: #0d6efd; color: #fff; text-decoration: none; font-size: 13px; padding: 4px 12px; border-radius: 3px; }
.btn-download:hover { background: #0b5ed7; }
.tabs { display: flex; gap: 8px; margin-bottom: 12px; flex-wrap: wrap; }
.tab { padding: 6px 16px; border-radius: 4px; border: 1px solid #ccc; background: #fff; cursor: pointer; font-size: 14px; transition: all 0.2s; }
.tab:hover { background: #eef; }
.tab.active { background: #1a1a2e; color: #fff; border-color: #1a1a2e; }
.category-select { margin-bottom: 12px; }
.category-select label { font-size: 14px; margin-right: 8px; }
.category-select select { padding: 6px 12px; border-radius: 4px; border: 1px solid #ccc; font-size: 14px; }
table { width: 100%; border-collapse: collapse; }
th, td { padding: 10px 12px; text-align: left; border-bottom: 1px solid #eee; font-size: 14px; }
th { background: #f8f9fa; font-weight: 600; color: #555; }
tr:hover td { background: #f8f9fa; }
.empty { text-align: center; color: #999; padding: 30px; font-size: 14px; }
.notice { background: #fff3cd; border: 1px solid #ffc107; border-radius: 6px; padding: 12px 16px; margin-top: 16px; font-size: 13px; color: #856404; }
.notice strong { font-weight: 600; }
</style>
</head>
<body>
<div class="container">
<div class="header">
 <h1>JX3 PVP 数据分享论坛</h1>
<p>上传和分享剑网3 PVP 计算器数据文件（.toml）</p>
</div>

<div class="card">
<div class="card-header">上传数据文件</div>
<div class="card-body">
<div class="category-select">
<label for="uploadCategory">分类：</label>
<select id="uploadCategory"></select>
</div>
<form id="uploadForm" enctype="multipart/form-data">
<div class="upload-area" id="dropArea">
<input type="file" name="file" id="fileInput" accept=".toml">
<label for="fileInput">
<div class="icon">&#128194;</div>
<div class="text">点击选择或拖拽 .toml 文件到此处</div>
</label>
</div>
<button type="submit" class="btn btn-primary" id="uploadBtn">上传</button>
</form>
<div id="uploadMsg" style="margin-top: 10px; font-size: 13px;"></div>
</div>
</div>

<div class="card">
<div class="card-header">已上传的文件</div>
<div class="card-body">
<div class="tabs" id="categoryTabs"></div>
<table>
<thead><tr><th>文件名</th><th>大小</th><th>上传时间</th><th>操作</th></tr></thead>
<tbody id="fileList"><tr><td colspan="4" class="empty">加载中...</td></tr></tbody>
</table>
</div>
</div>

<div class="notice">
<strong>提示：</strong>文件按分类（shuxing/combo等）组织。下载后放入计算器的对应 <code>data/{分类}/</code> 目录中使用。
</div>
</div>

<script>
let currentCategory = 'shuxing';

async function loadCategories() {
    try {
        const r = await fetch('/api/categories');
        const cats = await r.json();
        const tabs = document.getElementById('categoryTabs');
        const sel = document.getElementById('uploadCategory');
        if (cats.length === 1) { tabs.innerHTML = ''; return; }
        tabs.innerHTML = cats.map(c => `<button class="tab${c === currentCategory ? ' active' : ''}" onclick="switchCategory('${c}')">${c}</button>`).join('');
        sel.innerHTML = cats.map(c => `<option value="${c}"${c === currentCategory ? ' selected' : ''}>${c}</option>`).join('');
    } catch (_) {}
}

async function switchCategory(cat) {
    currentCategory = cat;
    document.getElementById('uploadCategory').value = cat;
    loadCategories();
    loadFiles();
}

async function loadFiles() {
    try {
        const r = await fetch('/api/files/' + currentCategory);
        const files = await r.json();
        const tbody = document.getElementById('fileList');
        if (!files.length) { tbody.innerHTML = '<tr><td colspan="4" class="empty">暂无上传的文件</td></tr>'; return; }
        tbody.innerHTML = files.map(f => `<tr><td>${f.name}</td><td>${(f.size / 1024).toFixed(1)} KB</td><td>${f.modified}</td><td><a href="/download/${currentCategory}/${f.name}" class="btn-download" download>下载</a></td></tr>`).join('');
    } catch (_) { document.getElementById('fileList').innerHTML = '<tr><td colspan="4" class="empty">加载失败</td></tr>'; }
}

const fileInput = document.getElementById('fileInput');
const dropArea = document.getElementById('dropArea');
dropArea.addEventListener('dragover', e => { e.preventDefault(); dropArea.style.borderColor = '#1a1a2e'; });
dropArea.addEventListener('dragleave', () => { dropArea.style.borderColor = '#ccc'; });
dropArea.addEventListener('drop', e => { e.preventDefault(); dropArea.style.borderColor = '#ccc'; if (e.dataTransfer.files.length) fileInput.files = e.dataTransfer.files; });

document.getElementById('uploadForm').addEventListener('submit', async e => {
    e.preventDefault();
    if (!fileInput.files.length) { document.getElementById('uploadMsg').textContent = '请先选择一个文件'; return; }
    const fd = new FormData();
    fd.append('file', fileInput.files[0]);
    fd.append('category', currentCategory);
    const btn = document.getElementById('uploadBtn');
    const msg = document.getElementById('uploadMsg');
    btn.disabled = true; btn.textContent = '上传中...';
    msg.textContent = '';
    try {
        const r = await fetch('/upload', { method: 'POST', body: fd });
        const d = await r.json();
        if (r.ok) { msg.style.color = 'green'; msg.textContent = d.message || '上传成功'; fileInput.value = ''; loadFiles(); }
        else { msg.style.color = 'red'; msg.textContent = d.error || '上传失败'; }
    } catch (e) { msg.style.color = 'red'; msg.textContent = '网络错误'; }
    finally { btn.disabled = false; btn.textContent = '上传'; }
});

loadCategories();
loadFiles();
</script>
</body>
</html>"#;

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let data_dir =
        PathBuf::from(std::env::var("FORUM_DATA_DIR").unwrap_or_else(|_| "forum_data".to_string()));
    fs::create_dir_all(&data_dir).expect("无法创建数据目录");

    let data_dir_display = data_dir.display().to_string();
    let state = Arc::new(AppState { data_dir });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/categories", get(categories_handler))
        .route("/api/files/{category}", get(list_files_handler))
        .route("/upload", post(upload_handler))
        .route("/download/{category}/{filename}", get(download_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("🚀 论坛服务启动: http://{}", addr);
    println!("📁 数据目录: {}", data_dir_display);
    println!("💡 访问 http://localhost:{} 打开论坛", port);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("无法绑定端口");
    axum::serve(listener, app).await.expect("服务器启动失败");
}

async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn categories_handler(State(state): State<Arc<AppState>>) -> Json<Vec<String>> {
    let mut cats = Vec::new();
    if let Ok(entries) = fs::read_dir(&state.data_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir()
                && let Some(name) = entry.file_name().to_str()
                && !name.starts_with('.')
            {
                cats.push(name.to_string());
            }
        }
    }
    cats.sort();
    Json(cats)
}

async fn list_files_handler(
    State(state): State<Arc<AppState>>,
    Path(category): Path<String>,
) -> Result<Json<Vec<FileInfo>>, (StatusCode, String)> {
    let dir = state.data_dir.join(&category);
    if !dir.is_dir() {
        return Ok(Json(vec![]));
    }
    let mut files = Vec::new();
    let mut entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    while let Some(entry) = entries
        .next()
        .transpose()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        if name.starts_with('_') || name.starts_with('.') {
            continue;
        }
        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| {
                let secs = d.as_secs();
                let (y, mo, da, h, mi, s) = unix_ts_to_ymd(secs);
                format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo, da, h, mi, s)
            })
            .unwrap_or_else(|| "未知".to_string());

        files.push(FileInfo {
            name,
            size: metadata.len(),
            modified,
        });
    }

    files.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(Json(files))
}

async fn upload_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut saved = false;
    let mut upload_category = "shuxing".to_string();

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": format!("读取上传数据失败: {}", e)})),
                ));
            }
        };

        let name = field.name().unwrap_or("").to_string();
        if name == "category" {
            if let Ok(val) = field.text().await {
                let val = val.trim().to_string();
                if !val.is_empty() {
                    upload_category = val;
                }
            }
            continue;
        }
        if name != "file" {
            continue;
        }
        let filename = match field.file_name() {
            Some(f) => sanitize_filename(f),
            None => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "缺少文件名"})),
                ));
            }
        };

        if !filename.ends_with(".toml") {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "仅支持 .toml 文件"})),
            ));
        }

        let data = field.bytes().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("读取文件数据失败: {}", e)})),
            )
        })?;

        if data.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "文件内容为空"})),
            ));
        }

        let dest_dir = state.data_dir.join(&upload_category);
        fs::create_dir_all(&dest_dir).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("创建分类目录失败: {}", e)})),
            )
        })?;

        let dest = dest_dir.join(&filename);
        fs::write(&dest, &data).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("保存文件失败: {}", e)})),
            )
        })?;

        saved = true;
    }

    if saved {
        Ok(Json(serde_json::json!({"message": "上传成功"})))
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "未找到上传的文件"})),
        ))
    }
}

async fn download_handler(
    State(state): State<Arc<AppState>>,
    Path((category, filename)): Path<(String, String)>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let filename = sanitize_filename(&filename);
    if !filename.ends_with(".toml") {
        return Err((StatusCode::BAD_REQUEST, "仅支持下载 .toml 文件".to_string()));
    }

    let path = state.data_dir.join(&category).join(&filename);
    if !path.exists() {
        return Err((StatusCode::NOT_FOUND, "文件不存在".to_string()));
    }

    let data = fs::read(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("读取文件失败: {}", e),
        )
    })?;

    Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(data))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|&c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
        .collect::<String>()
        .trim_matches('.')
        .to_string()
}

fn unix_ts_to_ymd(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let h = time_secs / 3600;
    let m = (time_secs % 3600) / 60;
    let s = time_secs % 60;

    let mut y = 1970u64;
    let mut d = days;
    loop {
        let leap = (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400);
        let yd = if leap { 366 } else { 365 };
        if d < yd {
            break;
        }
        d -= yd;
        y += 1;
    }
    let leap = (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400);
    let mdays: [u64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut mo = 1u64;
    for &md in &mdays {
        if d < md {
            break;
        }
        d -= md;
        mo += 1;
    }
    (y, mo, d + 1, h, m, s)
}
