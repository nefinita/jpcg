# JPCG - 剑网3 PVP 伤害计算器

> **精确计算每一发伤害，科学规划连招，轻松掌握击杀线。**
>
> **版本**: v2.1.0-alpha.2 | **更新日期**: 2026年8月
> **适用平台**: Windows / macOS / Linux (Android 适配中)

---

## 📖 工具简介

JPCG 是一款专为剑网3 PVP 玩家打造的桌面端伤害计算工具。从面板属性录入、目标防御设定、阵眼奇穴增益，到全技能伤害计算、连招击杀概率分析、技能数据编辑，覆盖 PVP 伤害计算的完整链路。

与网页版计算器不同，JPCG 采用桌面原生应用架构，数据本地保存，切换页面不丢失，离线可用，毫秒级响应。支持所有心法，数据文件随版本持续更新，社区论坛互通配置。

---

## 🚀 核心功能

### 📊 全门派 PVP 伤害模拟

完整实现从属性输入到伤害输出的全链路计算：

| 模块 | 输入项 | 说明 |
|------|--------|------|
| **面板属性** | 基础属性、基础攻击、会心等级、会心效果、破防等级、武器伤害 | 裸面板数据 |
| **目标设定** | 外功防御、内功防御、御劲等级、化劲等级、减伤比例、目标血量、目标最大/当前血量 | 模拟竞技场目标（最大/当前血量驱动追加真伤与击杀线） |
| **阵眼增益** | 基础攻击% / 会心% / 会效% / 破防% / 无视防御% / 伤害提升% | 6 项增益自由调节 |
| **系数设置** | 破防系数 / 会心系数 / 会效系数 / 化劲系数 / 防御系数 / PVP 全局减伤 | 默认填充公式常量，支持深度调参 |

**伤害公式链路**：

```rust
// 简化示意：完整计算链
攻击力 → (1 + 破防加成) → 会心判定 → 会效加成
    → 目标防御减免 → 化劲减免 → PVP 全局减伤 → 最终伤害
```

- 支持 **Dot 持续伤害** 完整计算
- 填写属性时 **实时显示** 会心率、破防率、减伤等关键指标
- 切换心法自动填充对应倍率参数

### 🔗 连招规划与击杀概率分析

从单技能计算升级为完整的连招模拟系统：

- **拖拽式编排**：基于 `@hello-pangea/dnd`，从技能池拖拽技能组成序列，支持拖拽排序
- **技能池动态加载**：自动读取当前心法全部技能，右键 ⭐ 标记最爱，快速筛选
- **每步伤害明细**：期望伤害 / 会心伤害 / 不会心伤害，逐条展示

**击杀概率曲线** —— 蒙特卡洛模拟（50,000 条随机路径）：

- 会心按技能判定，**Dot 逐跳独立判定**，每条路径按实际血量实时结算
- 追加真伤（已损失生命值 × 系数）随血量损失实时递增，伤害与真伤同扣血
- 击杀率 = 击杀路径占比，确定性无质连招给出精确 0/1

```rust
// 击杀率计算示意（简化）
for _ in 0..50_000 {
    let mut hp = target_hp;
    for step in combo {
        let dmg = if crit(&step) { step.h } else { step.g }; // Dot 逐跳判定
        let lost = append_lost_hp(max_hp, hp - dmg);         // 追加真伤
        hp = (hp - dmg - lost).max(0.0);
        if hp <= 0.0 { kills += 1; break; }
    }
}
kill_prob = kills / 50_000
```

输入目标血量（可选填最大/当前血量）后生成概率曲线图，直观回答"这套连招打多久能杀"。

- **每步伤害柱状图**：recharts 图表对比各技能贡献
- **单步微调**：每个技能可独立调整攻击、会心、破防等 8 项参数，模拟不同 buff 覆盖
- **预设管理**：保存 / 加载 / 删除多套方案，实战前提前规划

### ✏️ 内置技能编辑器

无需打开文本文件，在应用内直接维护心法数据：

| 可编辑内容 | 具体字段 |
|------------|----------|
| **心法信息** | 名称、基础属性（根骨/力道/身法/元气）、攻击/破防/会心倍率、版本号 |
| **技能数据** | 技能名称、基础伤害、伤害系数、武器系数、会心加成、无视防御/化劲/减伤、真实伤害 |
| **Dot 参数** | Dot 标识、Dot 数值、Dot 增益系数 |
| **增删操作** | 添加新技能、删除不需要的技能 |

修改后一键保存，即时生效。再也不用面对 TOML 文件手动改 JSON 了。

### 🌐 配置分享论坛

内置论坛面板，连接社区数据：

