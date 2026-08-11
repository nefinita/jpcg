# JPCG 贡献指南

JPCG 是一个剑网3 PVP 伤害计算器，采用 **git-flow** 分支模型，Rust workspace + Tauri v2 桌面应用。

## 分支模型 (git-flow)

```
master       生产分支（可发布，仅经 release/hotfix 合入，受保护）
  └─ develop 集成交互分支（日常开发的汇合点，受保护）
       ├─ feature/*   新功能 → 合入 develop
       └─ release/vX  发布准备 → 合入 master + develop
master ── hotfix/*   紧急修复 → 合入 master + develop
```

- **永不直接 push** master / develop；一律通过 PR
- 特性分支命名：`feature/<描述>`、`fix/<描述>`、`hotfix/<描述>`、`release/vX.Y.Z`
- 合并方式：squash（保持 master/develop 历史线性）

## 开发流程

1. 从 `develop` 切特性分支：`git flow feature start <name>` 或 `git checkout -b feature/<name> develop`
2. 编码 + 本地验证（见下方"验证命令"）
3. 每个改动附 `changes/YYYY-MM-DD-HHMMSS.md` 变更日志（描述 什么/为什么/关键决策）
4. 提交 → push → 开 PR（目标 `develop`），通过 CI + ≥1 review 后 squash 合入

## Commit 规范

沿用仓库风格：`type(scope): 中文描述`

```
feat(core): ...    fix(app): ...    refactor(calc): ...
test(engine): ... build: ...       chore: ...         docs: ...
```

## 验证命令（合并前必须全绿）

```sh
make check-all                  # static + dynamic + modules 构建 + workspace 测试
cargo test -p jpcg_core -- golden   # 金标准回归（计算数值锁定）
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
# 前端（examples/jpcg_app/）：
npm ci && npm run build         # tsc 类型检查 + vite build
```

## 版本管理

采用**组件独立版本**：

| 组件 | 版本方案 | 说明 |
|------|---------|------|
| data_version | `等级.赛季.日期`（如 `130.3.20260602`） | 数据更新判断 + UI 展示 |
| jpcg_const | `等级.赛季.日期` | 策划调常量时更新日期位 |
| jpcg_core | semver | 算法演进，**release tag / 安装包命名源** |
| jpcg_app | semver | 界面版本展示 |
| jpcg_api / jpcg_update / jpcg_updater | semver | 独立演进 |

- **数据版本**（`等级.赛季.日期`）映射到 shuxing 数据的 `{level, season, modified}`（modified = YYYYMMDD）
- 版本 bump：`cargo set-version <ver>`（改根 workspace version，继承 crates 自动跟随）
  + `scripts/sync-version.sh`（同步 package.json/tauri.conf.json/前端模拟串）
- 继承机制：`version.workspace = true` 的 crates 跟随根版本；`jpcg_const`（130.3.date）与
  `jpcg_updater`（独立）为显式独立版本，不随 workspace 变更
- **release tag 跟随 core**：`v<core_version>`（如 `v2.1.0`）；安装包命名用 core 版本

## 发布流程 (Release)

1. 从 develop 切 `release/vX.Y.Z` → 冻结（只修 bug，不加功能）
2. `scripts/release.sh`：全量测试 → bump 版本 → 聚合 CHANGELOG → commit → tag `vX.Y.Z`
3. tag 触发 CI `release.yml`：三平台矩阵构建 + 打包 → GitHub Release（命名用 core 版本）
4. 发布后合回 master + develop；如修复走 `hotfix/*` 并回补 develop
5. 按 `server_manifest.md` 部署 update.toml / manifests 到 `nefinita-ai.com`

## 测试要求

- 任何计算逻辑改动必须通过金标准（`engine/atkcal.rs::golden_tests`）
- 新增功能建议补充单元/冒烟测试（core FFI 协议、dynamic 桥接已有先例）
- 不得在库代码使用 `.unwrap()` / `.expect()`（用 `Result` / 安全默认值）

## Code style

- 中文（拼音）字段名/注释/配置键（项目约定）
- 遵循 rustfmt / clippy（CI 强制）
- 前端 React + TS，类型定义集中在 `src/types/index.ts`，命令调用在 `src/api/commands.ts`
