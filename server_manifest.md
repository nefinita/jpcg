# 更新服务器文件结构

## 基础 URL

| 通道 | URL |
|------|-----|
| 稳定版（应用/清单） | `https://nefinita-ai.com/updates/JPCG/` |
| Beta（应用/清单） | `https://nefinita-ai.com/updates/JPCG_beta/` |
| 稳定版数据/模块 | `https://nefinita-ai.com/files/JPCG/` |
| Beta 数据/模块 | `https://nefinita-ai.com/files/JPCG_beta/` |

> `version` = 应用版本（含 `v` 前缀，如 `v2.1.0`）；`data_version` = `update.toml` 中声明的数据版本，
> 当前由 `manifest-gen --version` 写入，等于发布 tag（如 `v2.1.0`）。

## 客户端 URL 约定（与 `jpcg_update` 实现一致）

| 用途 | 稳定版 | Beta |
|------|--------|------|
| 最新版本信息 | `/updates/JPCG/update.toml` | `/updates/JPCG_beta/update.toml` |
| 版本清单 | `/updates/JPCG/{version}/manifest.toml` | `/updates/JPCG_beta/manifest.toml` |
| app 二进制 | `/updates/JPCG/{version}/{binary}` | `/updates/JPCG_beta/{binary}` |
| 数据清单 | `/files/JPCG/{data_version}/data/data_manifest.toml` | `/files/JPCG_beta/data/data_manifest.toml` |
| 模块清单 | `/files/JPCG/{version}/modules/modules_manifest.toml` | `/files/JPCG_beta/modules/modules_manifest.toml` |

## 部署 stage 目录结构（`deploy-gen` 输出）

`release.yml` 的 package job 以 GitHub Release 资产为输入调 `deploy-gen` 生成可直接落位的 stage：

### 稳定版（版本目录式）

```
JPCG/
├── update.toml                      # 最新版本信息（必需，指向最新版）
└── v2.1.0/                          # 版本目录（v{major}.{minor}.{patch}）
    ├── manifest.toml                # 该版本的 app 清单（三平台二进制）
    ├── jpcg-app-macos-aarch64       # macOS aarch64（Apple Silicon，单架构）
    ├── jpcg-app-linux-x86_64        # Linux x86_64
    ├── jpcg-app-windows-x86_64.exe  # Windows x86_64
    ├── modules/                     # 模块 dll（三平台合并，9 个）
    │   ├── modules_manifest.toml
    │   ├── libjpcg_{core,update,const}.dylib
    │   ├── libjpcg_{core,update,const}.so
    │   └── jpcg_{core,update,const}.dll
    └── data/                        # 数据文件（目录名 = data_version）
        ├── data_manifest.toml
        ├── readme.md
        ├── shuxing/*.toml
        └── values/*.toml
```

> 目录名说明：`data/` 位于 `{data_version}/data/`；`data_version` 与 app 版本相同时即落在同一
> `v2.1.0/` 目录下（如上图）。服务端把 `update.toml` + 版本清单/二进制以 `/updates/JPCG/` 暴露，
> 把 `modules/`、`data/` 以 `/files/JPCG/` 暴露（见上方 URL 约定）。

### Beta（根式）

```
JPCG_beta/
├── update.toml                      # 最新版本信息
├── manifest.toml                    # 直接在根（无版本子目录）
├── jpcg-app-macos-aarch64
├── jpcg-app-linux-x86_64
├── jpcg-app-windows-x86_64.exe
├── modules/
│   ├── modules_manifest.toml
│   └── ...（三平台 dll ×9）
└── data/
    ├── data_manifest.toml
    └── ...（shuxing/、values/ 等清单列出的文件）
```

> 文件名/清单 `os`+`arch` 与客户端 `std::env::consts::{OS, ARCH}` 严格一致
> （mac=macos/aarch64、linux=linux/x86_64、windows=windows/x86_64）。
> 早期 Beta 产物曾以 `darwin`/`x86_64` 错标 macOS，客户端无法命中（见 CHANGELOG 2026-09-08 记录）。

