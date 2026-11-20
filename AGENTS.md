# JPCG 开发指南（Agent）

本文件是维护者与 AI 的长期工作约定，不记录一次性修复过程。代码、配置和脚本是最新事实来源；本文件与它们冲突时先核实再更新文档。修改 `data/` 前必须阅读 `data/AGENTS.md`。

## 项目与 workspace

JPCG 是剑网3 伤害计算器：Rust 2024 workspace（rustc >= 1.85）+ Tauri v2 + React 19 + TypeScript + Vite。`examples/jpcg_app/` 是正式桌面应用，不是可随意替换的示例。

| 路径 | 职责 |
| --- | --- |
| `crates/jpcg_api/` | serde DTO 与 `HostEventsTable`，前后端契约单一来源，尽量零业务依赖 |
| `crates/jpcg_core/` | 单技能计算、配置/数据存储、host JSON 层、core FFI |
| `crates/jpcg_combo/` | 连招与战斗时间轴、期望/蒙特卡洛结算；rlib + 独立 cdylib |
| `crates/jpcg_gear/` | 配装目录、属性合成、有界/联合搜索、gear host/FFI 与 `gear-opt` |
| `crates/jpcg_compute/` | 批量 MC 后端：CPU/AVX-512 与 wgpu；不是独立更新模块 |
| `crates/jpcg_const/` | 等级常数、药品/食物常量；独立版本 |
| `crates/jpcg_update/` | App、data、模块库更新 |
| `crates/jpcg_updater/` | 独立更新器，替换主程序并重启 |
| `examples/jpcg_app/` | Tauri/React 桌面应用 |
| `server_tools/` | `manifest-gen`、`deploy-gen`、`forum`、`json-to-toml` |
| `data/` | 技能、时序、配装、值集与数据生产材料；规则见 `data/AGENTS.md` |

## 每次修改的工作约定

- 先读相关实现、测试和现有未提交改动；不要覆盖、还原、暂存或提交用户已有工作，除非用户明确要求。
- 字段、注释和配置键沿用项目的中文/拼音命名，不在同一层另起一套中英文 DTO。
- 库代码禁止新增 `.unwrap()` / `.expect()`；错误经 `Result` 或明确的安全降级处理，panic 不得跨 FFI。
- 计算逻辑改动必须跑金标准；新增行为补针对性测试。金标准用于锁定既有数值，不能为了“修测试”随意改期望值。
- 每次修改都新增 `changes/YYYY-MM-DD-HHMMSS.md`，写明做了什么、为什么、关键决策和验证边界；发布时由脚本聚合进 `CHANGELOG.md`。
- 文档只保留长期事实或明确标注日期的证据快照。不要在长期指南里复制当前版本号、测试数量或临时机器路径。

## 常用命令

```sh
make setup                 # npm ci；缓存默认在工作区 .cache/npm，可用 NPM_CACHE 覆盖
make all                   # 前端 + updater + dynamic app/五模块 + static app
make check-all             # fmt、clippy、模块、双模式、Rust/前端测试与构建
make help

make build-static          # 静态链接引擎
make build-dynamic         # 运行时加载模块
make build-modules         # core / combo / gear / update / const 五个 cdylib
make modules-dir           # dynamic app + 五模块到 target/<build>/modules/

cargo test --workspace
cargo test -p jpcg_core -- golden
cargo test -p jpcg_combo -p jpcg_core
cargo test -p jpcg_gear --locked

cd examples/jpcg_app
npm ci && npm test && npm run build
npm run dev                # 仅 Vite，端口 1420
npx tauri dev              # 完整桌面开发模式
```

前端统一使用 npm 与 `package-lock.json`。不要只改本地命令就引入 bun/pnpm；迁移包管理器必须同时评审 Makefile、Tauri、CI、发布、Dependabot、audit 和版本同步。

发布/数据工具的常用入口：

```sh
cargo run -p manifest-gen -- --version vX.Y.Z --data-dir ./data
cargo run -p deploy-gen -- --channel stable --version vX.Y.Z \
  --assets-dir ./release-assets --data-dir ./data --output ./stage
cargo run -p forum
JPCG_DATA_DIR=./data python3 examples/python_demo/jpcg_demo.py
```

## 架构与数值不变量

### DTO、host 与 FFI

- DTO 单源是 `jpcg_api`；Tauri `commands/types.rs` 只做 re-export。`load_config` 返回形状与 `CalculateRequest` 一致。
- core 分层：`type_set/` → `engine/` → `store/` → `host/` → `ffi.rs`；DTO ↔ core 转换集中在 `host/conv.rs`，不要另写平行转换。
- FFI 是句柄 + JSON：`jpcg_handle_create` / `jpcg_call` / `jpcg_last_error` / `jpcg_free_string` / `jpcg_handle_free`，ABI 版本为 1。方法名与 Tauri command 对齐。
- combo 与 gear 有各自独立 FFI；combo 专属方法不塞回 core，gear 动态任务必须配对 poll/cancel/release，退出时 release。
- 更新进度经 `HostEventsTable` / `jpcg_set_host_events`，前端事件名为 `update-progress`。
- Tauri lib 名是 `jpcg_app_lib`；`withGlobalTauri=true`，前端使用 `window.__TAURI__`，CSP 当前关闭。

### static / dynamic

