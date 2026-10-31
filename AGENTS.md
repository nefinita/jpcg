# JPCG - JX3 PVP 计算器

Rust workspace for a 剑网3 damage calculator. Edition 2024 (requires rustc >= 1.85).

## Workspace

| Path | Role |
|------|------|
| `crates/jpcg_api/` | 纯 DTO 类型契约 crate（serde 类型 + HostEventsTable，零依赖，双端单一来源） |
| `crates/jpcg_core/` | Core calculation engine + `host/` JSON 入口 + FFI（句柄 + JSON 协议） |
| `crates/jpcg_combo/` | 连招引擎（依赖 jpcg_core，排轴器后端）：编排 + MC 击杀率 + hp 步进真伤；cdylib+rlib，双产物（static 直调 / dynamic 经 ffi_bridge 加载 libjpcg_combo），dll 不进 modules_manifest |
| `crates/jpcg_gear/` | 配装契约/模板合成/有界搜索 + host/ffi（独立 cdylib）；默认 engine，`default-features=false` 仅模型，供 dynamic 壳避免静态链接计算引擎 |
| `crates/jpcg_const/` | 等级常数真源 + 药品/食物常量（编译期 const；jpcg_core 默认系数与固定公式取自 `jpcg_const::level_constant`；独立版本 `130.3.{YYYYMMDD}`） |
| `crates/jpcg_update/` | App + data + modules(dll) auto-update (fetches from `nefinita-ai.com`) |
| `crates/jpcg_updater/` | Standalone updater binary — replaces main exe & relaunches |
| `examples/jpcg_app/src-tauri/` | Tauri v2 desktop app (this IS the app, not a mere example) |
| `examples/python_demo/` | ctypes Python demo calling libjpcg_core via FFI |
| `server_tools/manifest-gen/` | CLI to generate `data_manifest.toml` + `modules_manifest.toml` |
| `server_tools/deploy-gen/` | CLI：GitHub Release 资产 → 更新服务器通道 stage（beta 根式 / stable 版本目录式） |
| `server_tools/forum/` | Web forum for uploading/downloading `.toml` data files |
| `server_tools/json-to-toml/` | 技能 JSON（IcyTide/Generator）→ `data/shuxing/*.toml` 转换器 |

## Commands

```sh
# 一键流程（make；npm 缓存默认工作区 .cache/npm，可用 NPM_CACHE=/path 覆盖）
make setup                           # npm ci 安装前端依赖（已是最新则跳过，FORCE_SETUP=1 强制）
make frontend                        # 前端构建（tsc + vite → dist/，app 构建前置）
make frontend-test                   # 前端测试（vitest）
make updater                         # 编译 jpcg_updater → binaries/jpcg_updater-<host-triple>
make all                             # setup → frontend → updater → modules-dir → build-static
                                     # 结束：jpcg_app=static、jpcg_app-dynamic=dynamic、modules/ 含 5 库
make check-all                       # fmt + clippy + 模块库 + 双模式构建 + 全量测试 + 前端 test/build
make help                            # 列出所有目标

cargo build                          # build all workspace members (incl. Tauri app)
cargo test --workspace

# 双构建模式（app 静态链接 core vs dlopen core dll）
# 以下为 .cargo/config.toml 别名，与 make build-* 目标一致
cargo build-app-static               # 默认 static：编译期链接 jpcg_core
cargo build-app-dynamic              # dynamic：dlopen libjpcg_core（更新只换 dll）
cargo build-modules                  # 五个 cdylib（core/combo/update/const/gear）
make build-static | build-dynamic | build-modules | modules-dir | test
# make modules-dir 产物：target/<build>/modules/{4 dll} + jpcg_app-dynamic；make all 最终 jpcg_app=static

cargo run -p manifest-gen -- --version v2.1.0 --data-dir ./data
# also accepts: --output data_manifest.toml (default)
#               --modules-dir <dll目录> --modules-output modules_manifest.toml --platform darwin

# 发布通道布局（GitHub Release 资产目录 → 服务器 stage；beta/stable）
cargo run -p deploy-gen -- --channel stable --version v2.1.0 \
  --assets-dir ./release-assets --data-dir ./data --output ./stage

# 技能 JSON → data/shuxing TOML（IcyTide/Generator 数据源）
cargo run -p json-to-toml -- --skills skills.json --dots dots.json --overrides overrides.json \
  --out data/shuxing --xinfa 10447 --level 130 --season 3 --modified 20260814

cargo run -p forum                   # start forum (port 8080 by default)
# PORT=9090 FORUM_DATA_DIR=./forum_data cargo run -p forum

# Tauri dev (from examples/jpcg_app/):
# npm install && npx tauri dev    (requires @tauri-apps/cli, frontendDist: ../dist)
# For Vite dev server only: npm run dev (port 1420)
# Frontend build only: npm run build

# Before Tauri build, compile & copy updater（make updater 一步完成；等价手工命令）：
# cargo build -p jpcg_updater
# cp target/debug/jpcg_updater examples/jpcg_app/src-tauri/binaries/jpcg_updater-$(rustc -vV | grep host | cut -d' ' -f2)

# Python demo（需 JPCG_DATA_DIR 指向含 shuxing/ 的目录）
# JPCG_DATA_DIR=./data python3 examples/python_demo/jpcg_demo.py

# 更新服务器布局（参考 server_manifest.md）：
# release.yml 的 package job 自动调用 deploy-gen 生成通道 stage 与 tgz（无需手工）；
# 手工生成/调试用上方「发布通道布局」的 deploy-gen 命令

# 版本管理（组件独立，见 CONTRIBUTING.md）：
# 根 workspace version = core 版本（release tag / 安装包命名源，当前 2.1.0）；继承 crates 用 version.workspace=true
# jpcg_const = 130.3.{date}（等级.赛季.日期，当前 130.3.20260602）；jpcg_updater 独立（2.1.0）
# bump: scripts/release.sh <alpha|beta|release>，或手工只改根 [workspace.package] version
#       + scripts/sync-version.sh（同步 package.json/tauri.conf.json/commands.ts 模拟串/package-lock.json）
#       不要用 cargo set-version：它会把 jpcg_const 的独立版本一起降级而失败
```

