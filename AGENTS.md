# JPCG - JX3 PVP 计算器

Rust workspace for a 剑网3 damage calculator. Edition 2024 (requires rustc >= 1.85).

## Workspace

| Path | Role |
|------|------|
| `crates/jpcg_api/` | 纯 DTO 类型契约 crate（serde 类型 + HostEventsTable，零依赖，双端单一来源） |
| `crates/jpcg_core/` | Core calculation engine + `host/` JSON 入口 + FFI（句柄 + JSON 协议） |
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
cargo build-modules                  # 三个 cdylib（core/update/const）
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
```

No CI, test suites, or formatter config exist.

## Key facts

- **Chinese-language code**: field names, comments, config keys are Chinese (pinyin).
- **Data files** are TOML in `data/shuxing/`. Each profession has one `.toml` (e.g., `mowen.toml`). Core engine locates these via `std::env::current_exe()` parent → `data/shuxing/{xinfa}.toml`; env `JPCG_DATA_DIR` overrides (CLI/Python 场景).
- **`player.jcsx` is a `String`** — must NOT be coerced to number. Previously broke deserialization.
- **`saved_config.toml`** reads/writes from CWD, NOT exe parent dir.
- **`toml_input()` appends `.toml`** — callers pass the path without the extension.
- **Update server**: `https://nefinita-ai.com/updates/JPCG/` (stable) / `JPCG_beta/` (beta). `data_manifest.toml` at `https://nefinita-ai.com/files/JPCG/`. Modules(dll) manifest at `files/JPCG/{version}/modules/modules_manifest.toml` (beta: `files/JPCG_beta/modules/`).
- **FFI 协议** (`jpcg_core`): 句柄 + JSON — `jpcg_handle_create` / `jpcg_call(handle, method, request_json)` / `jpcg_last_error` / `jpcg_free_string` / `jpcg_handle_free` / `jpcg_abi_version` (1)。方法名与 Tauri 命令一一对应。更新编排经 `jpcg_set_host_events` 回调表（HostEventsTable 定义在 jpcg_api）。
- **core 分层**: `type_set/`(领域类型) → `engine/`(计算) → `store/`(文件) → `host/`(JSON 契约层，DTO↔core 转换在 `host/conv.rs`) → `ffi.rs`。金标准测试在 `engine/atkcal.rs::golden_tests`（不改行为）。
- **DTO 单源**: `jpcg_api` crate；Tauri `commands/types.rs` 仅为 `pub use jpcg_api::*`。`load_config` 返回形状与 CalculateRequest 一致（player/hostile/xinfa_config/buff/coefficient）。
- **Tauri 双模式**: `features: static`(默认)/`dynamic`。dynamic 时经 `commands/ffi_bridge.rs` dlopen（加载顺序: exe同目录/modules/ → exe同目录 → target → `JPCG_CORE_LIB` env），update 进度经 HostEvents/回调表。
- **Tauri commands** split by domain: `commands/calculate.rs`, `config.rs`, `update.rs`, `forum.rs`. Tauri events use `"update-progress"`.
- **Frontend** is React 19 + TypeScript + Vite. Entry: `src/main.tsx`. Components under `src/components/`. API layer in `src/api/commands.ts` (includes mock responses when not in Tauri).
- **Tauri lib name**: `jpcg_app_lib` (not `jpcg_app`) due to Windows crate-type conflict.
- **No `.unwrap()` / `.expect()`** in library code — all errors use `Result<_, Error>` or safe defaults. Previously caused Tauri command hangs.
- **No `rust-toolchain.toml`** — relies on `rustup default`.
- **`tauri.conf.json`** has `withGlobalTauri: true` — frontend accesses `window.__TAURI__`. CSP is `null` (disabled).
- **Forum** default port 8080, configurable via `PORT`. Data dir (`FORUM_DATA_DIR`, default `forum_data`).

## Agreement: change log

Each modification MUST be accompanied by a markdown file under `changes/` named `YYYY-MM-DD-HHMMSS.md` describing what, why, and notable decisions.
