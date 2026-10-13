# JPCG - JX3 PVP 计算器

Rust workspace for a 剑网3 damage calculator. Edition 2024 (requires rustc >= 1.85).

## Workspace

| Path | Role |
|------|------|
| `crates/jpcg_api/` | 纯 DTO 类型契约 crate（serde 类型 + HostEventsTable，零依赖，双端单一来源） |
| `crates/jpcg_core/` | Core calculation engine + `host/` JSON 入口 + FFI（句柄 + JSON 协议） |
| `crates/jpcg_combo/` | 连招引擎（依赖 jpcg_core，排轴器后端）：编排 + MC 击杀率 + hp 步进真伤；cdylib+rlib，双产物（static 直调 / dynamic 经 ffi_bridge 加载 libjpcg_combo），dll 不进 modules_manifest |
| `crates/jpcg_const/` | Drug/food buff constants (unused by other crates) |
| `crates/jpcg_update/` | App + data + modules(dll) auto-update (fetches from `nefinita-ai.com`) |
| `crates/jpcg_updater/` | Standalone updater binary — replaces main exe & relaunches |
| `examples/jpcg_app/src-tauri/` | Tauri v2 desktop app (this IS the app, not a mere example) |
| `examples/python_demo/` | ctypes Python demo calling libjpcg_core via FFI |
| `server_tools/manifest-gen/` | CLI to generate `data_manifest.toml` + `modules_manifest.toml` |
| `server_tools/forum/` | Web forum for uploading/downloading `.toml` data files |

## Commands

```sh
cargo build                          # build all workspace members (incl. Tauri app)
cargo test --workspace

# 双构建模式（app 静态链接 core vs dlopen core dll）
cargo build-app-static               # 默认 static：编译期链接 jpcg_core
cargo build-app-dynamic              # dynamic：dlopen libjpcg_core（更新只换 dll）
cargo build-modules                  # 四个 cdylib（core/combo/update/const）
make build-static | build-dynamic | build-modules | modules-dir | test | check-all

cargo run -p manifest-gen -- --version v2.0.X --data-dir ./data
# also accepts: --output data_manifest.toml (default)
#               --modules-dir <dll目录> --modules-output modules_manifest.toml --platform darwin

cargo run -p forum                   # start forum (port 8080 by default)
# PORT=9090 FORUM_DATA_DIR=./forum_data cargo run -p forum

# Tauri dev (from examples/jpcg_app/):
# npm install && npx tauri dev    (requires @tauri-apps/cli, frontendDist: ../dist)
# For Vite dev server only: npm run dev (port 1420)
# Frontend build only: npm run build

# Before Tauri build, compile & copy updater (see server_manifest.md for details):
# cargo build -p jpcg_updater
# cp target/debug/jpcg_updater examples/jpcg_app/src-tauri/binaries/jpcg_updater-$(rustc -vV | grep host | cut -d' ' -f2)

# Python demo（需 JPCG_DATA_DIR 指向含 shuxing/ 的目录）
# JPCG_DATA_DIR=./data python3 examples/python_demo/jpcg_demo.py

# Update server manifest generation (参考 server_manifest.md):
# 1. 将 app binary + manifest.toml 放到对应版本目录
# 2. 更新 update.toml
# 3. 将 updater binary 放到所有版本目录的公共位置

# 版本管理（组件独立，见 CONTRIBUTING.md）：
# 根 workspace version = core 版本（release tag / 安装包命名源）；继承 crates 用 version.workspace=true
# jpcg_const = 130.3.{date}（等级.赛季.日期）；jpcg_updater 独立版本
# bump: cargo set-version <ver> + scripts/sync-version.sh（同步 package.json/tauri.conf.json/前端模拟串）
```

## 维护流程（三分支线性 + CI，详见 CONTRIBUTING.md）

- **分支**：`dev`(集成/最上游，alpha.n，受保护) + `beta`(公测，beta.n，tag vX.Y.Z-beta.n，受保护) + `release`(稳定，X.Y.Z，tag vX.Y.Z，受保护)。线性：`release ⊂ beta ⊂ dev`，一律经 PR，squash 合并
- **发布**（scripts/release.sh --stage alpha|beta|release + release.yml）：tag 触发三平台矩阵构建，beta/stable 通道由 tag 判定
- **CI**（.github/workflows/ci.yml）：dev/beta/release PR 均跑 rustfmt + clippy(不 -D warnings) + 构建/测试 + 金标准 + 前端 build
- **依赖**：deps.yml（cargo audit / npm audit）+ dependabot.yml（每周）
- **变更日志**：`changes/` 逐条 + 发布时聚合到 CHANGELOG.md

## 开源与命名（当前决策）

- **名称**：当前保留 `JPCG`（= JX3 PVP Calc GUI）。曾考虑改名"试剑/演武/剑算"等场景无关名以支持 PVE/全场景，
  但**重命名已搁置**（内部 `jpcg_*` crate/FFI/`JPCG_*` env 均不动）。后续若改产品名，范围仅仓库/README/tauri productName/前端标题。
- **`data/`（shuxing 数据）**：当前**保留在仓库**。将来可能**联系其他项目负责人获取 lua 数据访问权限**后再评估数据合规与发布方式（详见 PLAN.md）。
- **开源状态**：仓库当前**私有**（GitHub Free）。**私有仓库不强制分支保护**（需 Team/Enterprise 或转公有）。
  已建 ci/release/deps 工作流 + 模板 + 脚本，转公有后即可配置完整分支保护。
