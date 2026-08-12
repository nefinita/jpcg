# 更新服务器文件结构

## 基础 URL

| 通道 | URL |
|------|-----|
| 稳定版 | `https://nefinita-ai.com/updates/JPCG/` |
| Beta | `https://nefinita-ai.com/updates/JPCG_beta/` |
| 数据文件 | `https://nefinita-ai.com/files/JPCG/` |
| Beta 数据 | `https://nefinita-ai.com/files/JPCG_beta/` |

## 稳定版目录结构

```
JPCG/
├── update.toml                    # 版本信息（必需）
├── v2.1.0/                        # 版本目录（命名规则 v{major}.{minor}.{patch}）
│   ├── manifest.toml              # 该版本的完整清单
│   ├── jpcg-app-x86_64-linux      # Linux x86_64 二进制
│   ├── jpcg-app-x86_64-windows.exe # Windows x86_64 二进制
│   ├── jpcg-app-aarch64-macos     # macOS ARM64 二进制
│   └── jpcg-app-x86_64-macos      # macOS x86_64 二进制
└── v2.0.0/
    └── ...
```

### update.toml

```toml
version = "v2.1.0"
major_version = 2
data_version = "v2.0.2026050201"
```

### manifest.toml

```toml
version = "v2.1.0"
major_version = 2

[[binaries]]
path = "jpcg-app-x86_64-linux"
os = "linux"
arch = "x86_64"
hash = "sha256hex值"
hash_type = "SHA256"

[[binaries]]
path = "jpcg-app-x86_64-windows.exe"
os = "windows"
arch = "x86_64"
hash = "sha256hex值"
hash_type = "SHA256"

[[binaries]]
path = "jpcg-app-aarch64-macos"
os = "macos"
arch = "aarch64"
hash = "sha256hex值"
hash_type = "SHA256"

# 可选：附带文件
[compressed_package]
path = "JPCG-v2.1.0.tar.gz"
hash = "sha256hex值"
hash_type = "SHA256"
```

## Beta 版目录结构

```
JPCG_beta/
├── manifest.toml                  # 直接放根目录（无版本子目录）
├── jpcg-app-x86_64-linux
├── jpcg-app-x86_64-windows.exe
└── ...
```

Beta 的 `manifest.toml` 格式同稳定版。

## 更新器二进制

`jpcg_updater`（无平台后缀）需放置在可执行文件同级目录下。在 Tauri bundle 中通过 `externalBin` 包含：

```json
"externalBin": ["binaries/jpcg_updater"]
```

构建前需将编译好的更新器复制到 `examples/jpcg_app/src-tauri/binaries/`：

```sh
cargo build -p jpcg_updater
cp target/debug/jpcg_updater examples/jpcg_app/src-tauri/binaries/
```

## 数据文件

数据文件独立管理，见 `data_manifest.toml`，由 `manifest-gen` 工具生成。

## 模块库（dll）增量更新（B 模式 / 动态模式）

模块库与 app 版本同目录部署，由 `manifest-gen` 的 `--modules-dir` 生成
`modules_manifest.toml`（含每个 dll 的 SHA256 与 size）：

```sh
cargo run -p manifest-gen -- --version v2.1.0 \
  --data-dir ./data \
  --modules-dir ./dist/modules/v2.1.0 \
  --modules-output ./dist/modules/v2.1.0/modules_manifest.toml
```

### 目录结构（与数据文件布局对齐）

```
files/JPCG/
└── v2.1.0/
    ├── data/
    │   └── data_manifest.toml
    └── modules/
        ├── modules_manifest.toml
        ├── libjpcg_core.dylib
        ├── libjpcg_update.dylib
        └── libjpcg_const.dylib
```

Beta 通道模块清单不带版本目录：`files/JPCG_beta/modules/`。

### 客户端行为（动态模式）

1. `check_update` 在 force 或有 app 更新时拉取 `modules_manifest.toml`，
   与本地 `exe同目录/modules/` 子目录按 SHA256 差量对比
2. `perform_modules_update` 下载 → 校验 → 原子替换到 `modules/` → 请求重启
3. 重启后 ffi_bridge 优先加载 `modules/libjpcg_core.*`（增量生效，app 本体无需重装）