## 维护流程（三分支线性 + CI，详见 CONTRIBUTING.md）

- **分支**：`dev`(集成/最上游，alpha.n，受保护) + `beta`(公测，beta.n，tag vX.Y.Z-beta.n，受保护) + `release`(稳定，X.Y.Z，tag vX.Y.Z，受保护)。线性：`release ⊂ beta ⊂ dev`，一律经 PR，squash 合并
- **分支保护**（三分支均已配置）：必需 1 review（CODEOWNERS）+ 3 个必需状态检查 + linear history + conversation resolution；`beta`/`release` 额外 strict，`release` 额外 require last push approval
- **发布**（scripts/release.sh --stage alpha|beta|release + release.yml）：tag 触发三平台矩阵构建，beta/stable 通道由 tag 判定；package job 调 deploy-gen 生成通道 tgz 并上传 Release
- **CI**（.github/workflows/ci.yml）：dev/beta/release PR 均跑 rustfmt + clippy(不 -D warnings) + 构建/测试（含 dynamic 冒烟）+ 金标准 + 前端 test/build；必需检查名：`Rust 构建 + 测试 + lint` / `金标准回归（计算数值）` / `前端构建 (tsc + vite)`
- **依赖**：deps.yml（cargo audit / npm audit）+ dependabot.yml（每周，目标分支 dev）
- **变更日志**：`changes/` 逐条 + 发布时聚合到 CHANGELOG.md

## 开源与命名（当前决策）

- **名称**：当前保留 `JPCG`（= JX3 PVP Calc GUI）。曾考虑改名"试剑/演武/剑算"等场景无关名以支持 PVE/全场景，
  但**重命名已搁置**（内部 `jpcg_*` crate/FFI/`JPCG_*` env 均不动）。后续若改产品名，范围仅仓库/README/tauri productName/前端标题。
- **`data/`（shuxing 数据 + values 值集）**：当前**保留在仓库**，并随 data 通道（`data_manifest.toml`）下发。
  将来可能**联系其他项目负责人获取 lua 数据访问权限**后再评估数据合规与发布方式（详见 PLAN.md）。
- **开源状态**：仓库已转为 **Public**（GitHub Free）。`dev`/`beta`/`release` 三分支保护均已启用：
  必需 1 review + CODEOWNERS、3 个必需状态检查、linear history、conversation resolution；
  `beta`/`release` 额外 strict，`release` 额外 require last push approval。转公有前的旧限制已不再适用。
- **场景扩展**：core 引擎本就场景无关（PVE=不同 hostile/target_hp 配置）；支持 PVE/全场景主要是 UI 预设 + 数据模板，命名无需绑死 PVP。

## Key facts

- **Chinese-language code**: field names, comments, config keys are Chinese (pinyin).
- **Data files** are TOML in `data/shuxing/`. Each profession has one `.toml` (e.g., `mowen.toml`). Core engine locates these via `std::env::current_exe()` parent → `data/shuxing/{xinfa}.toml`; env `JPCG_DATA_DIR` overrides (CLI/Python 场景).
- **值集目录** `data/values/`：`index.toml` 是清单（`default` + `[[value_sets]]` 的 id/name/level/file），
  每个值集一个快照 TOML（换算常数）当前有 `cszj-exp-260908`（体验服·苍生铸世一测，默认）与 `live-130`（正式服·130级）；
  随 data 通道下发，客户端运行期选择，快照按 mtime 校验缓存。
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
- **版本模型（组件独立）**：root `workspace.package.version` = core/release 版本源（当前 `2.1.0`）；`version.workspace=true` 继承（api/core/combo/update/app/forum/manifest-gen/deploy-gen/json-to-toml）；`jpcg_const` = `130.3.{YYYYMMDD}`（当前 `130.3.20260602`，等级.赛季.日期，独立）；`jpcg_updater` 独立（当前 `2.1.0`）。`data_manifest.data_version` 由 manifest-gen `--version` 写入（当前发布 = `v2.1.0`）；shuxing 文件内 `[version]={level,season,modified}`，modified=YYYYMMDD。FFI 版本 getter：`jpcg_core_version`/`jpcg_const_version`/`jpcg_update_version`；Tauri `get_module_versions` 命令 + ConfigPanel 底部展示。
- **模块更新**：`modules_manifest.toml` 逐 dll 带 `version`+`sha256`（+size），本地 `modules/modules_manifest.toml` 存快照，`check_modules_update` 逐 dll 比较；部署侧由 deploy-gen 合并三平台为 `platform="multi"`，客户端按扩展名过滤。

## Agreement: change log

Each modification MUST be accompanied by a markdown file under `changes/` named `YYYY-MM-DD-HHMMSS.md` describing what, why, and notable decisions.