- `static` 是默认特性；`dynamic` 经 `commands/ffi_bridge.rs`、`gear_bridge.rs` 等加载模块，不能在动态壳里又静态链接计算引擎。
- core 加载优先级：显式环境变量 → exe 同目录 `modules/` → exe 同目录 → target 候选。gear 使用 `JPCG_GEAR_LIB`，core 使用 `JPCG_CORE_LIB`。
- 新 DTO 能力若旧模块无法支持，要显式做能力探测并报错，不能静默走错公式。例如转换后的配装要求 `converted_attack_v1`。

### combo / timeline / gear

- combo 有期望与 MC 双通道；DOT 逐跳独立会心，MC 默认 50,000 路径。固定 seed 只保证同一有限样本下可比较，不等于真实概率的数学最优。
- HP：`max_hp > 0` 时使用 max/current，否则以 `target_hp` 为满血；全为零时旧连招击杀率恒为 1。timeline 的测量窗口有自己的满血重置语义，不要混用旧连招残血起点。
- 追加已损失生命真伤的唯一公式在 `JpcgConfig::lost_hp_append`：`已损失 = max - 结算后剩余`，伤害与真伤同扣血。
- 时序数据由 combat overlay 提供；加速、GCD、读条/引导、CD、形态、曲风、DOT、秘籍和奇穴不得在代码里按门派写分支。
- `jpcg_gear::player()` 是配装面板进入 core 的唯一适配器。`zhuanhua_gongji=Some(_)` 表示已转换并禁止旧 `atk_up` 再转换；`None` 保留旧公式与金标准。
- 联合搜索只有 `complete=true` 且 `stop_reason="exhausted"` 才能声称遍历完整；取消、超时、预算耗尽都不是全局最优证明。后端一致性、搜索完整性、统计精度分别验证。

## 路径与兼容细节

- 正式数据布局为 `data/<profession>/{shuxing.toml, combat.toml, gear/}`；`data/values/` 全职业共享，模板集中在 `data/_template/`。新增职业只新增自己的目录，不复制其他职业数值冒充真实数据。
- 旧 `data/{shuxing,combat,gear}/` 仅保留读取兼容；每个数据根内新布局优先，技能编辑只保存到新布局。当前仓库只有莫问真实职业数据。
- 数据根读取顺序：`JPCG_DATA_DIR` → macOS bundle 用户数据目录 → bundle Resources → exe 同目录 `data/`。高优先级数据损坏时应报错，不能静默回退到旧数据。
- `JPCG_DATA_DIR` 可指向数据根或旧 `shuxing/`；职业目录、旧 `combat/`、`gear/` 与共享 `values/` 按同级数据根解析。
- 职业目录选择、配装目录加载、默认配装生成、真实目录旧配装修复和属性规则校验都由 Rust host 完成。前端只传职业 ID／用户输入、保存并展示后端 DTO，不复制业务或计算规则。
- `saved_config.toml` 读写自 CWD，不是 exe 目录。
- `toml_input()` 自动追加 `.toml`，调用方不要重复扩展名。
- `player.jcsx` 是 `String`，前端清理数字字段时不得把它强制转成数字。
- `tauri.conf.json` 的 `frontendDist` 与 updater `externalBin` 会在 app 编译时校验；优先用 make 目标保证前端 dist 和 updater sidecar 的构建顺序。static/dynamic app 均经 `cargo tauri build`，默认 release 产物在 `target/release/bundle/`，不要把临时构建产物放到源码目录或 `dev` 分支。

## 分支、版本与发布

- 线性模型：`release ⊂ beta ⊂ dev`。日常 feature/fix 从最新 `dev` 切出，经 PR、review 和必需检查后 squash 合入；不要直接推受保护分支。
- hotfix 从 `release` 修复后向前传播到 `beta`、`dev`。具体保护和 CI 约定见 `CONTRIBUTING.md`。
- 根 `Cargo.toml` 的 workspace version 是 core/release tag 来源；`jpcg_const`、`jpcg_updater` 独立版本。不要使用会误改独立版本的 `cargo set-version`。
- 发布统一走 `scripts/release.sh <alpha|beta|release>`、`scripts/sync-version.sh` 与 tag 触发的 release workflow；服务器布局见 `server_manifest.md`，脚本细节见 `scripts/README.md`。
- 未经用户明确要求，不执行 commit、push、tag、发布或部署。

## 文档索引与长期决策

- `README.md`：面向用户的功能、安装和构建说明。
- `CONTRIBUTING.md`：分支、评审、验证、版本与发布流程。
- `data/AGENTS.md`：数据目录职责、录入契约、来源和验证规则。
- `docs/attribute-engine.md`：职业原始属性换算及证据边界。
- `crates/jpcg_gear/README.md`：配装目录、优化、CLI/FFI 与联合搜索。
- `docs/gear-full-benchmark.md`：完整穷举测速方法；`docs/gear-exhaustive-audit-2026-11-17.md` 是带日期的修复前审查快照。
- `server_manifest.md`：更新服务器 URL、清单和 stage 布局。
- `PLAN.md`：尚未完成的路线，不当作已实现能力。当前保留 JPCG 命名，仓库公开；数据授权仍需跟进。连招界面的器形／形态名称和面板规则由 combat 数据下发，内部 qifeng 结算模型保持兼容；场景扩展与资源引擎泛化不要顺手实现。
- `CHANGELOG.md` 与 `changes/`：历史证据，不是长期 Agent 指令；`.github/` 模板继续用于 Issue/PR。

核心引擎本身场景无关，PVE 主要是目标预设与数据模板；门派差异应落在 TOML，而不是 profession/xinfa 名称的条件分支。
