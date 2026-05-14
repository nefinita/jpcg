# AI 代码修改记录

## 1. 修复 `player` 字段类型不匹配 (api.js)

**文件**: `examples/jpcg_app/src/js/api.js`

**问题**: `_sanitizeNumbers` 将所有值转为数字，导致 `player.jcsx`（字符串 `"gengu"`）被转为 `0`，Rust 端 `PlayerConfigDTO.jcsx: String` 反序列化失败，报错 `invalid type: integer 0, expected a string`。

**修复**: 在 `_sanitizeNumbers` 中新增分支：若字符串值为非数字（如 `"gengu"`），则保留原字符串不变。

---

## 2. 实现自动更新模块 (jpcg_update)

### 2.1 `crates/jpcg_update/src/download.rs`

**新增结构体**:
- `DataFileEntry` — 数据文件条目（path, hash, hash_type）
- `DataManifest` — 数据清单（data_version + files）
- `UpdateProgressEvent` — 进度事件（stage, message, progress, file）
- `UpdateCheckResult` — 检查结果（含 app/data 版本、待更新文件列表）
- `ProgressCallback` trait — 进度回调接口

**新增常量**: `DATA_MANIFEST_FILENAME = "data_manifest.toml"`

**扩展字段**:
- `UpdateTomlInfo` 新增 `data_version: Option<String>`
- `LocalVersionInfo` 新增 `data_version: Option<String>`

**新增函数**:
- `fetch_data_manifest()` — 从服务器获取 data_manifest.toml
- `check_data_updates()` — 比对本地 data 文件哈希
- `download_file_with_progress()` — 带进度回调的下载（替代 indicatif 版）
- `download_and_install_data()` — 下载并安装 data 文件，更新本地 data_version

### 2.2 `crates/jpcg_update/src/lib.rs`

**新增程序化 API**:
- `check_updates(base_path, beta, force)` — 仅检查不下载，返回 `UpdateCheckResult`
- `download_updates(base_path, beta, check_result, progress)` — 执行下载

**保留**: `all_updates()` CLI 入口（追加 data 更新检查）

### 2.3 `server_tools/`

**新增目录**: `server_tools/`

**Cargo.toml** + `src/main.rs`: 扫描 `./data/` 目录递归计算 SHA256，生成 `data_manifest.toml`。

```
cargo run -p server_tools -- --version v2.0.2026050201 --data-dir ./data --output data_manifest.toml
```

---

## 3. Tauri 后端集成

### 3.1 `examples/jpcg_app/src-tauri/Cargo.toml`
- edition `2021` → `2024`
- 新增 `jpcg_update` 依赖

### 3.2 `examples/jpcg_app/src-tauri/src/commands/mod.rs`
- 新增 `check_update(beta, force)` 命令，返回 `UpdateCheckResult`
- 新增 `perform_update(beta, has_data_update, latest_data_version, data_files_to_update)` 命令
- `TauriProgress` 实现 `ProgressCallback`，通过 `app_handle.emit("update-progress", ...)` 推送到前端

### 3.3 `examples/jpcg_app/src-tauri/src/lib.rs`
- `generate_handler!` 注册 `check_update` / `perform_update`

---

## 4. 前端集成

### 4.1 `examples/jpcg_app/src/js/config.js`
- `TAURI_COMMANDS` 新增 `checkUpdate` / `performUpdate`

### 4.2 `examples/jpcg_app/src/js/api.js`
- `checkUpdate(beta, force)` — 调用 `check_update` 命令
- `performUpdate(beta, result)` — 调用 `perform_update` 命令
- `listenUpdateProgress(callback)` — 监听 `update-progress` 事件

### 4.3 `examples/jpcg_app/src/js/app.js`
- 启动时自动调用 `autoCheckUpdate()` → 发现 data 更新则 Toast 通知 + 按钮脉冲闪烁
- `btn-update` 点击后：检查 → 列文件列表 → confirm() 确认 → 下载 → 进度条 + 文件名实时显示
- 下载完成后重置按钮

### 4.4 `examples/jpcg_app/src/index.html`
- action-bar 中新增 `#update-progress` (进度条 + 文本)

### 4.5 `examples/jpcg_app/src/css/components.css`
- 新增 `.update-progress` 相关样式
- 新增 `.btn-updating` 按钮状态样式 + `@keyframes gradientShift`

---

## 5. Workspace 配置

### `Cargo.toml` (项目根)
- workspace members 新增 `"server_tools"`
- workspace dependencies 新增 `jpcg_update = { path = "crates/jpcg_update" }`
- 所有 crate edition 统一为 `2024`

---

## 6. 修复计算按钮卡死 & 后端 Panic

### `crates/jpcg_core/src/cal.rs`

**问题**: `start_calculation()` 中 3 处 `.unwrap()` / `.expect()` 在文件缺失或路径异常时 panic，导致 Tauri 命令无响应，前端按钮卡死。

**修复**:
- line 22-24: `path.parent().expect(...)` → `match` 返回 `Err`
- line 34: `file_path.to_str().unwrap()` → `match` + 返回 `Err`
- line 37: `toml::from_str(...).unwrap()` → `match` + 返回 `Err`

### `crates/jpcg_core/src/io.rs`

**问题**: `load_config()` 和 `save_config()` 中存在 `.unwrap()` / `.expect()`。

**修复**:
- line 53-55: `.expect("Failed to get parent directory")` → `map(..).unwrap_or_default()`
- line 62: `file_path.to_str().unwrap()` → `unwrap_or("")` + 空值检查
- line 63: `.expect("Failed to parse TOML")` → `match` + 返回 `TomlConfig::default()`
- line 77: `toml::to_string(&save_config).unwrap()` → `match` 处理序列化错误

---

## 服务端文件结构要求

```
nefinita-ai.com/
├── updates/
│   ├── JPCG/
│   │   ├── update.toml        # version, major_version, data_version
│   │   ├── v1.1.251222/
│   │   │   ├── manifest.toml  # binaries, files, compressed_package
│   │   │   └── ...
│   │   └── ...
│   └── JPCG_beta/
│       └── ...
└── files/
    ├── JPCG/
    │   └── {data_version}/
    │       ├── data_manifest.toml  # 由 server_tools 生成
    │       └── data/
    │           └── ...
    └── JPCG_beta/
        └── ...
```
