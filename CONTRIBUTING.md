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

### 分支保护现状（2026-10 核实）

仓库已为 **Public**，`dev` / `beta` / `release` 三分支保护均已启用：

- 必需 **1 个 approve review**，且 **require code owner reviews**（`.github/CODEOWNERS` 生效）
- **3 个必需状态检查**：`Rust 构建 + 测试 + lint`、`金标准回归（计算数值）`、`前端构建 (tsc + vite)`
- `linear history` + `required conversation resolution`；禁 force push / 禁删除分支
- `beta` / `release` 额外 **strict**（要求 PR 分支为最新）；`release` 额外 **require last push approval**

## 开发流程

1. 从 `dev` 切特性分支：`git checkout -b feature/<name> dev`
2. 编码 + 本地验证（见下方"验证命令"）
3. 每个改动附 `changes/YYYY-MM-DD-HHMMSS.md` 变更日志（描述 什么/为什么/关键决策）
4. 提交 → push → 开 PR（目标 `dev`），通过 CI + ≥1 review（含 CODEOWNERS）后 squash 合入
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
make check-all                  # fmt + clippy + 模块库 + 双模式构建 + 全量测试 + 前端 test/build
cargo test -p jpcg_core -- golden   # 金标准回归（计算数值锁定）
cargo fmt --all -- --check
cargo clippy --workspace --all-targets   # CI 与本地一致：不 -D warnings（既有告警暂不阻断）
# 前端（examples/jpcg_app/）：
npm ci && npm test && npm run build     # vitest + tsc 类型检查 + vite build
```

> 等价 make 目标：`make fmt-check` / `make clippy` / `make test` / `make frontend-test` /
> `make frontend`；`make check-all` 已按顺序串联以上检查，失败即停。

## 版本管理

采用**组件独立版本**：

| 组件 | 版本方案 | 说明 |
|------|---------|------|
| 根 workspace（core/tag 源） | semver（当前 `2.1.0`） | 算法演进，**release tag / 安装包命名源** |
| jpcg_app / jpcg_api / jpcg_combo / jpcg_update | 继承根 workspace | `version.workspace = true` 自动跟随 |
| jpcg_const | `等级.赛季.日期`（当前 `130.3.20260602`） | 策划调常量时更新日期位，独立版本 |
| jpcg_updater | semver（当前 `2.1.0`） | 独立更新器，独立版本 |
| data_manifest.data_version | `vX.Y.Z`（当前 `v2.1.0`） | 由 manifest-gen `--version` 写入，用于数据更新判断 |
| shuxing 数据 `[version]` | `{level, season, modified}`（modified=YYYYMMDD） | 数据文件自身的等级/赛季/修改日期 |

- 版本 bump：`scripts/release.sh <alpha|beta|release>`（仅改根 `[workspace.package]` version → `sync-version.sh` → 刷新 Cargo.lock）
  - 手工等价：只改根 `Cargo.toml` 的 `[workspace.package].version` + `scripts/sync-version.sh <ver>` + `cargo metadata` 刷新 lock
  - `scripts/sync-version.sh` 同步 `package.json` / `tauri.conf.json` / `commands.ts` 模拟串 / `package-lock.json`
  - **不要用 `cargo set-version`**：它会尝试改 `jpcg_const` 的独立版本并因"降级"失败
- 继承机制：`version.workspace = true` 的 crates 跟随根版本；`jpcg_const` 与
  `jpcg_updater` 为显式独立版本，不随 workspace 变更
- **release tag 跟随 core**：`v<core_version>`（如 `v2.1.0`）；安装包命名用 core 版本

## 发布流程 (三分支)

版本阶段：dev=`X.Y.Z-alpha.n` → beta=`X.Y.Z-beta.n` → release=`X.Y.Z`

1. **dev**（alpha）：日常集成，`scripts/release.sh alpha`（不 tag），或直接随 PR 合入
2. **beta**（公测）：在 `beta` 分支 `scripts/release.sh beta` → 打 `vX.Y.Z-beta.n` → 触发 release.yml 走 beta 通道
3. **release**（稳定）：beta 公测稳定后，在 `release` 分支 `scripts/release.sh release` → 打 `vX.Y.Z` → stable 发布
4. 三分支均受保护，`release.sh` **不直推目标分支**：先推 `prep/<版本>` 并开 PR，随后推 tag 触发 release.yml；review + squash 合并后版本提交落地目标分支
5. tag 触发 release.yml：三平台矩阵构建 → 上传 GitHub Release 资产；package job 调 `server_tools/deploy-gen`
   生成通道布局并打包 `jpcg-<channel>-<tag>.tgz`（+`.sha256`）上传同一 Release；服务器侧 webhook 拉取落位
   （布局见 `server_manifest.md`）

**hotfix**：从 `release` 切出 → 修后**向前传播** release→beta→dev（保持线性超集）

## 测试要求

- 任何计算逻辑改动必须通过金标准（`engine/atkcal.rs::golden_tests`）
- 新增功能建议补充单元/冒烟测试（core FFI 协议、dynamic 桥接、前端 vitest 已有先例）
- 不得在库代码使用 `.unwrap()` / `.expect()`（用 `Result` / 安全默认值）

## Code style

- 中文（拼音）字段名/注释/配置键（项目约定）
- `rustfmt` 强制（CI `cargo fmt --check`）；`clippy` 以"不产生编译错误"为门禁，新代码应避免新增告警
- 前端 React + TS，类型定义集中在 `src/types/index.ts`，命令调用在 `src/api/commands.ts`