### update.toml

```toml
version = "v2.1.0"        # 最新 app 版本（带 v 前缀，客户端比较时会归一化）
major_version = 2
data_version = "v2.1.0"   # 数据版本（= data_manifest.toml 的 data_version，目录名用它）
```

### manifest.toml

```toml
version = "v2.1.0"
major_version = 2

[[binaries]]
path = "jpcg-app-macos-aarch64"
os = "macos"
arch = "aarch64"
hash = "sha256hex值"
hash_type = "SHA256"

[[binaries]]
path = "jpcg-app-linux-x86_64"
os = "linux"
arch = "x86_64"
hash = "sha256hex值"
hash_type = "SHA256"

[[binaries]]
path = "jpcg-app-windows-x86_64.exe"
os = "windows"
arch = "x86_64"
hash = "sha256hex值"
hash_type = "SHA256"

# 可选：附带文件（当前发布流程未使用）
# [compressed_package]
# path = "JPCG-v2.1.0.tar.gz"
# hash = "sha256hex值"
# hash_type = "SHA256"
```

## 更新器二进制

`jpcg_updater` 在 Tauri bundle 中通过 `externalBin` 包含，构建前需按 **host-triple 后缀**
复制到 `examples/jpcg_app/src-tauri/binaries/`（`make updater` 一步完成）：

```json
"externalBin": ["binaries/jpcg_updater"]
```

```sh
make updater
# 等价手工命令：
cargo build -p jpcg_updater
cp target/debug/jpcg_updater \
  examples/jpcg_app/src-tauri/binaries/jpcg_updater-$(rustc -vV | grep host | cut -d' ' -f2)
```

> Release 也会单独上传 updater 二进制资产；服务端通道 stage（`deploy-gen` 输出）只编排
> app 二进制 + 模块 dll + 数据，不包含 updater。

## 数据文件

数据文件独立管理，由 `manifest-gen` 生成 `data_manifest.toml`：

```toml
data_version = "v2.1.0"

[[files]]
path = "shuxing/mowen.toml"
hash = "sha256hex值"
hash_type = "SHA256"

[[files]]
path = "values/index.toml"
hash = "sha256hex值"
hash_type = "SHA256"
```

- 清单当前覆盖 `readme.md`、`data/shuxing/*.toml`、`data/values/*.toml`
- 客户端按 hash 差量下载；stable 存 `{data_version}/data/`，beta 存 `data/`
- 数据版本（`data_version`）用于数据更新判断；shuxing 文件内另有 `[version]={level,season,modified}`

## 模块库（dll）增量更新（B 模式 / 动态模式）

`manifest-gen --modules-dir` 生成单平台 `modules_manifest.toml`（逐 dll 含 version/sha256/size）；
`deploy-gen` 合并三平台 dll 后写成 `platform = "multi"` 的合并清单（客户端按扩展名过滤）：

```toml
modules_version = "v2.1.0"
platform = "multi"

[[files]]
name = "jpcg_const.dll"
version = "130.3.20260602"   # jpcg_const 独立版本
hash = "sha256hex值"
hash_type = "SHA256"
size = 120320
```

```sh
# 单平台清单（CI 构建时）
cargo run -p manifest-gen -- --version v2.1.0 \
  --data-dir ./data \
  --modules-dir ./dist/modules \
  --modules-output ./dist/modules/modules_manifest.toml \
  --platform macos

# 通道 stage（stable/beta 布局 + 三平台合并 modules_manifest）
cargo run -p deploy-gen -- --channel stable --version v2.1.0 \
  --assets-dir ./release-assets --data-dir ./data --output ./stage
```

### 客户端行为（动态模式）

1. `check_update` 在 force 或有 app 更新时拉取 `modules_manifest.toml`，
   与本地 `exe同目录/modules/` 子目录按 SHA256 差量对比
2. `perform_modules_update` 下载 → 校验 → 原子替换到 `modules/` → 请求重启
3. 重启后 ffi_bridge 优先加载 `modules/libjpcg_core.*`（增量生效，app 本体无需重装）
