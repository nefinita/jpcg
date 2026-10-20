# JPCG 开源与发展计划 (PLAN)

> 本文件记录 JPCG 的开源路线与后续方向（数据授权、场景扩展）。
> 仓库已转为 **Public**（GitHub Free），`dev` / `beta` / `release` 三分支保护均已配置；
> 数据授权（M2）仍在推进。目标是长期健康运营并逐步扩展场景。

---

## 1. 背景与目标

- JPCG = JX3 PVP Calc GUI，剑网3 伤害计算器（Tauri v2 + Rust core，版本化桌面软件）。
- 计算引擎本就**场景无关**（PVE 只是换 hostile/target_hp 配置），具备扩展空间。
- 目标：**已通过转公有获得完整分支保护** + 形成可协作、可发布的维护流程；远景支持 PVE/全场景。

## 2. 当前状态（基线，2026-10-20 核实）

| 项 | 状态 |
|----|------|
| 最新发布 | **v2.1.0**（2026-10-13），GitHub Release 含三平台 app 产物 + 通道 tgz（+sha256）+ 模块 dll + 清单 |
| 分支策略 | 三分支线性：`dev`(alpha) + `beta`(beta) + `release`(稳定)，release ⊂ beta ⊂ dev |
| 仓库可见性 | **Public**（GitHub Free） |
| 分支保护 | 三分支均启用：必需 1 review（CODEOWNERS）+ 3 个必需状态检查 + linear history + conversation resolution；`beta`/`release` 额外 strict，`release` 额外 require last push approval；禁强推/禁删除 |
| 必需状态检查 | `Rust 构建 + 测试 + lint`、`金标准回归（计算数值）`、`前端构建 (tsc + vite)` |
| CI/CD | ci.yml / release.yml（三平台矩阵）/ deps.yml + dependabot（目标分支 dev） |
| 文档/模板 | CONTRIBUTING / CHANGELOG / PR+Issue 模板 / AGENTS / server_manifest / CODEOWNERS |
| 版本模型 | 组件独立：root=core/tag 源（2.1.0）；const=`130.3.{date}`（130.3.20260602）；updater 独立（2.1.0） |
| 命名 | 保留 `JPCG`（改名已搁置） |
| 数据 | `data/shuxing/` 与 `data/values/` 保留在仓库并随 data 通道下发；暂未获得上游 lua 数据授权 |

## 3. 里程碑

### M0 — 现状基线（已完成）
- 三分支（dev/beta/release）+ CI/CD + 模板 + 脚本 + 组件独立版本（见 CHANGELOG 中「来自 changes/2026-08-11-231500.md」条目）

### M1 — 转公有与分支保护（已完成，2026-10 核实）
- **转公有**：已完成，仓库为 Public（GitHub Free 即完整支持分支保护）
- **配置分支保护**：已完成，实际配置如下：
  - 必需 1 个 approve review + require code owner reviews（`.github/CODEOWNERS` 生效）
  - 3 个必需状态检查：`Rust 构建 + 测试 + lint` / `金标准回归（计算数值）` / `前端构建 (tsc + vite)`
  - `linear history` + `required conversation resolution`；禁 force push / 禁删除分支（dev/release 对管理员同样生效）
  - `beta` / `release` 额外 strict（要求 PR 分支为最新）；`release` 额外 require last push approval
- **重命名（可选，已搁置）**：保持 `JPCG`，内部 `jpcg_*` crate/FFI/`JPCG_*` env 均不动
- **CODEOWNERS**：已添加（`* @QinThirteen`）；merge-gate 未单独引入（由必需检查 + review 覆盖）
- **收尾文案（部分完成）**：README 徽章、`SECURITY.md` 尚未补齐，作为后续运营事项

### M2 — 数据授权与合规（进行中/待办，暂无新授权进展）
- **目标**：联系其他项目负责人，获取 `data/`（shuxing 技能数值/公式）的 **lua 数据访问/再分发权限**
- **待办**：
  - 梳理数据来源与归属；与上游项目确认授权方式（白名单/标注/免再分发等）
  - 评估三种数据发布形态：A) 随仓库公开；B) 代码公开 + 数据走更新服务器分发（不入库）；C) 其他
  - 若无法获得授权 → 回退到 B（仓库不携带数据，运行时经 `JPCG_DATA_DIR`/更新服务器加载）
- **达成标准**：`data/` 的发布方式有明确授权依据，并写入本 PLAN / README / server_manifest

### M3 — 开源运营（已转公有，进行中）
- Issue/PR 治理：模板 + CONTRIBUTING + CODEOWNERS + 三分支保护已就绪；维护者 review 纪律持续执行
- 发布节奏：`scripts/release.sh` + release.yml 三平台 Release；发布后经 deploy-gen 通道 tgz 自动化部署
- 依赖安全：dependabot + deps.yml（cargo/npm audit）定期
- 社区反馈渠道：README 指向 Issue/讨论；`SECURITY.md` 待补

### M4 — 场景扩展（远期，功能稳定后）
- **引擎层**：已场景无关，零改动
- **UI 层**：新增"目标类型"预设（PVP 玩家 / PVE Boss：大血量、0 化劲/御劲）
- **数据层**：boss 属性模板、副本环境增益模板
- **命名**：若支持 PVE/全场景，产品名无需含 PVP（与 M1 重命名决策联动）

## 4. 风险与合规要点

- **名字**：`JPCG`/`JPCG` 无版权风险（短名称不受版权保护）；商标风险低（非商业项目）。
- **游戏数据**（真正的合规重点）：剑网3 数据为西山居商业游戏衍生内容，**版权/IP 归其所有**。
  社区惯例普遍公开（如 JX3Toy 公开技能数据、本项目作者旧仓库 Apache-2.0 公开），但属"实际惯例"非"法律保障"。
  → 由 M2 授权结果决定最终形态。
- **更新服务器**：`nefinita-ai.com` 端点公开即暴露基础设施（无密钥，风险低）。
- **无密钥风险**：已扫描，仓库不含真实 token/密钥。

## 5. 决策记录

| 日期 | 决策 |
|------|------|
| 2026-08-11 | 采用三分支线性（dev/beta/release）+ CI/CD + 组件独立版本（M0 落地） |
| 2026-08-11 | 保留 `JPCG` 命名；`data/` 暂留仓库；未来联系上游获取 lua 授权后再定数据发布（本 PLAN） |
| 2026-10-20 | 核实并记录：仓库已 Public、三分支保护已配置（M1 完成）；v2.1.0 已发布（2026-10-13）；M2 数据授权仍待推进 |