- **分类浏览**：按心法分类浏览其他玩家上传的心法数据
- **一键下载**：感兴趣的数据直接下载到本地
- **已下载管理**：已下载文件绿色标识，支持一键删除
- **互通有无**：上传自己的配置，与社区交流

### 🎨 界面与体验

- **薄荷/松石绿清新主题**：自绘 SVG 图标，美观与清晰兼顾
- **深色/浅色主题**：一键切换，跟随系统或个人偏好
- **四页面导航**：计算 / 论坛 / 排轴器 / 技能编辑，侧边栏快速切换，页面状态保持
- **骨架屏加载**：计算过程中优雅动画替代空白等待
- **Toast 通知**：操作结果实时反馈
- **键盘快捷键**：ESC 关闭侧栏

---

## 📦 本次更新 (v2.1.0-beta.1)

- **更新自动部署上线**：tag 构建 → `deploy-gen` 编排通道目录并打包
  `jpcg-<channel>-<tag>.tgz`（附 `.sha256`）→ 上传 GitHub Release →
  服务器 webhook 自动拉取、校验并原子切换（beta 覆写根 / stable 保留 3 版），全程无人值守
- **等级常数编译期固化**：新增 `jpcg_const::level_constant`（预设 TOML + `const fn` 白名单解析），
  `CoefficientConfig` 默认值单一真源；补齐御劲/御会（会伤减免）分母并修正公式，golden 回填
- **发布/构建修复**：`release.sh` 兼容独立版本 crate（`jpcg_const`）与分支命名空间；
  Windows 构建 job 修正为 bash
- **模块更新多平台**：三平台 dll 合并清单（`platform="multi"`）+ 客户端按本机平台过滤

---

## 📦 版本记录 (v2.1.0-alpha.2)

```diff
+ 🎯 全新连招引擎 jpcg_combo —— 蒙特卡洛击杀率 + hp 步进追加真伤
+ 🐛 修复排轴器 dot 只算单跳 / 追加真伤语义两处 bug
+ ❤️ 目标最大/当前血量输入 —— 驱动追加真伤斩杀结算
+ ✏️ 技能编辑器 / 论坛 / 击杀曲线 / 预设管理与数据版本管理（alpha.1 起已包含）
```

### 连招引擎全面重建（jpcg_combo）
- 击杀率从正态近似升级为**蒙特卡洛模拟**（50,000 路径），dot 逐跳独立会心
- 追加真伤修正为语义 A：`已损失 = 最大血量 − 结算后剩余`，随损失实时递增结算
- 新引擎独立 dll（`libjpcg_combo`），与 core 分离，双构建模式（static 直调 / dynamic dlopen）

### 排轴器新增目标血量输入
目标设定支持「目标最大血量 / 当前血量」，未填时回退 `target_hp` 满血模型。

### ✏️ 技能编辑器
应用内直接查看和编辑心法技能数据。支持添加/删除技能、修改心法倍率与版本信息，所有字段可视可编辑，修改后一键保存。告别手动修改 TOML 配置文件。

### 🌐 论坛体验升级
- 已下载文件显示绿色"已下载"标签，一眼识别
- 支持一键删除已下载文件，释放本地空间
- 下载/删除后自动刷新列表

### 📊 连招系统
- **击杀概率曲线**：蒙特卡洛模拟（50,000 路径），配每步伤害柱状图 / 预设管理

### 🎯 动态职业列表
自动识别 `data/shuxing/` 下的数据文件，下拉菜单显示心法名称及版本信息（如"莫问 130级第3赛季"），无需手动配置。

### 📋 配装导入/导出
支持导出当前配置为 TOML 文件，方便保存和分享；也支持导入外部配置。

### 📁 数据版本管理
心法数据引入 `[version]` 节（等级、赛季、修改日期），支持多版本数据共存，新旧赛季一目了然。

---

## 🗓️ 未来计划

```rust
// 2026 后续开发路线
pub enum Roadmap {
    /// 正式稳定版本发布
    StableRelease,
    /// 属性微分分析 (AutoDiff) — 已实现（compute_derivatives + 边际收益展示）
    AttributeAutoDiffDone,
    /// 智能配装器 — 基于约束搜索的最优方案推荐
    /// 输入预算/品级/套装条件，自动输出最优属性配比
    SmartGearOptimizer,
    /// 论坛用户系统 + 评论互动
    ForumCommunity,
    /// 更多可视化分析工具（属性收益曲线、装备对比等）
    AdvancedVisualization,
}
```

- **正式稳定版本发布**
- **更多心法数据持续更新**，覆盖全部门派
- **智能配装器**：基于约束条件搜索最优配装方案
- **论坛系统**：用户账号、评论互动、配置评分
- **Windows / macOS 安装包**

---

## 🔧 编译与运行

> 源码编译需要 Rust ≥ 1.85（rustup 默认 toolchain）与 Node.js ≥ 18。
> Windows / macOS / Linux 三平台均可构建。

