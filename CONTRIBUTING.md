# JPCG 贡献指南

JPCG 是一个剑网3 伤害计算器，采用**三分支线性**模型，Rust workspace + Tauri v2 桌面应用。

## 分支模型（三分支线性：release ⊂ beta ⊂ dev）

```
dev      （集成/最上游，alpha.n）← 所有 feature/* 合入，受保护
  │ 稳定后 提升
  ▼
beta     （预发布/公测，beta.n，tag vX.Y.Z-beta.n）受保护
  │ 公测稳定后 提升
  ▼
release  （稳定/生产，X.Y.Z，tag vX.Y.Z）受保护
```

- **永不直接 push** dev / beta / release；一律通过 PR + review
- 分支命名：`feature/*`、`fix/*`、`hotfix/*`
- 合并方式：squash（保持链式线性）
- 线性关系：`release ⊂ beta ⊂ dev`（单向超集），新提交从 dev 流向 release

## 开发流程

1. 从 `dev` 切特性分支：`git checkout -b feature/<name> dev`
2. 编码 + 本地验证（见下方"验证命令"）
3. 每个改动附 `changes/YYYY-MM-DD-HHMMSS.md` 变更日志（描述 什么/为什么/关键决策）
4. 提交 → push → 开 PR（目标 `dev`），通过 CI + ≥1 review 后 squash 合入
5. 需要公测时：提升 dev → beta（`scripts/release.sh beta`，打 beta tag）
6. 公测稳定后：提升 beta → release（`scripts/release.sh release`，打正式 tag）

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

## 发布流程 (三分支)

版本阶段：dev=`X.Y.Z-alpha.n` → beta=`X.Y.Z-beta.n` → release=`X.Y.Z`

1. **dev**（alpha）：日常集成，`scripts/release.sh alpha`（不 tag），或直接随 PR 合入
2. **beta**（公测）：在 `beta` 分支 `scripts/release.sh beta` → 打 `vX.Y.Z-beta.n` → 触发 release.yml 走 beta 通道
3. **release**（稳定）：beta 公测稳定后，在 `release` 分支 `scripts/release.sh release` → 打 `vX.Y.Z` → stable 发布
4. 按 `server_manifest.md` 部署 update.toml / manifests 到 `nefinita-ai.com`（stable/beta 通道）

**hotfix**：从 `release` 切出 → 修后**向前传播** release→beta→dev（保持线性超集）

## 测试要求

- 任何计算逻辑改动必须通过金标准（`engine/atkcal.rs::golden_tests`）
- 新增功能建议补充单元/冒烟测试（core FFI 协议、dynamic 桥接已有先例）
- 不得在库代码使用 `.unwrap()` / `.expect()`（用 `Result` / 安全默认值）

## Code style

- 中文（拼音）字段名/注释/配置键（项目约定）
- 遵循 rustfmt / clippy（CI 强制）
- 前端 React + TS，类型定义集中在 `src/types/index.ts`，命令调用在 `src/api/commands.ts`