- **场景扩展**：core 引擎本就场景无关（PVE=不同 hostile/target_hp 配置）；支持 PVE/全场景主要是 UI 预设 + 数据模板，命名无需绑死 PVP。

## Key facts

- **Chinese-language code**: field names, comments, config keys are Chinese (pinyin).
- **Data files** are TOML in `data/shuxing/`. Each profession has one `.toml` (e.g., `mowen.toml`). Core engine locates these via `std::env::current_exe()` parent → `data/shuxing/{xinfa}.toml`; env `JPCG_DATA_DIR` overrides (CLI/Python 场景).
- **`player.jcsx` is a `String`** — must NOT be coerced to number. Previously broke deserialization.
- **`saved_config.toml`** reads/writes from CWD, NOT exe parent dir.
- **`toml_input()` appends `.toml`** — callers pass the path without the extension.
- **Update server**: `https://nefinita-ai.com/updates/JPCG/` (stable) / `JPCG_beta/` (beta). `data_manifest.toml` at `https://nefinita-ai.com/files/JPCG/`. Modules(dll) manifest at `files/JPCG/{version}/modules/modules_manifest.toml` (beta: `files/JPCG_beta/modules/`).
- **FFI 协议** (`jpcg_core`): 句柄 + JSON — `jpcg_handle_create` / `jpcg_call(handle, method, request_json)` / `jpcg_last_error` / `jpcg_free_string` / `jpcg_handle_free` / `jpcg_abi_version` (1)。方法名与 Tauri 命令一一对应。更新编排经 `jpcg_set_host_events` 回调表（HostEventsTable 定义在 jpcg_api）。`jpcg_combo` 同协议独立 dll（`jpcg_combo_call` 等），combo 专属方法（calculate_combo/预设 CRUD/export/import_config）只在 combo dll，core ffi 不再含 combo 分支。
- **core 分层**: `type_set/`(领域类型) → `engine/`(计算) → `store/`(文件) → `host/`(JSON 契约层，DTO↔core 转换在 `host/conv.rs`) → `ffi.rs`。金标准测试在 `engine/atkcal.rs::golden_tests`（不改行为）。
- **combo 计算模型** (`jpcg_combo/engine.rs`): 双通道 — 期望通道（g/h/q/dot 期望 + 累计/方差，含追加真伤期望）/ 蒙特卡洛通道（50k 采样，dot 逐跳独立会心，真伤逐路径实时结算）。hp 语义：`max_hp>0` 用 max/current；否则 `target_hp` 满血；都 0 → 击杀率恒 1。追加真伤公式唯一实现在 core `JpcgConfig::lost_hp_append`（语义 A：`已损失=max-结算后剩余`，真伤与伤害同扣血）。
- **DTO 单源**: `jpcg_api` crate；Tauri `commands/types.rs` 仅为 `pub use jpcg_api::*`。`load_config` 返回形状与 CalculateRequest 一致（player/hostile/xinfa_config/buff/coefficient）。
- **Tauri 双模式**: `features: static`(默认)/`dynamic`。dynamic 时经 `commands/ffi_bridge.rs` dlopen（加载顺序: exe同目录/modules/ → exe同目录 → target → `JPCG_CORE_LIB` env），update 进度经 HostEvents/回调表。
- **Tauri commands** split by domain: `commands/calculate.rs`, `config.rs`, `update.rs`, `forum.rs`. Tauri events use `"update-progress"`.
- **Frontend** is React 19 + TypeScript + Vite. Entry: `src/main.tsx`. Components under `src/components/`. API layer in `src/api/commands.ts` (includes mock responses when not in Tauri).
- **Tauri lib name**: `jpcg_app_lib` (not `jpcg_app`) due to Windows crate-type conflict.
- **No `.unwrap()` / `.expect()`** in library code — all errors use `Result<_, Error>` or safe defaults. Previously caused Tauri command hangs.
- **No `rust-toolchain.toml`** — relies on `rustup default`.
- **`tauri.conf.json`** has `withGlobalTauri: true` — frontend accesses `window.__TAURI__`. CSP is `null` (disabled).
- **Forum** default port 8080, configurable via `PORT`. Data dir (`FORUM_DATA_DIR`, default `forum_data`).
- **版本模型（组件独立）**：root `workspace.package.version` = core/release 版本源；`version.workspace=true` 继承（api/core/update/app/forum/manifest-gen）；`jpcg_const` = `130.3.{YYYYMMDD}`（等级.赛季.日期，独立）；`jpcg_updater` 独立。shuxing 数据 `version={level,season,modified}`，modified=YYYYMMDD。FFI 版本 getter：`jpcg_core_version`/`jpcg_const_version`/`jpcg_update_version`；Tauri `get_module_versions` 命令 + ConfigPanel 底部展示。
- **模块更新**：`modules_manifest.toml` 逐 dll 带 `version`+`sha256`，本地 `modules/modules_manifest.toml` 存快照，`check_modules_update` 逐 dll 比较。

## Agreement: change log

Each modification MUST be accompanied by a markdown file under `changes/` named `YYYY-MM-DD-HHMMSS.md` describing what, why, and notable decisions.
