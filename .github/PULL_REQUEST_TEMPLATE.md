## 背景 (Background)

<!-- 这个 PR 解决什么问题？链接相关的 issue -->

## 改动 (What changed)

<!-- 列出主要改动点，尽量具体 -->

## 分支/版本 (Branch & version)

- 来源分支：`<feature/xxx>` → 目标：`develop`
- 涉及组件版本变化：
  - jpcg_core: 
  - jpcg_const: 
  - data_version: 

## 测试验证 (Verification)

- [ ] `make check-all` 通过（static + dynamic + workspace 测试）
- [ ] 金标准回归：`cargo test -p jpcg_core -- golden`
- [ ] `cargo fmt --all -- --check` 通过
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 通过
- [ ] 前端 `npm run build` 通过
- [ ] 如有数据/模块版本变化，已更新对应 change 日志

## Change 日志

- `changes/<YYYY-MM-DD-HHMMSS>.md`：<引用路径>

## 备注

<!-- 其他需要 reviewer 注意的点 -->
