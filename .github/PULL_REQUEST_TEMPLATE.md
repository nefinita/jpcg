## 背景 (Background)

<!-- 这个 PR 解决什么问题？链接相关的 issue -->

## 改动 (What changed)

<!-- 列出主要改动点，尽量具体 -->

## 分支/版本 (Branch & version)

- 来源分支：`<feature/xxx>` → 目标：`dev`（受保护分支，需 PR + review + 必需检查，squash 合并）
- 涉及组件版本变化：
  - jpcg_core（根 workspace version）: 待填
  - jpcg_const（130.3.{date}，独立）: 待填
  - jpcg_updater（独立）: 待填
  - data_manifest.data_version: 待填

## 测试验证 (Verification)

- [ ] `make check-all` 通过（fmt + clippy + 模块库 + 双模式构建 + 全量测试 + 前端 test/build）
- [ ] 金标准回归：`cargo test -p jpcg_core -- golden`
- [ ] `cargo fmt --all -- --check` 通过
- [ ] `cargo clippy --workspace --all-targets` 通过（CI 不 `-D warnings`，但避免新增告警）
- [ ] 前端 `npm test` + `npm run build` 通过
- [ ] 如有数据/模块版本变化，已更新对应 change 日志

## Change 日志

- `changes/<YYYY-MM-DD-HHMMSS>.md`：<引用路径>

## 备注

<!-- 其他需要 reviewer 注意的点 -->