### 快速开始

```sh
cargo build                 # 构建全部 workspace 成员（含 Tauri 应用）
cargo test --workspace      # 全量测试（含金标准回归）
```

### 前端与桌面应用（Tauri v2）

```sh
# 首次：安装前端依赖
cd examples/jpcg_app
npm install

# 方式一：仅前端 Vite dev server（端口 1420）
npm run dev

# 方式二：完整 Tauri 开发模式（Rust + 前端热重载）
npx tauri dev

# 前端构建（产物输出到 dist/）
npm run build
```

### 双构建模式（应用 ↔ 引擎库）

应用与 `jpcg_core` / `jpcg_combo` 引擎存在两种链接方式：

| 模式 | 命令 | 说明 |
|------|------|------|
| **static**（默认） | `cargo build-app-static` / `make build-static` | 编译期静态链接引擎，调试最直接 |
| **dynamic** | `cargo build-app-dynamic` / `make build-dynamic` | 运行时 dlopen `libjpcg_core`，更新只需替换 dll |
| **模块 dll** | `cargo build-modules` / `make build-modules` | 构建四个 cdylib：core / combo / update / const |
| **模块目录** | `make modules-dir` | 把 dll 复制到应用同目录（dynamic 模式运行必需） |

### 打包发布

```sh
# 1. 先编译 updater 并复制到 Tauri binaries（必需，否则 build 失败）
cargo build -p jpcg_updater
cp target/debug/jpcg_updater \
  examples/jpcg_app/src-tauri/binaries/jpcg_updater-$(rustc -vV | grep host | cut -d' ' -f2)

# 2. Tauri 打包安装包
cd examples/jpcg_app
npx tauri build
```

### 更新发布（自动链路）

打 tag 后全链路自动完成，无需人工介入：

```
推 tag vX.Y.Z(-beta.n)
  └─ release.yml
       ├─ 三平台构建 → 资产上传 GitHub Release
       └─ package job：deploy-gen 生成通道布局
            → jpcg-<channel>-<tag>.tgz + .sha256 上传同一 Release
GitHub Release webhook
  └─ nefinita_download_service（服务器）
       POST /hooks/github → 验签 → 拉取 tgz → sha256 校验
       → 原子切换（beta 覆写 / stable 合并留 3 版）
```

相关工具：`server_tools/deploy-gen`（布局编排）、`scripts/release.sh`（三分支发布）、
`scripts/sync-version.sh`（版本同步）；服务器文件结构见 `server_manifest.md`。

### 完整校验

```sh
make check-all      # static + dynamic + 模块 dll + 全量测试 全绿检查
```

### 常用开发命令

```sh
# 连招引擎 / 核心引擎单测
cargo test -p jpcg_combo -p jpcg_core

# Python FFI 调用演示（需 JPCG_DATA_DIR 指向含 shuxing/ 的目录）
JPCG_DATA_DIR=./data python3 examples/python_demo/jpcg_demo.py

# 论坛服务（配置分享，端口 8080）
cargo run -p forum
```

### Workspace 结构速览

```
crates/jpcg_api/      纯 DTO 类型契约（serde，跨端单一来源）
crates/jpcg_core/     核心计算引擎 + host JSON 层 + FFI（句柄 + JSON 协议）
crates/jpcg_combo/    连招引擎（依赖 core；cdylib+rlib 双产物）
crates/jpcg_update/   App / 数据 / 模块(dll) 自动更新（nefinita-ai.com）
crates/jpcg_updater/  独立更新器（替换主程序并重启）
crates/jpcg_const/    药品/食物常量 + 等级常数（编译期由 preset TOML 固化）
examples/jpcg_app/    Tauri v2 桌面应用（React 19 + TypeScript + Vite）
server_tools/         manifest/deploy-gen 生成器 / 配置分享论坛 / 数据转换器
```

---

## 📁 配置文件示例

心法数据以 TOML 格式存储，结构清晰易懂：

```toml
# data/shuxing/mowen.toml — 莫问 130级第3赛季
[xinfa]
xinfa_name = "莫问"
xinfa_nom = "根骨"
atk_up = 1.96
pofang_up = 2.0
huixin_up = 0.0

[version]
level = 130
season = 3
modified = 20260602

[[skill]]
skill_name = "宫"
base_damage1 = 160
base_damage2 = 200
atk_xishu = 2.609375
```

---

## 🙏 致谢

- 感谢所有参与测试、反馈问题的江湖侠士
- 感谢社区提供的宝贵建议与数据支持

> 💬 问题反馈 / 建议请通过发布页面提交
>
> 🗡️ **剑起江湖，算无遗策** —— JPCG 与你一起，精进每一分输出！
