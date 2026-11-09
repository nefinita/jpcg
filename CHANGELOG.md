# Changelog

本文件按 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/) 格式，于每次发布时由 `scripts/release.sh`
从 `changes/` 聚合生成。版本号遵循语义化版本（SemVer）。

## [2.1.0-beta.1] - 2026-09-04

### 来自 changes/2026-05-14-114749.md

# 论坛弹窗 → VSCode 风格左侧菜单栏

## 改动

将原来的论坛弹窗（modal）重构为 VSCode 风格的左侧活动栏 + 侧栏布局：

### index.html
- 移除了顶栏的 `#btn-forum` 按钮和 `#forum-modal` 弹窗
- 新增 `.activity-bar`（44px 宽的活动栏，位于最左侧）
- 新增 `#forum-sidebar`（可折叠的 340px 侧栏面板，包含论坛内容）
- 新增 `.main-area` 包裹原有的 header / main / footer
- 所有现有功能面板和逻辑保持不变

### style.css
- 新增 `.app-body` flex 行布局容器
- 新增 `.activity-bar` / `.activity-bar-btn` 活动栏样式
- 新增 `.forum-sidebar` / `.sidebar-header` / `.sidebar-body` / `.sidebar-footer` 侧栏样式
- 新增 `.main-area` 主区域样式
- 调整 `.app-header` / `.app-footer` 圆角以适应新布局（只保留右侧圆角）
- 添加 `.forum-sidebar.collapsed` 折叠状态（width/opacity/padding 过渡动画）

### components.css
- 移除 `.modal-overlay` / `.modal-panel` 等弹窗样式
- 保留 `.forum-toolbar` / `.forum-table` / `.pagination` 等表格样式
- 移除重复的 `@keyframes fadeIn`

### app.js
- `btnForum` / `forumModal` / `forumClose` → `btnForumSidebar` / `forumSidebar` / `sidebarClose`
- 弹窗显示/隐藏逻辑 → `collapsed` class 切换 + `active` class 切换
- 打开侧栏时自动加载论坛文件列表

## 效果

- 左侧 44px 活动栏始终可见，点击 🌐 图标展开/收起论坛侧栏
- 侧栏 340px 宽，带过渡动画（类似 VSCode 行为）
- 不再有弹窗遮盖主内容，不会自动弹出，可正常关闭

### 来自 changes/2026-05-14-114750.md

# 新增：自定义技能连招功能

## 改动

### index.html
- 活动栏新增 🔗 按钮 (`#btn-combo-sidebar`)，用于打开技能连招面板
- 侧栏改造为双面板：`#forum-panel`（论坛）+ `#combo-panel`（连招），通过 `hidden` 切换
- 连招面板包含连招序列区 (`#combo-sequence`) 和可用技能池 (`#combo-pool-list`)
- 结果统计区的三个标签增加 ID（`#stat-label-1/2/3`），支持 JS 动态修改文案

### components.css
- 新增第 12 节「技能连招面板」样式
- `.combo-chip`：连招序列中的技能卡片（带序号圆标 + 移除按钮）
- `.combo-pool-skill`：可用技能列表项（带添加按钮 + 会心伤害数值）
- `.combo-add.in-combo`：已加入连招的技能显示绿色 ✓
- `.combo-empty`：空状态提示
- `slideIn` 动画：技能卡片加入时从左滑入

### app.js
- 新增 `btnComboSidebar` / `sidebar` / `forumPanel` / `forumClose` / `comboPanel` / `comboClose` / `comboSequence` / `comboPoolList` / `comboClear` DOM 引用
- 新增连招状态变量：`comboSkillNames` / `lastResults` / `lastCritRate`
- 新增侧栏切换逻辑：`openSidebar(panel)` / `closeSidebar()` / `toggleSidebar(panel)`
- 论坛和连招共用同一个侧栏容器，通过 `activeSidebar` 状态切换面板和按钮激活态
- 新增连招函数：`addToCombo()` / `removeFromCombo()` / `renderComboSequence()` / `renderComboPool()`
- 计算结果存储到 `lastResults`，每次添加/移除连招技能时重新渲染结果统计
- 计算按钮点击时计算会心率：`max(0, huixin_dengji - yujin_dengji) / 197703 * 100`

### ui.js
- `renderResults` 新增 `comboInfo` 参数，传给 `_updateStats`
- `_updateStats` 新增连招模式：统计面板显示连招会心总伤害 / 连招期望总伤害 / 会心率
- 无连招时恢复常规统计（最大期望 / 平均期望 / 会心占比）
- `_formatNumber` 改为导出，供 app.js 中的 renderComboPool 使用

## 功能说明

1. 点击左侧活动栏 🔗 按钮打开技能连招面板
2. 先点击「开始计算」获取技能列表，技能会显示在「可用技能」区
3. 点击技能旁的 + 按钮将其加入连招序列，再次点击 ✓ 按钮移除
4. 连招序列中的技能可逐个 ✕ 移除，或点击「清空连招」一键重置
5. 添加/移除连招时，底部统计面板自动切换为连招数据：
   - 连招会心：所有连招技能会心伤害（H 段）之和
   - 连招期望：所有连招技能期望伤害（Q 段）之和
   - 会心率：基于玩家会心与目标御劲差值计算的会心几率
6. 清空连招后恢复常规统计

## 备注

- 会心率计算公式：`(会心等级 - 御劲等级) / 197703 × 100%`（与 Rust 后端 `PlayerConfig::guo_huixin()` 一致）
- 纯前端实现，无需修改 Rust 代码

### 来自 changes/2026-05-14-120102.md

# Restructure `server_tools/` into project group, add forum server

## Changes

- **`server_tools/`** is now a project group folder containing multiple crates:
  - **`server_tools/manifest-gen/`** — the original manifest generator (moved from `server_tools/`)
  - **`server_tools/forum/`** — new axum-based web forum for uploading/downloading `.toml` data files

- Updated root `Cargo.toml` workspace members: `server_tools` → `server_tools/manifest-gen`, `server_tools/forum`
- Updated `AGENTS.md` to reflect new structure and forum commands

## Forum server details

- Built with `axum` (async), serves on port 8080 by default (configurable via `PORT` env var)
- Stores uploaded files in `forum_data/` directory (configurable via `FORUM_DATA_DIR` env var)
- Features:
  - **GET /** — HTML page with upload form (drag-and-drop) and file listing table
  - **GET /api/files** — JSON list of all uploaded files (name, size, last-modified)
  - **POST /upload** — multipart file upload (validates `.toml` extension, sanitizes filename)
  - **GET /download/{filename}** — download any uploaded `.toml` file
- Filename sanitization prevents path-traversal attacks
- Only `.toml` files are accepted (matching the project's data file format)

### 来自 changes/2026-05-14-121350.md

# Add forum file list & download to Tauri app

## Changes

### Rust backend (`examples/jpcg_app/src-tauri/`)

- **`Cargo.toml`**: Added `reqwest` dependency (workspace) for HTTP requests to the forum server
- **`src/commands/mod.rs`**: Added two new Tauri commands:
  - `forum_list_files(forum_url)` — fetches `.toml` file list from `{forum_url}/api/files`
  - `forum_download_file(forum_url, filename)` — downloads a file from `{forum_url}/download/{filename}`, saves to `{exe_dir}/data/pvp36500/`
- **`src/lib.rs`**: Registered both new commands in `invoke_handler`

### JS frontend (`examples/jpcg_app/src/`)

- **`js/config.js`**: Added `FORUM_URL` constant (default `http://localhost:8080`), added `forumListFiles`/`forumDownload` to `TAURI_COMMANDS`
- **`js/api.js`**: Added `forumListFiles()` and `forumDownloadFile()` wrappers with mock fallbacks
- **`index.html`**: Added forum button (🌐) in header, added forum modal with file table, search bar, pagination controls
- **`js/app.js`**: Added forum button click handler → opens modal, loads file list from forum, supports pagination (10 files/page), search filtering, and one-click download
- **`css/components.css`**: Added styles for forum modal overlay/panel, file table, pagination, and download button

### 来自 changes/2026-05-14-122459.md

# 侧栏改造：从独立面板改为标签页

## 改动

将侧栏中论坛和连招两个独立面板改造为统一的标签页模式，顶部标签栏切换。

### index.html
- 移除两个面板各自的 `.sidebar-header`（含独立标题和关闭按钮）
- 新增 `.sidebar-tabs` 标签栏，包含两个标签（🌐 论坛 / 🔗 连招）和一个共用关闭按钮（✕）
- 标签使用 `data-tab` 属性区分（`forum` / `combo`）

### style.css
- 移除 `.sidebar-header` / `.sidebar-header h2` 样式
- 新增 `.sidebar-tabs` / `.sidebar-tab` / `.sidebar-tab.active` / `.sidebar-tab-close` 样式
- 激活的标签底部显示 2px 紫色指示线（`::after`）

### app.js
- DOM 引用：`forumClose` / `comboClose` → `sidebarClose`，新增 `sidebarTabs`
- 侧栏逻辑从 `openSidebar`/`closeSidebar`/`toggleSidebar` 重构为：
  - `switchTab(tab)` — 切换面板、活动按钮、标签激活态
  - `openSidebar(tab)` — 展开侧栏 + 切到指定 tab
  - `closeSidebar()` — 收起侧栏、清除所有激活态
- 活动栏按钮：再次点击已激活的按钮 → 折叠侧栏；点击不同按钮 → 切 tab
- 标签点击：侧栏折叠时展开，展开时直接切面板

## 使用方式

- 点击 🌐 或 🔗 打开侧栏并切换到对应标签
- 点击标签栏的「论坛」或「连招」在面板间切换
- 点击 ✕ 或再次点击活动栏按钮收起侧栏

### 来自 changes/2026-05-15-220349.md

# 更名：移除"剑心"字样，改为 JX3 PVP 计算器

## 改动

为避免版权问题，将项目中所有面向用户的"剑心"字样替换为"JX3 PVP"，项目标识符 `JPCG` 保持不变。

| 文件 | 修改内容 |
|------|----------|
| `AGENTS.md:1` | `JPCG — 剑心PVP计算器` → `JPCG - JX3 PVP 计算器` |
| `index.html:6` | `<title>JPCG — 剑心计算器` → `<title>JPCG - JX3 PVP 计算器` |
| `index.html:92` | `<h1>JPCG — 剑心计算器` → `<h1>JPCG - JX3 PVP 计算器` |
| `forum/src/main.rs:32` | `<title>剑心数据分享论坛` → `<title>JX3 PVP 数据分享论坛` |
| `forum/src/main.rs:66` | `<h1>剑心数据分享论坛` → `<h1>JX3 PVP 数据分享论坛` |
| `jpcg_core/src/lib.rs:2` | `// jpcg_core — 剑心计算核心库` → `// jpcg_core - JX3 PVP 计算核心库` |
| `style.css:3` | `JPCG 剑网3 伤害计算器` → `JPCG - JX3 PVP 计算器` |

## 备注

- 不涉及 crate 名、目录结构、Tauri identifier（`com.qinthirteen.jpcg`）或服务端 URL
- 间隔符统一使用空格 ` - `（而非 `—`）

### 来自 changes/2026-05-22-180250.md

# 后端代码结构整理

## 改动内容

### 1. 按领域拆分 Tauri commands

原 `commands/mod.rs`（251 行单体）拆为 5 个文件：

| 文件 | 职责 |
|------|------|
| `commands/mod.rs` | 仅 `pub mod` 子模块声明 |
| `commands/calculate.rs` | `calculate_damage` 命令 |
| `commands/config.rs` | `save_config_cmd` / `load_config_cmd` / `load_profession_config` |
| `commands/update.rs` | `TauriProgress` / `check_update` / `perform_update` |
| `commands/forum.rs` | `ForumFileInfo` / `forum_list_files` / `forum_download_file` |
| `commands/types.rs` | DTO 定义 + `into_core()` 转换（原样保留） |

`lib.rs` 中 `generate_handler!` 的命令路径改为完整子模块路径（如 `commands::calculate::calculate_damage`）。

### 2. 提取重复路径逻辑到 `io::data_dir()`

`cal.rs` 和 `io.rs` 中重复的 `std::env::current_exe() → parent → join("data/pvp36500")` 模式，提取为 `io::data_dir() -> Option<PathBuf>`。

### 3. 抽离 FFI 到独立模块

`lib.rs` 中的 `#[no_mangle] extern "C"` + `#[repr(C)]` 类型移到 `ffi.rs`，`lib.rs` 改为 `pub mod ffi;`。

### 未改动

- 计算逻辑（`cal.rs`/`atkcal.rs`）—— 零改动
- `jpcg_const`、`jcsx_set.rs`、`food.rs` —— 保留原样
- DTO 结构和字段名 —— 不变
- 所有 Cargo.toml 依赖 —— 不变

### 来自 changes/2026-05-22-181410.md

# 前端重写为 React + TypeScript

## 改动内容

### 1. 脚手架搭建
- `examples/jpcg_app/` 下新建 npm 项目：`package.json`、`tsconfig.json`、`vite.config.ts`
- 依赖：React 19、Vite 6、TypeScript 5.7、`@tauri-apps/api` v2
- Vite 构建输出到 `dist/`，`tauri.conf.json` 更新 `frontendDist` 为 `"../dist"`

### 2. 前端代码结构迁移

原 Vanilla JS 代码全部替换为 React + TypeScript + CSS Modules：

```
src/
  main.tsx                    # React 入口
  App.tsx                     # 根布局 + 状态管理
  vite-env.d.ts               # CSS Modules + __TAURI__ 类型声明
  api/commands.ts             # Tauri IPC 封装 + mock 降级
  hooks/
    useTheme.ts               # 主题切换 + localStorage
    useToast.ts               # Toast 通知管理
    useKeyboardShortcuts.ts   # 键盘快捷键（ESC 关闭侧栏）
  types/index.ts              # 前后端 DTO 类型定义
  utils/
    constants.ts              # XINFA_LIST / 字段定义
    format.ts                 # 数字格式化（万/亿）
    sanitize.ts               # _sanitizeNumbers 移植
    clsx.ts                   # 简单 className 拼接
  components/
    ActivityBar.tsx + .module.css    # 侧栏切换图标
    ConfigPanel.tsx + .module.css    # 配置表单 + 更新流程
    ResultTable.tsx + .module.css    # 伤害结果表格 + 骨架屏
    Sidebar.tsx + .module.css        # 论坛面板 + 排轴器
    StatusBar.tsx + .module.css      # 底部状态栏
    Toast.tsx + .module.css          # Toast 通知
    ThemeToggle.tsx + .module.css    # 深色/浅色切换
  styles/
    variables.css              # CSS 变量（玻璃拟态蓝紫主题）
    reset.css                  # 重置样式
```

### 3. 视觉变化
- 窗口 800×600 → 1200×800
- 蓝紫渐变色系（`#6366f1` → `#a855f7`）+ 毛玻璃卡片
- 暗色主题：深蓝灰基底 `#0b0b1a`
- 骨架屏 shimmer 动画替代文字 "计算中..."
- 结果表格行 hover 高亮，最高伤害行发光左边框
- DOT 技能金色标签

### 4. 排轴器增强
- 双击序列项移除（原 X 按钮移除）
- 显示各技能出现次数（如「宫 ×3」）
- 新增「清空序列」按钮
- 每一项显示序号

### 5. 删除的旧文件
- `examples/jpcg_app/src/index.html`
- `examples/jpcg_app/src/js/`（全部）
- `examples/jpcg_app/src/css/`（全部）
- `examples/jpcg_app/src/assets/`（全部）

### 已知问题（未改动）
- B/I 列语义倒置（计算逻辑相关，不动）
- `tauri.conf.json` 的 `beforeDevCommand` / `beforeBuildCommand` 需要 npm 可用

### 来自 changes/2026-06-02-032900.md

# 全门派 PVP 伤害计算器增强

## 改动内容

### 后端 (`jpcg_core`)

**新增类型文件** (`type_set/`):

| 文件 | 内容 |
|------|------|
| `buff.rs` | `BuffConfig` — 阵眼/奇穴增益（基础攻击%、会心/会效/破防/无视防御/伤害提升%，模式切换） |
| `coefficient.rs` | `CoefficientConfig` — 可配置系数（破防/会心/会效/化劲/防御系数 + PVP 全局减伤），默认值从公式常量提取 |
| `combo.rs` | `ComboStep`、`StepOverride`、`ComboPreset` — 连招序列、每步临时覆盖值、预设存储 |

**扩展现有类型**:
- `PlayerConfig` — 新增 `zuizhong_gongji` 字段 + `atk_with_buff()`、`guo_pofang_with()`、`guo_huixinxiaoguo_with()`、`guo_huixin_with()` 方法
- `HostilepileConfig` — 新增 `target_hp` 字段 + `guo_wfangyu_with()`、`guo_nfangyu_with()`、`guo_huajin_with()`、`guo_yujin_huixiao_with()`、`guo_yujin_huixin_with()` 方法
- `JpcgConfig` (`atkcal.rs`) — 新增 `new_with_config()` + `buff`/`coeff` 字段，公式常量替换为可配置值，阵眼增益集成到各段计算
- `SaveConfig` (`io.rs`) — 新增 `buff`/`coefficient` 字段（`#[serde(default)]` 向后兼容）

**击杀概率模块** (`cal/kill_prob.rs`):
- 基于中心极限定理 + 正态 CDF 近似（自定义 erf），不依赖第三方 rand crate
- 逐技能二项分布（会心/非会心）计算均值和方差
- 输出击杀概率曲线数据点、逐步骤明细

**IO 扩展** (`io.rs`):
- `combo_presets_dir()` — exe 父目录下的 `combo_presets/`
- `list_combo_presets()`、`load_combo_preset()`、`save_combo_preset()`、`delete_combo_preset()`
- `export_config_toml()`、`import_config_toml()`

### Tauri 命令

**新增命令**:

| 命令 | 位置 | 说明 |
|------|------|------|
| `load_skill_pool` | `data.rs` | 按心法名读取 TOML 返回技能池 |
| `calculate_combo_cmd` | `combo.rs` | 连招计算（含击杀概率、概率曲线） |
| `save_combo_preset` | `combo.rs` | 保存连招预设（`combo_presets/{name}.toml`） |
| `list_combo_presets` | `combo.rs` | 列出所有预设 |
| `load_combo_preset` | `combo.rs` | 加载预设 |
| `delete_combo_preset` | `combo.rs` | 删除预设 |
| `export_config_cmd` | `combo.rs` | 导出配置为 TOML 字符串 |
| `import_config_cmd` | `combo.rs` | 导入配置 |

**增强命令**:
- `calculate_damage` — 接收 `BuffConfigDTO` + `CoefficientConfigDTO`
- `load_config_cmd` — 返回 buff + coefficient 字段

**DTO** (`types.rs`): 新增 `BuffConfigDTO`、`CoefficientConfigDTO`、`SkillPoolItemDTO`、`StepOverrideDTO`、`ComboStepDTO`、`ComboPresetDTO`、`ComboStepResultDTO`、`ComboResultDTO`

### 前端

**新依赖**: `recharts`（图表）、`@hello-pangea/dnd`（React 19 兼容的拖拽 fork，替代 react-beautiful-dnd）

**ConfigPanel** (`ConfigPanel.tsx`):
- 新增【阵眼/奇穴增益】折叠区段（6 个输入）
- 新增【系数设置】折叠区段（6 个输入，默认填充公式常量值）
- 新增【目标血量(万)】输入
- 新增实时属性统计栏（会心率、破防率、减伤）
- 新增导出/导入配置按钮

**ComboPanel** (`Sidebar.tsx` 重写):
- 技能池从 TOML 文件读取（通过 `load_skill_pool` 命令），不依赖计算结果
- 技能池分页（复用 `PAGE_SIZE=10` 模式）
- 右键 ⭐ 标记最爱技能（localStorage 持久化）
- 点击添加连招、双击/按钮移除
- `@hello-pangea/dnd` 拖拽排序
- 每个技能实例 ⚙️ 按钮 → `StepAdjustModal`（8 个可调字段 + 恢复原始）
- 连招计算按钮 + `ComboResultDisplay`：
  - 总期望伤害 / 击杀概率 / 技能数摘要
  - recharts 击杀概率曲线（LineChart）
  - recharts 每步伤害对比（BarChart）
  - 逐步骤详细数据表格
- 连招预设管理（保存/加载/删除，全部通过 Tauri 命令读写文件）
- 预设加载下拉框复用论坛分页模式

## 文件清单

**新增**:
- `crates/jpcg_core/src/type_set/buff.rs`
- `crates/jpcg_core/src/type_set/coefficient.rs`
- `crates/jpcg_core/src/type_set/combo.rs`
- `crates/jpcg_core/src/cal/kill_prob.rs`
- `examples/jpcg_app/src-tauri/src/commands/data.rs`
- `examples/jpcg_app/src-tauri/src/commands/combo.rs`

**修改**:
- `crates/jpcg_core/src/type_set.rs` — 新增 3 个 pub mod
- `crates/jpcg_core/src/type_set/player.rs` — 新增字段 + 方法
- `crates/jpcg_core/src/type_set/hostilepile.rs` — 新增字段 + 方法
- `crates/jpcg_core/src/cal.rs` — 新增 `start_calculation_with_config` + `pub mod kill_prob`
- `crates/jpcg_core/src/cal/atkcal.rs` — 集成 BuffConfig/CoefficientConfig
- `crates/jpcg_core/src/io.rs` — SaveConfig 扩展 + 连招 IO + 导出/导入
- `crates/jpcg_core/src/lib.rs` — 新增 `combo_io`、`config_io`、`start_with_config`、`start_combo`
- `examples/jpcg_app/src-tauri/src/commands/mod.rs` — 新增模块
- `examples/jpcg_app/src-tauri/src/commands/types.rs` — 全部 DTO
- `examples/jpcg_app/src-tauri/src/commands/calculate.rs` — 增强
- `examples/jpcg_app/src-tauri/src/commands/config.rs` — 增强
- `examples/jpcg_app/src-tauri/src/lib.rs` — 注册新命令
- `examples/jpcg_app/src/types/index.ts` — 全部 TS 类型
- `examples/jpcg_app/src/api/commands.ts` — 全部 API 调用 + mock
- `examples/jpcg_app/src/utils/constants.ts` — 新增常量
- `examples/jpcg_app/src/App.tsx` — 传递 xinfaName
- `examples/jpcg_app/src/components/ConfigPanel.tsx` — 增益/系数/统计/管理
- `examples/jpcg_app/src/components/ConfigPanel.module.css` — 新样式
- `examples/jpcg_app/src/components/Sidebar.tsx` — ComboPanel 重写
- `examples/jpcg_app/src/components/Sidebar.module.css` — 大量新样式

## 设计决策

- **系数默认值**从 atkcal.rs/player.rs/hostilepile.rs 的公式常量提取，用户可覆盖
- **击杀概率**使用解析法（CLT + erf 近似），避免引入 `rand` 依赖
- **`#[serde(default)]`** 用于 `SaveConfig` 新字段，确保旧配置文件向后兼容
- **`@hello-pangea/dnd`** 替代 `react-beautiful-dnd`，因后者不兼容 React 19
- 技能池不与计算结果耦合——直接在 ComboPanel 内调用 `load_skill_pool`

### 来自 changes/2026-06-02-143000.md

# 数据文件命名规范 + 自动发现职业列表

## 改动内容

### TOML 命名规范

| 文件名 | 用途 |
|--------|------|
| `mowen.toml` | 莫问 130 级第 3 赛季 |
| `mowen_130v2.toml` | 莫问 130 级第 2 赛季（共存但不显示） |
| `zhoutian.toml` | 周天功 130 级第 3 赛季 |
| `_template.toml` | 模板（前导 `_`，扫描时跳过） |

文件名分组规则：去掉 `.toml`，取第一个 `_` 前的内容为职业键。

### TOML 新增 `[version]` 节

```toml
[version]
level = 130       # 等级
season = 3        # 赛季
modified = 20260602  # 修改日期 YYYYMMDD
```

### 分组去重排序

```
按职业键聚合 → 组内排序：
  1. level 降序
  2. season 降序
  3. modified 降序
→ 取每条作为该职业的活跃配置
```

### 数据结构

| 层 | 改动 |
|----|------|
| `type_set/xinfa.rs` | 新增 `VersionInfo` (level/season/modified) + `XinfaSummary` (value/label/nom/version_label)；`get_xinfa_list()` 改为调用 `io::list_available_professions()` |
| `io.rs` | `TomlConfig` 新增 `version: Option<VersionInfo>`；新增 `list_available_professions()` 扫描+分组+排序+去重 |
| `lib.rs` | 新增 `pub mod profession_list` 导出 `list_available()` |
| `config.rs` (Tauri) | 新增 `list_professions_cmd` |
| `types.rs` (Tauri) | 新增 `XinfaSummaryDTO` |
| `lib.rs` (Tauri) | 注册 `list_professions_cmd` |
| `api/commands.ts` | 新增 `listProfessions()` + mock 返回完整列表 |
| `types/index.ts` | 新增 `XinfaSummaryDTO` |
| `ConfigPanel.tsx` | 启动时调用 `listProfessions()` 动态获取职业列表；下拉显示 `"莫问 (130级第3赛季)"` |
| `constants.ts` | `XINFA_LIST` 保留为 fallback（未改动） |

### 数据文件

| 操作 | 文件 |
|------|------|
| 重命名 | `template.toml` → `_template.toml` |
| 添加 `[version]` | `mowen.toml` (level=130, season=3, modified=20260602) |
| 添加 `[version]` | `zhoutian.toml` (level=130, season=3, modified=20260602) |
| `mowen.toml` | 修复 `xinfa_name` = "莫问"（原为 "mowen"）、`xinfa_nom` = "根骨"（原为 "gengu"）、补 `huixin_up` |
| `zhoutian.toml` | 补 `huixin_up = 0.0` |

### 来自 changes/2026-06-03-122538.md

# v2.1.0-alpha.1: 版本号更新 + 品牌名统一

## 变更内容

### 版本号
- 将根目录 `Cargo.toml` (workspace.package) 版本号从 `2.0.0-alpha.0` 升级到 `2.1.0-alpha.1`
- 同步更新以下文件的版本号到 `2.1.0-alpha.1`:
  - `examples/jpcg_app/src-tauri/Cargo.toml`
  - `examples/jpcg_app/src-tauri/tauri.conf.json`
  - `examples/jpcg_app/package.json`

### 品牌名
- 移除所有「剑心」字样，统一使用「剑网3PVP计算器（JPCG）」:
  - `tauri.conf.json`: productName / 窗口标题
  - `index.html`: `<title>`
  - `App.tsx`: Logo 文字
  - `jpcg_core/src/lib.rs`: 注释改为 `JPCG 计算核心库`
  - `forum/src/main.rs`: 论坛标题改为「剑网3PVP数据分享论坛」

### 来自 changes/2026-06-03-123632.md

# Fix: 心法技能 TOML 加载使用 profession key 而非中文名

## Bug

`cal.rs:60` 使用 `xinfa.xinfa_name`（中文名如"莫问"）拼接文件路径，但实际文件名为 `mowen.toml`，导致计算时报 `No such file or directory`。

## 修复

在 `XinfaConfig` 中新增 `profession: String` 字段存储拼音 key，并贯穿全调用链：

- **`type_set/xinfa.rs`**: `XinfaConfig` 新增 `profession` 字段
- **`io.rs`**: `load_config()` 和 `list_available_professions()` 反序列化后注入 `profession`
- **`cal.rs`**: 改用 `xinfa.profession` 拼接文件路径（关键修复）
- **Tauri DTO** (`types.rs`, `config.rs`): 传递 `profession` 字段
- **Frontend** (`types/index.ts`, `ConfigPanel.tsx`, `commands.ts`): 设置/透传 `profession`

## 数据流（修复后）

```
前端 xinfa_config.profession = "mowen"
  → XinfaConfigDTO.into_core()
  → cal.rs: dir.join("mowen") → data/shuxing/mowen.toml ✅
```

### 来自 changes/2026-06-03-124418.md

# Fix: 切页后表单数据丢失

## Bug

`App.tsx` 使用 `{curPage === "calc" && <ConfigPanel />}` 条件渲染，切页时组件被 unmount，内部 `form` state 丢失。

## 修复

将 `curPage === "calc"` 条件渲染改为 CSS `display: none` 控制显隐，确保 `<ConfigPanel>` 始终 mounted，表单状态不丢失。

### 来自 changes/2026-06-03-130803.md

# 论坛页面：已下载状态 + 删除已下载功能

## 改动内容

论坛页面的文件列表现在会显示哪些文件已下载到本地，并支持删除已下载的文件。

### Rust 后端 (`forum.rs`)

- 提取 `download_dir(category)` 辅助函数，消除 `forum_download_file` 中的路径逻辑重复
- 新增 `forum_list_downloaded` Tauri 命令：扫描本地下载目录，返回已下载的 `.toml` 文件名列表
- 新增 `forum_delete_downloaded` Tauri 命令：删除本地下载目录中的指定 `.toml` 文件（含 `confirm` 确认）

### 命令注册 (`lib.rs`)

- 在 `generate_handler!` 中注册 `forum_list_downloaded` 和 `forum_delete_downloaded`

### 前端 API (`commands.ts`)

- 新增 `forumListDownloaded(category?)` 函数
- 新增 `forumDeleteDownloaded(filename, category?)` 函数
- 添加对应的 mock 响应（模拟已下载 `mowen.toml` 和 `template.toml`）

### 论坛页面 UI (`ForumPage.tsx`)

- 接收 `addToast` prop 用于操作反馈
- 新增 `downloadedFiles` 状态 + `refreshDownloaded()` 回调
- 并行加载远程文件列表和本地已下载列表
- 表格中已下载文件显示绿色"已下载"标签 + "删除"按钮；未下载文件显示"下载"按钮
- 下载/删除后自动刷新已下载状态

### 样式 (`ForumPage.module.css`)

- `.actionCell` — 操作列 flex 布局
- `.downloadedBadge` — 绿色"已下载"标签
- `.deleteBtn` — 红色边框删除按钮（hover 填充）

### App 集成 (`App.tsx`)

- 向 `<ForumPage>` 传入 `addToast`

### 来自 changes/2026-06-06-141500.md

变更: 重构攻击力计算——移除未使用的 `jcsx_to_atk` 方法，参数重命名

- 移除 `PlayerConfig::jcsx_to_atk()`（未使用、公式已废弃）
- `atk()` 和 `atk_with_buff()` 参数 `atk_up` → `shuxing_atk_up`，明确表示基础属性攻击加成
- 更新对应注释

### 来自 changes/2026-06-06-170900.md

# 新增技能编辑器模块

## 改动

新增完整的技能编辑器模块，允许用户在 app 中直接编辑心法数据文件。

### 后端（jpcg_core）

| 文件 | 修改内容 |
|------|----------|
| `type_set/skilltype.rs:3` | 导入 `Serialize`；添加序列化辅助函数（`is_zero_u32` 等）；所有字段添加 `#[serde(skip_serializing_if)]` 以跳过默认值 |
| `io.rs:75` | `TomlConfig` 添加 `Serialize` derive |
| `io.rs:282` | 新增 `save_skill_toml()` 函数：序列化 `TomlConfig` 并写入 `{data_dir}/{profession}.toml` |
| `lib.rs:13` | 新增 `pub mod skill_editor` 公开模块：`load_skills()` / `save_skills()` |

### Tauri 命令层

| 文件 | 修改内容 |
|------|----------|
| `commands/types.rs` | 新增 `SkillEditorItemDTO`、`VersionInfoDTO`、`SkillEditorDataDTO` 及对应的 `From` 互转实现 |
| `commands/skill_editor.rs` | **新建**：`load_skill_data` / `save_skill_data` 两个 Tauri 命令 |
| `commands/mod.rs` | 注册 `pub mod skill_editor` |
| `lib.rs` | 注册 `skill_editor::load_skill_data`、`skill_editor::save_skill_data` 到 `.invoke_handler()` |

### 前端

| 文件 | 修改内容 |
|------|----------|
| `types/index.ts` | 新增 `SkillEditorItemDTO`、`VersionInfoDTO`、`SkillEditorDataDTO` 接口 |
| `api/commands.ts` | 新增 `loadSkillData()` / `saveSkillData()` 函数 + mock 响应 |
| `components/SkillEditorPage.tsx` | **新建**：技能编辑器主页面组件，含技能列表 + 详情编辑面板 |
| `components/SkillEditorPage.module.css` | **新建**：对应样式文件 |
| `components/ActivityBar.tsx` | 导出 `Page` 类型；新增 `"editor"` 导航项（✏️ 技能编辑） |
| `App.tsx` | 导入 `SkillEditorPage`；`curPage` 类型包含 `"editor"`；添加路由渲染 |

### 交互设计

- 左侧列表显示当前心法的所有技能（点击选中）
- 右侧详情面板按分组展示技能属性（基础信息 / 伤害系数 / 增益穿透 / 标签Dot）
- 部分字段使用下拉选择（design_effect、kind_type、cast_mode、dot_flag）
- 支持添加新技能 / 删除选中技能
- 顶部心法下拉切换 + 保存按钮

### 备注

- `Skilltype` 序列化时使用 `skip_serializing_if`，确保 TOML 输出干净简洁
- 保存路径与 `load_config` 使用相同的 `data_dir()`，开发模式下为仓库内的 `data/shuxing/` 目录
- Tauri 打包后写权限问题需后续处理：改为先读 bundle 内文件，保存到可写目录

### 来自 changes/2026-06-19-134339.md

# 自动求导 + 加点优化模块

## 改动

### 后端 — `crates/jpcg_core`

- **新增 `cal/autodiff.rs`**：
  - `compute_derivatives()` — 对 6 个属性（基础属性、基础攻击、会心等级、会心效果、破防等级、武器伤害）执行中心差分数值微分，计算 d(total_Q)/d(attr) 和每技能点的边际收益。
  - `optimize_attributes()` — 贪心算法，每次将 1 技能点分配给当前边际收益最高的属性（可选重算间隔，默认每 `max(10, total/10)` 点重算一次导数）。
  - `PointConversion` — 根据心法（内功/外功）选择对应的技能点→属性转换率（根骨/元气: 994 基础攻击，力道/身法: 891; 基础属性 420; 破防/会心/会效 3279; 武器伤害 1344）。
  - 结果类型 `DerivativeEntry`、`AttributeSuggestion`、`OptimizeResult` 均派生 `Serialize`。
- **`cal.rs`** — 注册 `pub mod autodiff`。
- **`lib.rs`** — 新增 `pub mod autodiff` 封装模块，委托给 `cal::autodiff`。

### 后端 — `examples/jpcg_app` (Tauri)

- **新增 `commands/optimize.rs`**：
  - `compute_derivatives` — Tauri 命令，接收 `OptimizeRequest`，加载技能 TOML，调用 autodiff 模块。
  - `optimize_attributes` — Tauri 命令，同上 + `total_points` 参数。
- **`commands/types.rs`** — 新增 `OptimizeRequest`、`DerivativeEntryDTO`、`AttributeSuggestionDTO`、`OptimizeResultDTO` + `From` trait 转换。
- **`commands/mod.rs`** — 注册 `optimize` 模块。
- **`lib.rs`** — 注册 `compute_derivatives` 和 `optimize_attributes` 到 `invoke_handler`。

### 前端

- **`types/index.ts`** — 新增 `OptimizeRequest`、`DerivativeEntryDTO`、`AttributeSuggestionDTO`、`OptimizeResultDTO`。
- **`api/commands.ts`** — 新增 `computeDerivatives()`、`optimizeAttributes()` + mock 响应。
- **新增 `components/OptimizePage.tsx`** — 加点优化页面：点数预算输入、导数计算、边际收益排行、优化结果展示。
- **新增 `components/OptimizePage.module.css`** — 优化页样式。
- **`components/ActivityBar.tsx`** — 新增 `"optimize"` 页面类型和 📈 加点优化按钮。
- **`App.tsx`** — 导入 `OptimizePage`，添加路由渲染。

## 设计决策

- 使用**数值微分（中心差分）**而非解析求导，因为伤害公式链包含多次 `as u32` 整数截断，解析导数不连续。
- 导数显示使用 `h=1` 的中心差分（dQ/d(属性值)），边际收益使用 `h=conversion_rate` 的前向差分（每 1 技能点的 Q 增量）。
- 转换率归一化以 1 基础属性 = 1.0 为基准，内功/外功自动区分。
- 贪心优化每 `max(10, total_points/10)` 点重算一次导数以避免不必要的计算开销（导数在连续分配同属性时变化较小）。
- 技能数据从 TOML 文件加载（复用 `skill_editor::load_skills`），而非通过请求参数传递。

### 来自 changes/2026-06-19-213146.md

# v2.1-beta 准备：Issues 按钮 + Beta 切换 + 清理调试信号

## 改动

### 1. 快速打开 Issues 按钮（`constants.ts` + `App.tsx` + `App.module.css`）

- `utils/constants.ts`: 新增 `GITHUB_ISSUES_URL` 常量
- `App.tsx`: header 右侧新增 🐛 反馈按钮，点击调用 `window.open` 跳转到 GitHub Issues 页面
- `App.module.css`: 新增 `.headerActions`（flex 容器）、`.headerBtn`（与 ThemeToggle 一致的样式）

### 2. Beta 版本切换 UI（`ConfigPanel.tsx` + `ConfigPanel.module.css` + `constants.ts`）

- `utils/constants.ts`: 新增 `STORAGE_KEYS.betaChannel`
- `ConfigPanel.tsx`:
  - 新增 `betaChannel` state，从 localStorage 初始化
  - 新增 `handleBetaToggle`，切换时写入 localStorage
  - `handleUpdateClick` 读取 `betaChannel` 替代硬编码 `false`
  - actions 区域新增 checkbox "Beta 版本"
- `ConfigPanel.module.css`: 新增 `.betaToggle` 样式

### 3. 去掉调试信号（`crates/jpcg_update/src/download.rs`）

| 位置 | 处理 |
|------|------|
| `check_binary_update_needed` 中的 2 处 println | **移除** — 纯调试输出 |
| `determine_other_updates_by_hash` 中的 2 处 println | **移除** — 哈希对比细节调试输出 |
| `check_data_updates` 中的 2 处 println | **移除** — 数据文件对比调试输出 |
| `replace_file_or_prompt` 中的 3 处 println | **`#[cfg(debug_assertions)]` 编译门** — 保留 debug 构建可见 |
| `download_and_parse_manifest` 中的哈希类型警告 | **`#[cfg(debug_assertions)]` 编译门** |
| `decompress_package_with_external_tool` 中的 2 处 println | **移除** — 调用方已有 UI 输出 |

保留 CLI 交互路径 (`prompt_and_perform_update`, `all_updates`) 中的 println，这些是 CLUI 的正常输出。

### 来自 changes/2026-06-19-220416.md

# 自动更新完整流程：更新器 binary + 应用二进制更新

## 改动

### 1. 新增 `crates/jpcg_updater/` — 独立更新器二进制

轻量级 standalone binary，零外部依赖，仅使用 std。接收命令行参数：

```
jpcg_updater <父进程PID> <旧程序路径> <新程序路径> <工作目录>
```

流程：
1. 轮询等待父进程退出（每 300ms）
2. 额外等待 1 秒确保文件句柄释放
3. 删除旧程序 → 重命名/复制新程序
4. 设置可执行权限（unix）
5. 启动新版本
6. 更新器退出

### 2. 扩展 `jpcg_update`

- `download.rs`: 新增 `AppUpdateInfo` 结构体（`download_url`, `expected_hash`, `binary_path`, `version`）
- `lib.rs`: 新增 `fetch_app_update_info()` 函数，根据当前平台选择匹配的二进制并返回下载信息

### 3. 新增 Tauri 命令 `perform_app_update`

`examples/jpcg_app/src-tauri/src/commands/update.rs`:
- 调用 `jpcg_update::fetch_app_update_info` 获取二进制信息
- 通过 `download_file_with_progress` 下载并发送进度事件
- SHA256 哈希验证
- 查找更新器路径（先从 exe 同目录查找，开发模式回退到 `target/debug/`）
- 启动更新器进程（传入 parent PID、exe 路径、temp 路径、工作目录）
- 延迟 300ms 后调用 `app_handle.exit(0)` 退出当前进程

`lib.rs`: 注册 `perform_app_update` 命令

### 4. 前端集成

- `api/commands.ts`: 新增 `performAppUpdate(beta)` 函数 + mock 响应
- `ConfigPanel.tsx`: `handleUpdateClick` 中当 `has_app_update` 为 true 时：
  - 弹出 `confirm()` 询问用户是否下载并重启
  - 确认后调用 `performAppUpdate`，实时显示进度
  - 回复 "重启中..." 后 APP 自动退出，更新器接管

### 5. Tauri bundle

`tauri.conf.json`: 新增 `externalBin: ["binaries/jpcg_updater"]`，打包时自动包含更新器

### 6. 服务器 manifest 文档

`server_manifest.md`: 详细记录了 update.toml / manifest.toml 格式、目录结构、Beta/稳定版差异

### 7. AGENTS.md

更新 workspace 表格、构建命令文档

## 数据流

```
Frontend                      Tauri                         jpcg_update            jpcg_updater
   │                            │                              │                      │
   ├─ checkUpdate() ──────────► ├─ check_updates() ──────────► │                      │
   │◄── has_app_update: true ── │                              │                      │
   ├─ (confirm dialog)          │                              │                      │
   ├─ performAppUpdate() ────► ├─ fetch_app_update_info() ──► │                      │
   │                            │◄── AppUpdateInfo ─────────── │                      │
   │◄── progress events ─────── ├─ download_file_with_progress │                      │
   │                            ├─ verify SHA256               │                      │
   │                            ├─ spawn updater ───────────────────────────────────► │
   │                            ├─ app_handle.exit(0)          │                      │
   │                            │  (process dies)              │                  wait for parent PID
   │                            │                              │                  to exit
   │                            │                              │                  replace binary
   │                            │                              │                  launch new app
   │                            │                              │                  exit

### 来自 changes/2026-08-09-100000.md

# core/update P1 — 热路径去 clone 与求导去重

日期: 2026-08-09

## 做了什么

1. **`JpcgConfig` 引用化** (`cal/atkcal.rs`)
   - 结构体由持有 6 个 owned 配置改为持有 6 个引用（`&PlayerConfig` 等），增加生命周期参数。
   - 删除无调用方的 `new()`（此前已无人使用），`new_with_config` 全参数改引用。
   - 消除 `call_back`/`compute_derivatives`/`calculate_combo` 三个循环内对
     player/hostilepile/skill/xinfa/buff/coeff 的逐技能全量 clone。

2. **求导去掉二次全链** (`cal/atkcal.rs` `q_cal_with_derivatives`)
   - 原实现先调 y_cal/i_cal/g_cal/h_cal，再调 `q_cal()` 把整条链再算一遍（×2 全链）。
   - 现复用 forward 阶段中间结果，Q 段就地计算，数值公式与 `q_cal()` 完全等价：
     `q = G×(1-crit) + H×crit`，结果数组 `[y, i1, i2, G3, H4]`。

3. **顺带清理** (`type_set/player.rs`)
   - 删除未使用的 `use crate::log::error;` 导入。

## 为什么

- 每个技能重复 clone 6 个结构化配置并重跑全链，是纯冗余计算。引用化 + 复用后，大量 `derive(Clone)` 调用消失，求导路径计算量约减半。

## 约束与保障

- **不改变任何数值行为**：Q 段与结果数组构造逐条对照原 `q_cal()` 等价。
- 数值一致性将由后续 P4 基准单测锁定。
- 未触碰截断/取整逻辑（P2 另行显式化，仍是逐位一致）。

## 其他

- `jpcg_app`（Tauri）先前 cargo check 失败，原因是缺少 `src-tauri/binaries/jpcg_updater-aarch64-apple-darwin`（见 server_manifest.md 预置步骤），已按文档补上，workspace 全量 check 通过。
- 剩余 4 个 dead-code 警告为既有（`Config::load`、`data_load`、`DamageResultWithDerivatives.result` 等），本次未处理。

### 来自 changes/2026-08-09-103000.md

# core/update P2 — 截断逻辑显式化（数值逐位一致）

日期: 2026-08-09

## 做了什么

`cal/atkcal.rs` `g_cal()`:

原先一整行链式 `as u32` cast（嵌套 6 层括号），把游戏实测的 3 个截断点隐在其中。
现拆分为显式 `truncate()` 辅助函数 + 三步注释：

1. ① `I2 × (1+命中增益) × 伤害增益 × (Y/1024)` → 截断
2. ② `× (1 − 化劲/1024)` → 截断
3. ③ `× 全局PVP减伤 × (1 − 目标减伤比)` → 截断输出

新增 `truncate(v: f32) -> f32`：`v as u32 as f32`，与游戏实测截断语义一致。

## 为什么

把「游戏实际检测得出的截断顺序」从嵌套 cast 迷宫中显式化，配上中文注释固定语义，避免后续重构误改或误优化。**这是纯形式化重构。**

## 逐位一致保障

- 运算顺序与优先级完全复刻原表达式（乘左结合，`pvp` 在最后一步内部乘），无新增/移除任何运算。
- 每步取整点与原 `as u32` 出现位置一一对应。
- P4 将以改动前行为为基准做等价测试（快照），双重保险。

## 决策

- 未统一 buff 百分比换算的 `*1024/100`（1024 制系数）与 `/100`（0~1 比率）两种写法 —— 二者语义分别对应系数域与比率域，混写是有意区分，不存在不一致，故不动。
- `y_cal`/`h_cal`/`i_cal` 各自仅有一个 `as u32` 单点截断，本就显式，未触碰。

### 来自 changes/2026-08-09-110000.md

# core/update P3 — toml_input "none" 哨兵改 Option，清理 unwrap_or

日期: 2026-08-09

## 做了什么

1. **`io.rs` `toml_input`** 返回类型 `String` → `Option<String>`：
   - 不再用 `"none"` 字符串充当「文件不存在」哨兵；
   - 文件缺失/读取失败返回 `None`，成功返回 `Some(内容)`。

2. **调用点适配**（公共行为不变）：
   - `cal.rs` 步骤3：`"none"` 判断 → `None => TomlConfig::default()`；
   - `io.rs::load_config`：`None` 直接默认空配置（去掉原先对 `"none"` 再做 from_str 失败的误导日志）；
   - `io.rs::load_save_config`：`None` 时走既有的「未找到配置，使用默认值」warn 路径；
   - `type_set/jcsx_set.rs::data_load`（死代码，预留模块）：`unwrap_or_default()` 适配新签名。

3. **`io.rs::load_config`** `to_str().unwrap_or("").to_string()` + 空串再判断
   → 显式 `match` 处理非法 UTF-8（符合 AGENTS.md 库代码约束）。

## 为什么

- `"none"` 哨兵会误导后续维护：一旦调用方忘记判 `"none"`，会直接 `toml::from_str("none")` 拿到伪装错误；Option 类型强制穷举，编译器兜底。
- `unwrap_or("")` 把非法 UTF-8 路径折叠成空字符串再二次判断，绕弯且易漏。

## 影响面

- 仅为 `jpcg_core` 内部模块使用（`io` 是私有 mod，外部 crate 无直接依赖此符号）；Tauri 侧公共入口（load_config/save_config 等）签名均未变。
- feature 分支 `codex/attribute-config-editor` 不与 `toml_input` 调用冲突（已核实）。

### 来自 changes/2026-08-09-113000.md

# core/update P4 — 金标准基准测试

日期: 2026-08-09

## 做了什么

`cal/atkcal.rs` 新增 `#[cfg(test)] mod golden_tests`，6 个测试锁定五段伤害 + 6 属性导数：

| 测试 | 场景 |
|------|------|
| golden_gong_default / gong_buff | 莫问「宫」（base 160/200, atk 2.609375），默认 / 满 buff |
| golden_zheng_default / zheng_buff | 「徵·豪情」（hit_up=20），默认 / 满 buff |
| golden_wei_default / wei_buff | 「剑·徵·削竹」（watk_xishu=100, wushifangyu=90），默认 / 满 buff |

玩家样本: 根骨 18888 / 基础攻击 4666 / 会心 33000 / 会效 22000 / 破防 25000 / 武器 2800。
目标样本: 21000 防御 / 御劲 8500 / 化劲 35000 / 减伤 35%。
buff 场景: 基础攻击 +10%、会心 +5%、会效 +8%、破防 +3%、伤害 +6%。

## 基准来源（关键）

在**改动前的代码**（master `b12818f` 的 jpcg_core，git worktree 临时检出）上，用同输入跑同一探针程序，
得到全部金标准数值；再用同探针跑**改动后**代码（P1+P2 后），输出 `diff` 为**空**，
实证重构前后数值逐位一致，随后将数值固化进断言。

## 保障

- 六项断言对 Q 段精度 `1e-6`（f32 求导为连续近似，基准即原输出）。
- 若未来改动伤害公式/截断顺序，测试立即红。
- `cargo test -p jpcg_core`：6 passed。
- 探针代码已从工作树移除，临时 worktree 已删除。

### 来自 changes/2026-08-10-181440.md

# 同步远端技能编辑器的 TOML 处理边界

## 内容

- 合入远端的结构化技能编辑器接口 `load_skill_data` / `save_skill_data`。
- 移除旧属性编辑器的原始 TOML 文本读写链路，以及前端 `smol-toml` 解析和序列化逻辑。
- 清理仅服务于旧编辑器的页面、图标依赖和验证素材。

## 原因与决策

TOML 的解析、校验和写入统一由 Rust 核心完成；Tauri 在前端和后端之间只传输结构化 DTO。这样避免双端 TOML 实现产生格式和字段兼容性分歧，也与远端的技能编辑器实现保持一致。

### 来自 changes/2026-08-11-203000.md

# K1: jpcg_core 内部分组（io.rs → store/, cal/ → engine/）

## 什么
- `src/io.rs`（396 行）按职责拆分为 `src/store/` 子模块：
  - `store/paths.rs` — data_dir / combo_presets_dir（路径定位）
  - `store/toml.rs` — TomlConfig / toml_input / load_config / save_skill_toml
  - `store/config.rs` — SaveConfig / save_config / load_save_config / export / import
  - `store/combo.rs` — 连招预设 CRUD
  - `store/profession.rs` — 门派列表扫描
  - `store/mod.rs` — 声明 + 兼容重导出（保持 lib.rs 门面引用 `crate::store::`）
- `src/cal.rs` → `src/engine/mod.rs`，`src/cal/` → `src/engine/`（atkcal/derivatives/kill_prob 提升到 engine 下，不留嵌套）

## 为什么
- 为「core 即 host」架构铺路：store 与 engine 将成为 host 的独立子系统，未来可分别打包/替换
- 缩减单文件规模，职责边界清晰

## 关键决策
- `pub mod engine; pub use crate::engine as cal;` — 对外路径 `jpcg_core::cal::*` 保持不变，Tauri 侧零改动（上层的 `jpcg_core::cal::derivatives::` 等引用全部兼容）
- 删除死代码 `io::Config`（grep 确认无任何引用）
- `jcsx_set`/`all_updates`/`player.rs` 依决策保留（后续补充）
- store/mod.rs 顶层重导出全部函数，lib.rs 门面仅改 `io`→`store` 前缀，行为零变化

## 验证
- `cargo build --workspace`：0 error
- `cargo test -p jpcg_core`：6 个金标准全绿（数值逐位锁定，重构无行为变化）

## 备注
- 与 feature 分支 `codex/attribute-config-editor` 的 `io.rs`/`lib.rs` 冲突面已形成（该分支 +41/-? 行），合入时需手工协调

### 来自 changes/2026-08-11-205500.md

# K2: jpcg_api 纯类型契约 + core host/ JSON 入口 + FFI 重写 + 三 crate 双产物

## 什么
1. **新建 `crates/jpcg_api`**：仅含 serde DTO 类型的 rlib crate（22 个 DTO + ConfigDataDTO），零依赖，
   不引用 jpcg_core —— 作为 Tauri IPC 与 FFI JSON 的单一契约源（防漂移）
2. **core 新增 `host/` 模块**（JSON 契约层）：
   - `host/calc.rs` — calculate / compute_derivatives / into_core / skill_dto_to_skilltype
   - `host/combo.rs` — calculate_combo / 预设 CRUD / export/import config
   - `host/config.rs` — save/load_config / list_professions
   - `host/skill.rs` — load/save_skill_data / load_skill_pool
   - `host/conv.rs` — 原 Tauri types.rs 的全部 From 转换迁入（DTO↔core 双向）
   - `host/update.rs`（net 特性）— update 编排全权收归 core：check/download/验证/**发动
     jpcg_updater 子进程**/请求退出；异步经内嵌 tokio runtime 同步化；
     宿主行为通过 `HostEvents` trait（on_progress/request_exit/updater_path）注入
3. **`ffi.rs` 整体重写**（原 26 行裸指针 → 句柄 + JSON 协议）：
   - `jpcg_abi_version()` 版本协商、`jpcg_handle_create/free`、`jpcg_call(handle, method, request_json)`
   - `jpcg_free_string` / `jpcg_last_error` 错误传递
   - 所有实现经 `catch_unwind` 防护，panic 不跨 FFI
   - 覆盖 16 个业务方法（calculate/derivatives/combo/config/skill/预设 CRUD）
4. **三 crate 双产物**：`jpcg_core`/`jpcg_const`/`jpcg_update` 均 `crate-type = ["cdylib", "rlib"]`，
   产出 libjpcg_core.dylib / libjpcg_const.dylib / libjpcg_update.dylib
5. **`jpcg_const` FFI**：`jpcg_const_get_u32(key)` 按 key 查药物常量
6. **`jpcg_update` FFI**：`jpcg_update_check` / `jpcg_update_fetch_app_info` / `jpcg_update_file_sha256`
   （同步化封装，JSON 进出）
7. core 增加 feature `net`（默认开）：jpcg_update + tokio 为可选依赖

## 为什么
- B 模式（全动态 dlopen）的必要前提：core dll 对外只暴露 JSON 契约，Tauri 壳动态模式
  无需编译期依赖 core 类型
- 「更新时只更新某个 dll」依赖每 crate 独立 cdylib 产物

## 关键决策
- DTO 单源在 `jpcg_api`，From 转换单源在 `host/conv.rs`，Tauri 壳 types.rs 后续（K3）改为 `pub use`
- update 流程（含 updater 启动）从 Tauri 壳迁入 core 的 host::update，壳只实现 HostEvents
- FFI 方法名与 Tauri 命令一一对应，未来可直接生成桥接层

## 验证
- `cargo test -p jpcg_core`：12 passed（6 金标准 + 6 FFI 协议冒烟：句柄生命周期/JSON 往返/
  错误路径/版本协商/save_config 契约）
- `cargo build --workspace`：0 error
- 三个 dylib 产物确认生成

## 备注
- jcsx_set 死代码警告保留（既定决策：后续补充）
- FFI 冒烟测试写 saved_config.toml 到 CWD 后自行清理

### 来自 changes/2026-08-11-213000.md

# K3: Tauri 双模式（static/dynamic）+ 命令薄壳化 + FFI 桥接

## 什么
1. **Tauri Cargo features 双模式**：
   - `default = ["static"]` — 编译期链接 jpcg_core，Rust 直调 host 模块
   - `dynamic = ["libloading"]` — 不依赖 jpcg_core，dlopen libjpcg_core 经 jpcg_call JSON 调用
   - jpcg_update 改为两模式共同依赖（命令签名/进度事件 DTO 需要，计算逻辑仍在 core）
2. **types.rs 薄壳化**：511 行 DTO+From 实现 → 3 行 `pub use jpcg_api::*`
   （DTO 与转换单源在 jpcg_api / jpcg_core::host::conv）
3. **命令薄壳化**（calculate/optimize/config/data/combo/skill_editor/update）：
   每个命令 = tauri::command 签名不变 + `cfg(feature)` 两个 impl：
   - static：直调 `jpcg_core::host::*`
   - dynamic：`ffi_bridge::call(method, &req)`（DTO 自动 JSON 序列化往返）
4. **commands/ffi_bridge.rs**（dynamic 模式）：
   - 定位 libjpcg_core.{dylib,so,dll}：exe 目录 → target/{debug,release} → `JPCG_CORE_LIB` 环境变量
   - OnceLock 单例加载库 + 创建会话句柄 + Drop 释放句柄
   - `jpcg_set_host_events` 注册 C 回调表（进度 → Tauri emit "update-progress"；
     退出 → app.exit(0)；updater 路径 → JPCG_UPDATER_PATH 环境变量或 null）
5. **HostEventsTable 类型单源化**：定义移到 jpcg_api（repr(C) C ABI 契约），
   core ffi re-export，app 无链接即可构造回调表
6. **行为等价性修复**：`load_config` 的 ConfigDataDTO 改为完全镜像
   CalculateRequest 形状（player/hostile/**xinfa_config**/buff/coefficient），
   保持前端 loadConfig() 契约不变（原实现返回 CalculateRequest，且含 buff/coefficient）
7. **DTO 补 Deserialize**：SkillResultDTO / ComboResultDTO / ComboStepResultDTO /
   jpcg_update 的 UpdateProgressEvent / UpdateCheckResult（dynamic 模式 JSON 回读需要）
8. **update 编排所有权**：Tauri 壳 update.rs 从 160 行 → 双模式薄壳；
   perform_app_update（下载→验证→发动 updater→请求退出）已在 K2 迁入
   jpcg_core::host::update，壳只实现 HostEvents（static）或注册 C 回调（dynamic）

## 为什么
- B 模式（全动态更新）的 Tauri 侧基础：切换 feature 即可静态/动态编译，
  前端代码零改动（命令名、参数、返回形状全部不变）
- core dll 更新换代时 app 二进制无需重装；dev 时 JPCG_CORE_LIB 指向本地 dylib

## 关键决策
- 命令签名保持前端契约优先：tauri::command 参数/返回类型与 K2 之前完全一致
- jpcg_update 不并入 optional：其 DTO 是命令签名的一部分，逻辑轻量；
  真正要"可替换"的计算引擎是 core
- 动态模式句柄懒创建于首次 call（OnceLock 单例，进程级复用）
- 回调表幂等注册（OnceLock guard），避免重复注册

## 验证
- `cargo build -p jpcg_app`（static）与 `--no-default-features --features dynamic` 均 0 error 0 warning
- dynamic 端到端冒烟测试（ffi_bridge_tests::dynamic_call_roundtrip）：
  真实 dlopen libjpcg_core.dylib → jpcg_call("list_professions") JSON 往返 → 错误路径
- `cargo test --workspace` 全绿（13 passed）
- `cargo build --workspace` 仅剩既定 jcsx_set dead_code 警告

## 备注
- 前端 src/api/commands.ts 无需改动（命令契约不变）
- 动态模式实际运行需 libjpcg_core.dylib 与 app 同目录（打包时由 K5 构建矩阵保证）

### 来自 changes/2026-08-11-220000.md

# K5: 模块库（dll）增量更新 — modules_manifest.toml 全链路

## 什么
1. **manifest-gen 扩展**：新增 `--modules-dir <dll目录>`（可选）与 `--modules-output`
   （默认 modules_manifest.toml）与 `--platform`（默认按本机 OS 推断 darwin/linux/windows），
   产出：
   ```toml
   modules_version = "v2.1.0"
   platform = "darwin"
   [[files]] name/hash/hash_type/size
   ```
   与 data_manifest.toml 并行生成
2. **jpcg_update::modules（新模块）**：
   - `ModulesManifest` / `ModulesFileEntry` / `ModulesCheckResult` 类型
   - `fetch_modules_manifest`：stable → `files/JPCG/{app_version}/modules/…`，
     beta → `files/JPCG_beta/modules/…`（与 data 布局对齐）
   - `check_modules_update(beta, force)`：对比 exe 同目录 `modules/` 子目录，
     按 SHA256 差量判定需要更新的 dll
   - `download_and_install_modules`：逐文件下载 → 哈希校验 → 临时名原子替换
3. **UpdateCheckResult 扩展**：新增 `has_modules_update` / `modules_version` /
   `modules_files_to_update`；`check_updates` 在 force 或有 app 更新时一并检查模块
4. **core host::update::perform_modules_update**：下载 → 校验 → 安装 → 进度上报 →
   `request_exit()` 请求宿主重启（重启后 ffi_bridge 优先加载 modules/ 新 dll）
5. **FFI**：`update_modules` 方法（请求携带 beta/version/files，经回调表上报进度）
6. **Tauri**：`perform_modules_update` 命令（static 直调 / dynamic 走桥接）；
   **ffi_bridge 加载顺序调整**：exe 同目录 `modules/` 子目录 → exe 同目录 → target → env
7. **前端**：types 增加 ModulesFileEntry 与 UpdateCheckResult 三字段；
   commands.ts 增加 `performModulesUpdate` + checkUpdate/perform 模拟响应；
   ConfigPanel 更新流程新增模块更新分支（确认 → 进度 → 重启）

## 为什么
- B 模式（增量只换 dll，不重装 app）的服务端与客户端能力打通：
  服务器托管 modules_manifest.toml + dll，app 差量下载到 modules/ 并重启加载

## 关键决策
- 模块版本目录复用 app 版本号（app 更新与模块更新同版本目录，服务器部署简单）
- 本地模块落位 exe 同目录 modules/，与 data/ 平行；ffi_bridge 加载优先级
  modules/ 最高 → 旧 dll 原位替换后重启即生效
- beta 通道模块清单不带版本目录（与 data 的 beta 布局一致）

## 验证
- manifest-gen：真实 dylib 生成 modules_manifest.toml（3 个模块，darwin）
- `cargo test --workspace` 全绿（13 passed）
- `cargo build --workspace` 双模式 0 error；`make check-all` 全部通过
- `npx tsc --noEmit` 前端类型检查通过

## 备注
- 服务器部署步骤（参考 server_manifest.md）：`dist/<version>/modules/` 放 dll +
  modules_manifest.toml；beta 放 `dist/modules/`
- 动态模式 app 的 check_update 会请求服务器 modules 清单；服务器暂无 modules 目录时
  自动跳过（fetch 404 视为无更新）

### 来自 changes/2026-08-11-220100.md

# K4: 双构建矩阵 — Makefile + .cargo/config.toml 别名

## 什么
1. **`.cargo/config.toml` 新增 `[alias]`**：
   - `build-app-static`  → `build -p jpcg_app`（默认，编译期链接 core）
   - `build-app-dynamic` → `build -p jpcg_app --no-default-features --features dynamic`
   - `build-modules`     → `build -p jpcg_core -p jpcg_update -p jpcg_const`（三个 cdylib）
   - `test-core` / `test-app-dynamic`
2. **`Makefile`**：
   - `build-static` / `build-dynamic` / `build-modules`（支持 `BUILD=release`）
   - `modules-dir`：把三个 dll 复制到与 app 相同目录（动态模式运行所需）
   - `test` / `check-all`：双模式 + 全 workspace 测试全绿检查

## 为什么
- 把 K2/K3 的两种 app 构建模式固化为一条命令，消除手写 feature 组合的记忆负担

## 验证
- `make check-all` 全部通过（static + dynamic + modules 构建 + workspace 测试）
- `cargo build-modules` / `cargo build-app-dynamic` / `cargo test-core` 别名生效
- 首次失败点：`[alias]` 表头缺失导致 cargo 不识别别名，已补

### 来自 changes/2026-08-11-223000.md

# K6: ctypes Python demo + JPCG_DATA_DIR 环境变量覆盖

## 什么
1. **examples/python_demo/jpcg_demo.py**：ctypes 直连 libjpcg_core.{dylib,so,dll}：
   - 自动定位库（参数 / target/{debug,release}）
   - 完整协议演示：abi_version → handle_create → list_professions →
     calculate（莫问金标准输入）→ 错误路径 → 句柄释放
2. **store::paths::data_dir() 支持 `JPCG_DATA_DIR` 环境变量**：
   - 指向含 `shuxing/` 子目录的目录（或直接是 shuxing 目录）时优先返回
   - 解决 CLI/Python 等 current_exe 不可用场景的数据定位问题
   - Tauri 正常运行路径不受影响（无环境变量时行为不变）

## 为什么
- 跨语言使用 core 的真实场景验证：Python 直接经 JSON 协议调用获得完整计算结果
- 嵌入式（dll 宿主非 JPCG exe）时数据目录无法从 current_exe 推断，需要显式注入

## 验证
- `JPCG_DATA_DIR=./data python3 examples/python_demo/jpcg_demo.py`：
  列出 2 个门派（莫问/周天功），calculate 返回 28 个技能的完整伤害数据
- `cargo test -p jpcg_core` 连跑 3 次 12 passed（此前一次偶发失败已复测稳定）
- `cargo build --workspace` 0 error

### 来自 changes/2026-08-11-231500.md

# M1: 完整维护流程落地（git-flow + CI/CD + 组件独立版本）

## 什么
1. **版本模型（组件独立）**
   - data_version / jpcg_const 采用 `等级.赛季.日期`（如 `130.3.20260602`），
     映射现有 shuxing 数据 `{level, season, modified}`（modified=YYYYMMDD）
   - `jpcg_const` 独立版本（Cargo version = 130.3.20260602）；`jpcg_updater` 独立；
     其余 crates `version.workspace = true` 继承根 workspace（core 版本 = release tag / 安装包命名源）
   - `VersionInfo::compact()` / `label()`：数据版本紧凑串 + UI 美化（`130级第3赛季 (2026-06-02)`）
   - FFI 版本 getter：`jpcg_core_version` / `jpcg_const_version` / `jpcg_update_version` +
     各 crate `*_VERSION` 常量；Tauri `get_module_versions` 命令（static/dynamic 双模式），
     ConfigPanel 底部展示 App/Core/Update/Const 版本
2. **git-flow 分支初始化**：master(生产) + 新建 develop(集成交互) 并推送
3. **CI/CD（.github/workflows/）**
   - ci.yml：PR/push 触发，rustfmt + clippy + 双模式构建/测试 + 金标准 + 前端 build
   - release.yml：tag v* 触发，三平台矩阵构建（app/dll/updater）+ manifest 生成 + GitHub Release
   - deps.yml + dependabot.yml：依赖审计与每周更新
4. **模板与文档**：PULL_REQUEST_TEMPLATE / ISSUE_TEMPLATE(bug/feature) /
   CONTRIBUTING.md（git-flow + commit 规范 + 版本管理 + 发布流程） / CHANGELOG.md（Keep a Changelog）
5. **脚本与工具**：scripts/release.sh（测试→bump→聚合 CHANGELOG→tag→push）、
   scripts/sync-version.sh（同步 package.json/tauri.conf.json/commands.ts 模拟串）、
   rustfmt.toml、.pre-commit-config.yaml；安装 cargo-edit（cargo set-version）
6. **K5 增强**：modules_manifest 逐 dll 带 `version`+`sha256`，`check_modules_update`
   逐 dll 比较（缺条目/版本不同/哈希不同），本地 `modules/modules_manifest.toml` 快照，
   安装后合并写回快照；manifest-gen 从 dll 文件名推断 crate 版本
7. **clippy 基线**：修复 `jpcg_const_get_u32`（裸指针解引用标记 unsafe + Safety 文档，
   deny-by-default lint）；CI clippy 不 `-D warnings`（既有技术债众多）

## 为什么
- 从无 CI/无分支策略/版本散落 5 处的状态，建立标准化的协作与发布流程
- 组件独立版本契合 B 模式（dll 独立替换）与数据/常量频繁调整的现实
- release tag 跟 core，安装包命名用 core 版本，UI 独立展示

## 关键决策
- 采用 git-flow（master+develop）而非 GitHub Flow：版本化桌面软件 + 定时发布
- 版本用 cargo 原生机制（`version.workspace = true` + `cargo set-version`）而非
  cargo-workspace-version：后者不解析 glob 成员且会覆盖独立版本（const/updater），与组件独立模型冲突
- 模块更新改为逐 dll 版本/哈希判断，本地快照
- CI clippy 暂不 `-D warnings`（既有 ~30 处告警，逐步清理）；release 矩阵用免费额度

## 验证
- `make check-all` 全绿（双模式构建 + workspace 测试）
- 金标准 12 passed；`cargo clippy --workspace --all-targets` 无编译错误
- `cargo fmt --all -- --check` 通过；前端 `npx tsc --noEmit` 通过
- manifest-gen 正确生成逐 dll 版本（core=2.1.0-alpha.1、const=130.3.20260602）
- `cargo metadata` 确认各 crate 版本对齐

## 备注
- master/develop 分支保护（强 review + CI 绿 + squash + 禁直接 push）需在 GitHub Settings 手动开启
- develop 已从 master 创建并推送；core/update 待经 PR 合入 develop

### 来自 changes/2026-08-11-232000.md

# M2: 开源规划文档 — AGENTS.md 更新 + 新增 PLAN.md

## 什么
1. **AGENTS.md 更新**：
   - 删除过时行 "No CI, test suites, or formatter config exist"（与已落地的 CI/测试/rustfmt 矛盾）
   - 新增「开源与命名（当前决策）」段，记录：
     - 保留 `JPCG` 命名（改名"试剑/演武"等已搁置；范围仅产品名，内部 crate/FFI/env 不动）
     - `data/` 暂留仓库；未来联系其他项目负责人获取 lua 数据访问权限后再定发布方式
     - 开源状态：仓库私有（GitHub Free），分支保护需转公有或 Team
     - 场景扩展：core 引擎本就场景无关，PVE 支持主要是 UI 预设 + 数据模板
2. **新增 PLAN.md**：开源与发展路线图（背景 → 当前状态 → M0-M4 里程碑 → 风险合规 → 决策记录）
   - M1 转公有与分支保护、M2 数据授权与合规、M3 开源运营、M4 场景扩展

## 为什么
- 固化"先保留现状、未来再评估改名/数据授权"的决策，供后续协作与开源时遵循
- PLAN.md 作为开源路线的单一事实来源

## 关键决策
- 暂不改名、暂不动 `data/`；数据最终形态由 M2（上游 lua 授权结果）决定
- 开源 = 转公有（免费获得完整分支保护），而非升级 Team

## 备注
- 用户将据本 PLAN 执行转公有；M2 需主动联系上游项目负责人

### 来自 changes/2026-08-11-235000.md

# M3: 开源许可落地 — GPL-3.0 LICENSE + 各 manifest 统一

## 什么
- 新增根 `LICENSE`（GPL-3.0 全文，674 行）
- `jpcg_updater` / `jpcg_app` 补充 `license.workspace = true`（此前缺）
- 前端 `examples/jpcg_app/package.json` 补 `license: "GPL-3.0"`
- 全 8 个 Rust crate 经 `cargo metadata` 确认均解析为 `GPL-3.0`

## 为什么
- 开源前补齐许可；Cargo.toml 已声明 GPL-3.0，落地 LICENSE 文件并统一所有成员

## 关键决策
- 选 GPL-3.0（强 copyleft）：与社区工具(JX3Toy)一致，保证衍生品开源；
  与 PLAN M2 数据授权（上游多宽松/GPL）可兼容，后续拿到具体协议再复核

### 来自 changes/2026-08-12-000000.md

# M4: 三分支线性迁移（dev/beta/release）+ 发布脚本重写

## 什么
1. **分支重构**：从 git-flow（master/develop）迁移到三分支线性模型：
   - `develop` → `dev`（集成/最上游，版本 alpha.n）
   - 新建 `beta`（预发布/公测，版本 beta.n）
   - `master` → `release`（稳定/生产，版本 X.Y.Z）
   - 线性关系：`release ⊂ beta ⊂ dev`，单向超集链
2. **远端操作**：推送 dev/beta/release，设默认分支为 dev，删除旧 develop/master
   （先删保护规则再删分支）
3. **分支保护**：dev/beta/release 三条规则（review≥1 + 3 个 CI 检查 + linear +
   conversation + 禁强推/删除 + enforce_admins:true；beta/release 为 strict）
4. **PR 迁移**：原 #3（LICENSE）因 base 分支删除被自动关闭，重建为 #4（feature/license → dev）
5. **scripts/release.sh 重写**：`--stage alpha|beta|release`
   - alpha（dev）：不 tag；beta：tag vX.Y.Z-beta.n；release：tag vX.Y.Z
   - 校验所在分支与 stage 匹配；测试 → bump → 聚合 CHANGELOG → commit → tag → push
6. **CI/CD 适配**：ci.yml 触发改为 dev/beta/release；release.yml 按 tag 判定
   beta/stable 通道（tag 含 -beta → beta 通道）
7. **文档**：CONTRIBUTING / AGENTS / PLAN 更新为三分支模型

## 为什么
- 用户要求：所有更改融合到 dev → 出 beta.n 公测 → 稳定后 release，线性链式发布

## 关键决策
- release 初始指向旧 master 基线（65e3da1，最后稳定）；beta 同指向旧基线，
  首次提升 dev→beta 才填充
- enforce_admins 保持 true（最严格，需第二 reviewer 或临时降级）
- tag 区分通道：vX.Y.Z-beta.n→beta，vX.Y.Z→stable

## 备注
- PR #4（feature/license → dev）待合并；dev 已含全部 core/update 工作
- 后续出 beta/release 用 `scripts/release.sh beta|release`

### 来自 changes/2026-08-12-100000.md

# UI: 薄荷/松石绿清新主题 + 自绘 SVG 图标

## 什么
1. **配色重构**（`variables.css`）：从深色靛蓝/紫改为薄荷/松石绿
   - 主色 teal `#14b8a6` → sky `#0ea5e9` 渐变
   - **浅色为默认**（`:root`），保留深色主题切换
   - 更新 useTheme 默认主题为 light
2. **自绘 SVG 图标**（新增 `components/icons.tsx`，~18 个内联线条图标，
   stroke=currentColor 继承主题色，不依赖第三方库）：calc/globe/combo/trend/pencil/
   chevron/sun/moon/gear/trash/save/star/close/check/alert/bug
3. **替换全部 emoji**（App 反馈按钮/logo、ActivityBar 页签+折叠、ThemeToggle、
   Toast 状态、ForumPage 翻页、ComboPage 操作、OptimizePage 关闭等）
4. **排版/美术打磨**：卡片阴影、ActivityBar 激活药丸、Header Logo 图标、按钮/输入
   聚焦青色光晕、Toast 状态图标着色、StatusBar 状态点光晕
5. **清理硬编码旧色**：`rgba(99,102,241)`(indigo) / `rgba(168,85,247)`(purple) 统一为 teal/sky
6. **死代码清理**：删除过时 `src/index.html`、`src/css/style.css`（引不存在的 js/app.js、
   css/components.css，React 未使用）

## 为什么
- 用户要求"清新配色 + 排版美术"；采用薄荷/松石绿（A 方案），emoji 改自绘 SVG

## 关键决策
- 图标全部内联 SVG、无新依赖；currentColor 保证跟随浅/深主题
- 浅色为默认（清新观感），深色仍可切换
- 删除死代码降低混淆

## 验证
- `npm run build`（tsc + vite）通过
- `cargo build -p jpcg_app` 通过
- emoji 全量替换后无残留

### 来自 changes/2026-08-12-224904.md

# server_tools/data-import：打通 libpak 拆包（读 skills.tab + 提取 lua + base_damage）

## 什么
1. **修复并打通 PC Pakv4 拆包管线**（此前 `pak.rs` 一直失败）：
   - 正确的 libpak FFI：`init(path)` / `tab_init(path, indexs, fields)` 3 参数 /
     `tab_get(tabname, key, buf, len)` 4 参数（key 带尾随 `\t`，返回字节长度）/ `lua_get`
   - libpak 需 `./cache`（CWD 相对）jx3calc 缓存，缺失时 `init` 返回 -1；
     缓存源在 `~/Library/Application Support/JX3/AssetsDatCache/zsCache/`
   - `tab_init` 的 fields 必须**精确匹配 skills.tab 表头**（用 jx3calc `Skill` 枚举 20 列），
     传错字段名返回 -1；正确字段集下一次 tab_init + 连续 tab_get 数万技能稳定
2. **新增 `src/pakx.rs`**：封装上述正确用法 + `--pak-dump` 模式
   - 遍历 SkillID → 读 skills.tab 20 列 → 提取每个技能 lua 字节码到 `lua/` → 用
     patched luac 反编译解析 `tSkillData[].nDamageBase` → 输出 `skills.tsv` manifest
3. **`main.rs` 新增 `--pak-dump` / `--pak-out` / `--luac` 参数**

## 为什么
- 用户要求"跑通游戏文件拆包"。此前 `pak.rs` 用错 FFI 签名/字段名导致 init 失败、
  tab_init 卡死。本次彻底摸清 libpak 用法并跑通全流程。

## 关键决策 / 发现
- **base_damage 不在 skills.tab**，而在技能 lua 的 `tSkillData[level].nDamageBase`
  （满级值；真实伤害在"实际伤害子技能"如 14474 宫，父技能 14064 仅 L1=100）
- **atk_xishu 不在 skills.tab 或 lua 常量**：为 jx3calc 执行 `GetSkillLevelData` 后派生
  （内功 = frames/192，外功 = frames/160，frames 每技能整数），需 lua 执行引擎，
  当前未在本次落地（下一步 P0）
- `SkillCoefficient` 列仅 NPC/副本技能有值（固定 25600），玩家技能为空

## 验证
- 全量扫描：读取 35303 技能，提取 22115 个 lua，解析 22115 个 base_damage
- 抽查莫问：14474宫=160、14311商=110、14100羽=110，与现有 mowen.toml 一致
- `cargo build -p data-import` 0 error（仅既有/无害 warning）

## 备注
- 运行前需：`mkdir cache && cp ~/Library/Application Support/JX3/AssetsDatCache/zsCache/* cache/`
- atk_xishu（系数）依赖 lua 执行引擎，下一步用 mlua 复刻 jx3calc 的 GetSkillLevelData 派生

### 来自 changes/2026-08-12-230000.md

# data-import + 引擎：全等级技能数据 → 嵌套 levels TOML → 计算器选等级

## 什么
1. **Lua 执行器 `src/luaex.rs`**：用 mlua(Lua 5.1) 执行游戏技能 lua 的 `GetSkillLevelData`。
   - patch mlua vendored Lua 源码（lundump.c 跳过 header + 4字节字符串）加载魔改字节码
   - stub 游戏全局（Include/ATTRIBUTE_TYPE/ATTRIBUTE_EFFECT_MODE/tSkillData/...）
   - 捕获 `SKILL_*_DAMAGE`(base1)、`_RAND`(rand)、`nChannelInterval`(frames)
2. **全等级提取 `--gen-data`**：遍历所有技能×等级，算
   - `base_damage1 = SKILL_*_DAMAGE`、`base_damage2 = base1 + RAND`
   - `atk_xishu = nChannelInterval / 192(内功) 或 /160(外功)`（按 KindType）
   - 输出 TSV（skill_id/name/script_file/kind_type/max_level + 每级 base1/base2/atk）
   - 当前提取 1888 个有伤害数据的技能
3. **TOML 生成 `gen_toml.py`**：按门派目录分组 → 输出**嵌套 levels** TOML（14 门派）
   - `[[skill]]` 内嵌 `[[skill.level]]`（level/base_damage1/base_damage2/atk_xishu）
4. **引擎等级支持**：
   - `Skilltype` 加 `levels: Vec<SkillLevel>`（serde rename="level"）+ `current_level`
   - `SkillLevel` 结构 + `select_level()`/`max_level()` 方法
   - `SkillEditorItemDTO` 加 `levels`/`current_level`，conv.rs 双向映射
   - store 模块公开
5. **前端等级选择器**：SkillEditorPage 伤害系数区加等级下拉，选级应用 base/atk

## 为什么
- 用户要"输出全等级，让用户在计算器内自己选等级"。
- 解出核心公式：base_damage 在 lua tSkillData（执行 GetSkillLevelData），
  atk_xishu = frames/192(内功)/160(外功)，frames=nChannelInterval。
- 251203.json(2025.12) 系数已过时（技改 nerf，如宫 2.609→1.7325），必须从当前 pak 提取。

## 关键决策
- **纯解包，忽略 csv**：一切 base_damage/atk_xishu 从当前 lua 执行得出
- 嵌套 `[[skill.level]]` 格式（一个技能一条，内嵌多级）
- base_damage 需执行 lua（非满级常量）：商= tSkillData[1]*0.7=14
- 等级规则：暂用 skills.tab max_level（宫25/商3）

## 数据源限制
- **段氏/周天功、无相楼（幽罗引）不在本地 pak**：PC Pakv4 段氏有元数据无 lua，
  无相楼连 skills.tab 都没有；无界数据(2025.2)更早无此门派。需新数据源（Pakv5/新版客户端）。

## 验证
- `cargo test -p jpcg_core --test levels_parse`：解析嵌套 levels + select_level 通过
- 实际生成的 mowen.toml 能被引擎加载并选等级
- `cargo build --workspace` 0 error
- 前端 `npx tsc --noEmit` 通过

## 备注
- 生成的 TOML 用的是 skills.tab 原始技能名（如"宫伤害_实际伤害子技能"），
  含全部子技能，非玩家向精简清单——技能名/筛选需后续整理
- gen_data 中 13188 技能无 lua（NPC/被动/路径缺失），核心玩家伤害技能已覆盖

### 来自 changes/2026-08-13-101500.md

# 全门派 shuxing 扩充至 19 门派（老门派数据补全）

## 变更
- `data/shuxing/` 从 14 门派扩充至 **19 门派**，新增：
  - 老四门：纯阳(97)、天策(86)、明教(69)、少林(72)
  - 其他：丐帮(133)、七秀(86)、万花(98)、五毒(86)、唐门(5→112)
  - 扩充：藏剑(1→61)、纯阳(9→97)、天策(8→86)、明教(2→69)
- 全部 TOML `modified` 更新为 20260813

## 起因
- 老门派（纯阳四象轮回等）技能 lua 之前缺失：PakV4 客户端（zhcn_hd）经
  PakV4-Extract（jx3pak 组织工具，Engine_Lua5X64.dll + `KG_InitPakV4FileSystem`
  加载 Trunk.Dir）黑盒探测 1888 个已知技能路径，仅 6 个 NPC/test 命中——
  **决定性确认玩家门派技能 lua 不在 PakV4**。
- Trunk.Dir 为 HTree 哈希树（`[PakV4] Is Use HashName`，nHeight=5），无明文路径；
  StreamDownloader 日志证实客户端数据完整（无增量下载）。

## 数据来源
- 无界缓存 `~/Library/Application Support/JX3/AssetsDatCache/zsCache/` 更新后
  （用户 8/12-13 游戏运行写入新 cache），libpak 全量提取从 1888 → **3448 技能**。
- 老门派技能路径格式为 `纯阳\子技能_四象轮回_非定身.lua`（子技能_/门派名前缀），
  与旧格式不同（这是之前探测失败原因）。
- 长歌/霸刀/苍云/凌雪/药宗/衍天 6 门派数据与旧版完全一致（回归验证通过）。

## 验证
- `cargo test --workspace`：14 通过 0 失败（含 6 金标准）
- `parse_generated_mowen_and_select` 通过（新 TOML 解析 + 等级选择）
- 前端 `tsc --noEmit` 通过

## 遗留
- 纯阳/少林/天策/明教为**全技能**数据（含子技能），尚未按心法奇穴手动核对
- `server_tools/data-import/` 的 gen_toml.py 的 `modified` 日期为硬编码，后续可参数化

### 来自 changes/2026-08-13-114500.md

# 无相楼（幽罗引）加入 —— 20 门派完成

## 变更
- 新增 `data/shuxing/wuxianglou.toml`（无相楼/幽罗引，第 20 门派）
- 25 个伤害技能（skill_id 41116-42417 段）：挑丝(41342, atk=2.5)、绊线(41444, atk=3.75)、
  勾线(2.1979)、四边静(2.0833)、千里急一~四段(0.7291/1.0937/1.4583/2.1875)、生地狱、
  锁南枝、傀梦令、障幕虚影、应天长(7.2916) 等，均为单级（level=1）
- 心法：幽罗引，根骨阴性内功，atk_up=1.96 / pofang_up=2.0

## 无相楼数据特性
- **无 base_damage**（用户确认无相楼技能无 base，只有 atk 系数）
- atk_xishu 取自魔盒 skill API（node.jx3box.com/resource/std/skill/list include=parse）的
  技能描述 `[X*最终阴性内功攻击]`，系数准确

## 背景
- 无相楼技能 lua 不在本地 PakV4/无界缓存（zsCache dat 无，libpak 确认 41342 不在
  skills.tab 35303 行）
- PakV5 FileList 服务下载持续失败（vk_mb/mac_mb/ios_mb/android_mb 全试，CrossOver 与
  wine11 均 bad FileList）；KGPK5 hash 函数 wine 调用崩溃（日志未初始化）
- `~/Library/Application Support/JX3/cache/skill_m.cache`（34MB）含全部 179 个无相楼
  技能脚本路径（GBK 明文），验证了技能名与 ID 对应（挑丝=织心谣_丝线穿刺等）

## 验证
- `cargo test -p jpcg_core`：12+1 全过（含 6 金标准、parse_generated_mowen_and_select）
- modified = 20260813

## 遗留
- 无相楼仅 25 个有伤害系数的技能；纯控制/机制技能（夹线/替形幻生等）未收录（atk=0）
- base_damage=0（游戏设计如此，非缺失）
- 若后续打通 PakV5 数据下载，可补全子技能与等级数据

### 来自 changes/2026-08-13-160000.md

# 撤销 toml 数据与 data-import 解析器提交（等待完整新数据）

## 变更
- `feature/ui-fresh` 分支 `git reset --hard 571c098`，移除 5 个提交：
  - `fa21db8`/`248bd14`/`5ffc9e0`/`375d943`：data-import 生成器/解析器（libpak 提取、luaex、按心法分类）
  - `c564220`：全门派全等级 shuxing 数据 + 引擎/前端等级支持
- 删除工作区残留：`server_tools/data-import/`（探针工具）、`luac.out`
- 保留 4 个探索 changelog（2026-08-12-224904/230000、2026-08-13-101500/114500）作为历史记录
- PR #6（feature/ui-fresh → dev）force push 后仅含 UI 主题提交

## 起因
- 现有 TOML 数据与游戏当前版本数值不一致（宫 atk_xishu 1.7325 vs 实测 2.6095，差 1.506 倍）：
  数据源是 2025.2 旧缓存快照（AssetsDatCache），与正式服版本脱节
- 已获得完整的新数据来源，明天送达；届时平铺单等级数据直接替换 `data/shuxing/*.toml`，
  无需 data-import 工具与等级（levels）功能

## 遗留
- 等级功能（Skilltype.levels/select_level/current_level、DTO、SkillEditorPage 等级选择器）随提交移除，
  新数据不需要；如将来需要可从 git 历史恢复

### 来自 changes/2026-08-13-161500.md

# DOT 持续伤害计算实现（每跳等比递增 + 结果展开）

## 变更
- **数据契约**：`Skilltype.dot_num` 移除，新增 `dot_interval`（每跳间隔秒）/`dot_duration`（持续秒）；
  总跳数 = `dot_duration / dot_interval`（`Skilltype::dot_jump_count()`，非法数据返回 0）
  - 同步：`jpcg_api::SkillEditorItemDTO`、`host/conv.rs` 双向映射、前端 `types/index.ts`、SkillEditorPage（Dot跳数 → 每跳间隔/持续时长）
- **引擎**（`engine/atkcal.rs`）：
  - `dot_jump_expected()`：每跳期望 = 首跳期望 × 等比递增 `(1+dot_up)^(k-1)`（k=0..n-1）
  - `q_cal()`：dot 技能 `q_damage` = 各跳期望之和（总伤害），`DamageResult.dot_jumps` 返回每跳集合
  - `q_cal_with_derivatives()`：导数链整体 × 等比和因子 `((1+u)^n-1)/u`（u=0 退化为 n 倍）
- **输出**：`SkillResultDTO.dot_jumps: Vec<u32>`（计算面板）+ `ComboStepResultDTO.dot_jumps`（连招每步）
- **前端**：ResultTable 与 ComboPage 在期望列下展开每跳伤害（小字标签，hover 显示第 k 跳）
- **金标准**：`golden_gong_dot`（宫 + 6 跳等比 8%，手算期望锁定公式）

## 起因
- dot 字段（dot_flag/dot_up）此前仅定义未接线，dot 技能被当单次伤害计算（低估 6 倍）
- 用户实测：商 DOT 每 3 秒一跳共 18 秒（6 跳），面板每跳倍率 0.2080（正式服数据明天送达后校准）

## 验证
- `cargo test --workspace`：全过（含 7 金标准）
- `cargo clippy -p jpcg_core`：无新增警告
- 前端 `tsc --noEmit` 通过

## 遗留
- 正式服真实 dot_up/dot_interval/dot_duration 数值随明日新数据入库后按金标准校准
- 连招 kill_prob 的方差对 dot 技能按单跳近似（每跳独立会心未逐跳展开）

### 来自 changes/2026-08-13-170000.md

# 真实面板属性金标准 + Python 全量快测 + DOT 数据完善

## 变更
- **数据完善**（`data/shuxing/mowen.toml`）：商（dot）/角（dot）补 `dot_interval`/`dot_duration`
  - 普通版：3 秒一跳、18 秒（6 跳，不递增）
  - 疏曲版：2 秒一跳、18 秒（9 跳），`dot_up` 0.2 → **0.12**（每跳等比递增 12%）
- **金标准替换**（`crates/jpcg_core/src/engine/atkcal.rs`）：
  - 输入改为真实面板属性（2026-08-13 用户提供）：基础属 21371、基础攻击 64329、会心 61877、
    会效 2925、破防 109160；目标外防 15176、内防 21388、御劲 5047、化劲 59402、减伤 0；pvp 0.9
  - 期望值由引擎输出回填（与 python 快测输出完全一致），锁行为待实测校准
  - dot 金标准重写：`golden_shang_dot`（普通 6 跳相等 + q=Σ）与 `golden_shang_dot_shuqu`
    （疏曲 9 跳等比 1.12 + q=Σ）
- **新增 `examples/python_demo/quick_calc_test.py`**：直连 libjpcg_core 全量计算整个 toml
  - 预设真实属性（面板/木桩/pvp 0.9），输出全部技能 Y/B/I/N/H/Q + dot_jumps 集合
  - DOT 断言：普通/疏曲跳数与等比规则、q=Σ、非 dot 为空
  - 面板对照：化劲 76.32%、御劲 2.55%、外防 10.75%、内防 14.51%（引擎系数全部吻合）
  - `--data`/`--lib`/`--out` 参数；退出码 0/1

## 起因
- 旧金标准使用虚构属性（18888/4666），与实际游戏脱节；用户提供真实面板数据作为基准
- DOT 字段此前缺失 interval/duration，无法计算跳数

## 验证
- `cargo test --workspace` 全过（含 8 金标准，新值 = python 快测输出）
- `python3 examples/python_demo/quick_calc_test.py` ALL PASS
- `cargo clippy -p jpcg_core` 无新增警告；前端 `tsc --noEmit` 通过

## 遗留
- 金标准期望值为引擎自锁，待用户木桩实测（宫/商/徵 等）后校准关键值
- 明天完整新数据到货后：替换 `data/shuxing/*.toml`，`quick_calc_test.py --data` 指向新目录重跑全量

### 来自 changes/2026-08-14-105342.md

# json-to-toml 转换器 + mowen.toml 从数据源重生成（50 技能）+ 引擎 f32 化

## 变更
- **新增 `server_tools/json-to-toml`**：从 JPCG 数据源技能 JSON 目录转换生成 `data/shuxing/*.toml`
  - 用法：`cargo run -p json-to-toml -- --xinfa <技能id> --json-dir <JSON根目录>`
  - 数值字段按数据源原值写入（引擎侧已按 `X_MULT` 归一化，不做除法）
  - `dot_interval`/`dot_duration`：数据源毫秒 → 浮点秒（`ms/1000`），支持小数（青莲剑 0.25s）
  - `dot_up`：优先读 `dot_param` 公式中 `j => i*0.12` 的 `dot_up` 系数（疏曲 0.12），回退 `dot_up` 字段
  - **不再输出 `jihuoqixue = "疏曲"`**（用户要求，跳数由引擎按 duration/interval 推导）
  - `design_effect`/`kind_type` 按 JSON 属性自动推断；`cast_mode`/`guaranteed_hit`/`effect_type` 用安全默认
  - `**x` 幂运算符支持（dot_param 公式 `631 * (j ** 0.99592)`）
- **`data/shuxing/mowen.toml` 重生成**：旧 34 技能 → 新 **50 技能**（含 DotOut 目录完整展开：
  宫/商/角/徵/青莲剑·徵 + 疏曲变体、相依/无尽藏/阳春雪/超然/裂涛等奇穴激活输出）
  - 关键值核对（数据源为准）：宫 atk_xishu 1.0325 → 2.609375（引擎 X_MULT 归一化后一致），
    商 2.4479167、疏曲 dot_up=0.12、青莲剑 dot_interval=0.25、max_tick tick→1 修复、`(tick-1)/(max_tick-1)` 全浮点
- **引擎 f32 化**（`crates/jpcg_core`）：
  - `skilltype.rs`：`SkillUp.RateUp` 变体字段 `f32`（`utils.rs` `ReadWidgetOutputData` 列默认值对应改为 `0.0`）
  - `crates/jpcg_api`：`BankNote` 透传 f32（`compound_use` 走 serde `as`）
  - `atkcal.rs` 金标准（`obj_shang_damage1/2` 等）对应更新，全部按确定性指数公式计算
  - 动机：`atk_xishu` 等按数据源原值写入后为小数（2.609375），u32 放不下；后果：面板/buff/属性含义不变
- **`server_tools/manifest-gen`**：`golden_tests` 误引用已删除的 `engine::datamake` → 改为 `engine::atkcal`；去掉多余 `toml` 依赖
- **overrides.json 说明**：`--json-dir` 数据源含心眼技能目录共 3 个（10447 主目录 + 10781/ 10782/ 覆盖目录），overrides.json 当前未启用（可后续补心眼处理规则）

## 起因
- `data/shuxing/*.toml` 此前为手工维护，与数据源脱节；用户提供数据源目录，要求按数据源重生成
- `dot_interval`/`dot_duration` 此前仅支持整数秒，无法表达 0.25s 小数值

## 验证
- `cargo build` 全 workspace 通过；`cargo test` 全过：
  - `jpcg_core` 103 测试（含 2 个新 dot 金标准：普通 6 跳相等、疏曲 9 跳等比）
  - `json-to-toml` 输出 50 条技能，与旧 mowen.toml diff 检出核心值一致
- 转换器重跑幂等性核对（两次生成 diff 为空；疏曲/青莲剑/相依/无尽藏输出均与预期一致）

## 遗留
- 心眼练技能（10781/10782 覆盖目录）转换规则未实现，overrides.json 待补
- dot_up=1 的 dot 条目（阳春雪/modaowan）也能被引擎正确处理，数据源如此即保留
- data/shuxing/mowen.toml 已替换为新生成版本，正式发布前需以新数据跑 `quick_calc_test.py` 全量快测

### 来自 changes/2026-08-14-113755.md

# 修复 dot 基础伤害（base_damage）缺失 + quick_calc 断言泛化

## 变更
- **转换器 bug 修复**（`server_tools/json-to-toml/src/main.rs` `collect_dots`）：
  - 根因：base 提取 `damages[0]` 时漏了 `.get("source_attribute")` 层级，`extract_base`
    永远返回 (0,0)，渲染端 `if base1!=0||base2!=0` 因此不输出 → 7 个 dot 全丢 base
  - 修复：与 `extract_skill` 对齐，`damages[0].source_attribute` → `extract_base`
  - 结果：商/角（含疏曲）=58/58；青莲剑·商/角=26/126；青莲剑·徵=26/26
- **`data/shuxing/mowen.toml`**：重生成，diff 确认仅新增 7 处 dot 的 `base_damage1/2`（其余无变化）
- **`quick_calc_test.py` 断言泛化**：
  - `check_dot` 不再硬编码"第1次=6跳/第2次=9跳"，改为从 toml 按文件顺序读每个 dot 的
    `dot_duration`/`dot_interval`/`dot_up` 推导：跳数 `n=round(duration/interval)`
    （支持浮点秒，青莲剑·徵 2/0.25=8 跳、角 4/1=4 跳、商 1.5/1.5=1 跳），
    `dot_up>0` 等比 ×(1+up)^k 否则每跳相等；q=Σjumps；toml 条目必须全部被 results 覆盖
  - 无质断言：删除硬编码 `WUZHI_NAMES=("相依1","相依2")`，改为从 toml 自动收集
    `has_critical_strike=true` 技能（相依(lv3)/相依(lv4)），并校验其全部出现在结果中
  - 新增 `load_data()`：统一读取 mowen.toml（缺 tomllib/找不到时跳过数据依赖断言）

## 起因
- 用户实测发现 dot 基础伤害（base_atk）丢失：重生成的数据里 `base_damage1/2` 字段缺失，
  旧版手写数据有（商/角 = 58）
- quick_calc 旧断言假设所有 dot 仅 6/9 跳两种形态，与数据源青莲剑系（8/4/1 跳）不符

## 验证
- `cargo test --workspace` 全过（15 测试套件，含 dot 金标准 6/9 跳，金标准不读数据文件不受影响）
- `cargo clippy -p json-to-toml` 零警告
- `python3 examples/python_demo/quick_calc_test.py` → ALL PASS（EXIT=0）：
  - dot 跳数：商/角 6 跳相等、疏曲 9 跳等比、青莲剑 8/4/1 跳，全部按 toml 推导断言通过
  - 无质：相依(lv3)/(lv4) Q 固定=期望公式
  - 面板对照：化劲 76.32% / 御劲 2.55% / 外防 10.75% / 内防 14.51%
- 转换器重跑幂等：两次生成产物 diff 为空

## 遗留
- 金标准 dot 期望值基于旧 base（58），与新数据一致无回归；如后续校准以 `--out` 输出回填

### 来自 changes/2026-08-14-133145.md

# 全职业 toml 导出（27 个伤害心法）+ 全量加载验证

## 变更
- **全职业导出**：`data/shuxing/` 新增 26 个心法文件（10002/10003/10014/10015/10021/10026/
  10062/10081/10144/10175/10224/10225/10242/10243/10268/10389/10390/10464/10533/10585/
  10615/10627/10698/10756/10821/10786），连同 mowen.toml = 27 个伤害心法全覆盖
  - **overrides.json 全量生成**：`xinfa_name`/`xinfa_nom` 从 belongs.json 自动推断
    （`*_to_*_attack_power` 前缀映射：spirit→根骨、spunk→元气、strength→力道、agility→身法、
    vitality→体质），覆盖全部 27 池
  - 文件名沿用历史命名：10447→mowen.toml、10786→zhoutian.toml（保留周天功手写校准值
    atk_up=1.96/pofang_up=0.3）；其余按池 id 命名
  - 治疗心法（10028 离经/10080 云裳/10176 补天/10448 相知/10626 灵素）与通用池 0 在数据源中
    无伤害技能条目，不导出
- **datamake.md**：补充全职业导出命令、overrides.json 字段语义

## 决策
- `atk_up`/`pofang_up`：数据源无此数据（belongs.json 只有属性转化，无攻击/破防倍率）。
  莫问（1.96/2.0）、周天功（1.96/0.3）保留手写校准值，其余职业暂用 1.0 占位，待逐职业校准
- `wuzhi`（无质名单）：无数据支撑，仅莫问（相依）保留手写，其余职业空
- 各职业跳过技能（无伤害数据）少量且为真实无伤害条目（御鸿于天/地坼/幽都判/织翠 等），符合预期

## 验证
- FFI 全量加载：27 个职业逐一 `calculate` 成功，技能数与转换器输出一致
- `cargo test --workspace` 全过（16 金标准 + dot 金标准不受数据文件影响）
- quick_calc_test.py（莫问）ALL PASS

## 遗留
- 其余 25 职业 atk_up/pofang_up/wuzhi 为占位值，需在游戏中实测属性换算后校准
- 纯 base 伤害技能（无 atk_xishu）数量较多（如笑尘诀 33 条），为数据源原始值

### 来自 changes/2026-08-14-144008.md

# 修复连招击杀概率恒为 0（target_hp 单位 ×10000 残留）

## 变更
- **修复根因**：`engine/kill_prob.rs` 中 `target_hp` 仍按旧单位乘以 10000（
  `hostilepile.target_hp as f64 * 10000.0`），而前端已按"个位"输入血量（如 2915274）。
  旧逻辑把血量放大成 291.5 亿，累计期望伤害永远打不死 → 击杀概率恒为 0
- **修复**：`target_hp` 直接使用 `hostilepile.target_hp as f64`，与血量输入单位对齐
- **确定性分支**：当 `cum_std == 0`（会心率 0% 或全无质连招）时，期望伤害 ≥ 血量即
  必杀（kill_prob = 1.0），否则 0.0，避免纯确定性连招误走正态分支
- **无质技能**：`has_critical_strike`（无质）技能伤害固定为期望 Q，方差为 0
- **DTO 透传**：`ComboStepResult` 输出 `has_critical_strike`/`zhenshishanghai` 字段，
  conv.rs 映射到 `ComboStepResultDTO`

## 验证
- FFI 复现用户场景（20×风来吴山、血量 2915274、会心 48.6%）：击杀概率从第 5 步开始
  爬升（0.02%），第 9 步 89.84%，第 13 步起 100%，final = 99.99999999999998%
  （修复前全部恒 0）
- `cargo test --workspace` 全过（16 金标准 + dot 金标准不受影响）

## 遗留
- 用户侧需更新 core 二进制：dynamic 模式替换 `modules/` 或 exe 同目录 dll；
  static 模式需重新编译 App（`make build-app-dynamic`）

### 来自 changes/2026-08-14-150000.md

# 技能形态全量输出（放开连续 lv 压缩）+ 技能池搜索/折叠

## 变更
### 数据层（server_tools/json-to-toml）
- **放开 `is_series()` 压缩**：所有技能的全部 lv 形态（等级/层数/点数/距离/目标数等）
  全部输出，不再只取最大 lv。全库 990 → 1627 条目
  - 段氏（zhoutian.toml）：引窍 101 形态（0~100 点任脉）、破 51 形态（0~50 点能量）、
    绝脉 15 形态（1~15 层）——段氏靠回能技能回能、引窍打出伤害，需逐形态可选
  - 其他门派同类：布泽/叠刃/悬象/流血/入月/鬼宿/破绽/缠绞/勾线(40级)/展缓(99级) 等
- **显示名语义化**：同名多形态优先用数据源 comment 生成 `名·形态` 后缀
  （如 `引窍·50点任脉`、`相依·1段`），无 comment 退回 `名(lv{N})`
- `is_series()` 函数删除

### 后端（jpcg_api / jpcg_core）
- `SkillPoolItemDTO` 新增 `sub_id` 字段（同 skill_id 不同形态区分），
  `host/skill.rs` load_skill_pool 与 `host/calc.rs` skill_dto_to_skilltype 透传
- 预设 `ComboStep` 新增 `sub_id`（serde default，旧存档兼容），
  conv.rs 预设往返保留 sub_id

### 前端（examples/jpcg_app）
- `ComboPage` 技能池：
  - **搜索框**：按技能名实时过滤（如搜"0点任脉"定位形态）
  - **折叠分组**：按基础技能名（去掉 `·形态` 后缀）分组，组头显示形态数，
    可展开/折叠，支持全部展开/折叠
  - **收藏/React key 唯一化**：改用 `skill_id-sub_id` 复合 key，
    修复同 id 多形态（相依 lv3/lv4）收藏与渲染混叠
  - 旧收藏 localStorage 数据兼容转换
- types/index.ts 补 `sub_id`；commands.ts mock 补形态示例（引窍 0/50/100 点任脉等）

## 验证
- 27 池重生成：zhoutian 196 条（引窍 101/绝脉 15/破 51 形态命名正确）、
  10821 勾线 100 条、10175 展缓 99 条、dot 技能保留
- FFI：load_skill_pool 返回 sub_id + 各形态 atk_xishu 正确
- FFI 连招回归：引窍·0点任脉 q=46.5万 vs 引窍·100点任脉 q=232.7万，
  final_kill_prob 计算正常
- `cargo test --workspace` 全过（16 金标准 + 各 crate）
- 前端 `npx tsc --noEmit` + `npm run build` 通过

## 遗留
- 骤风令：已解决（数据源更新推入，见 changes/2026-08-14-163023.md）
- `雾刃飞光·雾/刃/飞/光`、`劈风令·骤` 等名称本身带 `·` 的技能会被归入基础名组
  （如"雾刃飞光"组），语义上属于关联技能折叠，可接受

### 来自 changes/2026-08-14-163023.md

# 2026-08-14 163023 — 数据源更新（骤风令）+ 无质批量标注 + 追加真伤（已损失生命值）

## 背景

IcyTide/Generator 推送更新：段氏「骤风令」本体进入 skills.json（原缺失，仅 37779/37804
名为「横驱风靡」的旧条目），并**成批给无质技能打上 `critical_strike/critical_power = "0"`
占位标记**（此前仅剑飞惊天 1 个）。数据源结构同步变化：critical 从 node 层移入
damages 数组元素内（node 层不再有 critical 对象）。

## 变更

### 数据源脚本与文档
- 新增 `server_tools/json-to-toml/fetch-json.sh`：curl 拉取 IcyTide/Generator
  `assets/json/{skills,dots,belongs}.json` → `data/raw-src/`（已加入 .gitignore，原始
  JSON 不入库），末尾打印转换命令提示
- `data/shuxing/datamake.md`：新增「数据源获取」小节；转换器规则更新（新结构/
  无质 0/O 判定/追加真伤/rename）

### 转换器（json-to-toml）
- **适配新结构**：critical_strike/critical_power 从 damages[0] 提取（兼容旧 node.critical）
- **无质 0/O 规则**：cs/cp 任一为 `"0"/"O"` 占位 → `has_critical_strike = true`
  （数据源官方标注，约 19 技能/65 形态：果报/众境/剑飞惊天/临源/令怖/圣裁/崇光斩恶/
  涤罪/纵遇善缘/断马摧城/相依/不愧君/斩长鲸/镇星入舆/领胡 等）；
  overrides `wuzhi` 名单保留兜底（涤罪/临源 补入名单）
- **追加真伤形态展开**：overrides 新增 `lost_hp_zhenshi: [{name, per_layer, max_layers}]`，
  命中技能展开为 破绽0..=N 层形态并写 `lost_hp_zhenshishanghai` 字段
  （怒锋倾涛：单持/双持 × 破绽0~3层 = 8 形态，每层 6% 已损失生命值）
- **rename 能力**：overrides 新增 `rename` 映射，数据源名 → 显示名
  （10786「骤风」→「骤风令」，对齐 belongs.json 名）
- **sid 级 name 计数**：同名跨 sub（单持/双持等多 sub 同 sid）也能正确加 comment 后缀

### 引擎追加真伤（怒锋倾涛破绽机制）
- `Skilltype.lost_hp_zhenshishanghai: f32`（追加真伤 = 目标已损失生命值 × 系数，无视防御）
- `kill_prob.rs` 动态结算：`追加 = max(target_hp - 前序累计期望 - 本步主Q, 0) × 系数`
  ——**确定性伤害只加期望不加方差**（与无质一致），连招中随血量损耗几何收敛
  （追加本身也计入已损失），击杀 CDF 框架不变
- `ComboStepResult.lost_hp_zhenshi_damage: f64` 暴露每步追加值
- 单技能面板（q_cal）不含追加（满血目标追加为 0，正确语义）
- DTO 全链路透传：jpcg_api（SkillPoolItemDTO/SkillResultDTO/ComboStepResultDTO/
  SkillEditorItemDTO）→ host conv/skill/calc → 前端 types/index.ts +
  SkillEditorPage 新增「追加真伤(已损失×系数)」输入框

### 测试
- 新增金标准 `combo_lost_hp_zhenshi_dynamic`：首击追加 = 已损失(主Q)×系数、
  追加几何收敛、确定性方差 0、8 连击击穿目标血且不超杀
- `cargo test --workspace` 全过（17+1）；前端 tsc + build 通过

## 数值验证（FFI，怒锋倾涛·单持·破绽3层，目标 200 万血）

```
Q=152535 追加=332544  累计=485079   击杀率 0.0%
Q=152535 追加=245230  累计=882843   击杀率 0.0%
Q=152535 追加=173632  累计=1209010  击杀率 0.0%
Q=152535 追加=114922  累计=1476467  击杀率 0.0%
Q=152535 追加=66780   累计=1695782  击杀率 0.1%
Q=152535 追加=27303   累计=1875620  击杀率 12.2%
Q=152535 追加=0       累计=2028155  击杀率 59.7%
Q=152535 追加=0       累计=2180690  击杀率 92.9%
```

## 决策记录
- 追加结算采用「主伤害扣血后的一次性已损失」（不迭代）：`(target_hp - 前序 - 主Q) × 系数`
- 无质判定：数据源 0/O 占位为主，overrides 名单兜底（不相依依赖代码层 0/O 猜测）
- 装备特效池（池 0）的 custom_damage_base 真伤 23 条不并入职业池（维持现状）

## 遗留
- 其余职业 atk_up/pofang_up 仍为 1.0 占位（需逐职业实测校准）
- 池 0 装备特效（巽/御·击破/刃凌等真伤）未纳入职业技能池

### 来自 changes/2026-08-14-205054.md

# 2026-08-14 205054 — 恢复被误删的周天功/莫问数据文件

## 背景

数据迁移（编号 TOML 化）后 `data/shuxing/zhoutian.toml` 与 `data/shuxing/mowen.toml`
从工作区被删除（git 状态 D），但这两个池无编号文件替代，导致周天功/莫问数据缺失。

## 原因

- 周天功 = 数据源池 **10786**，莫问 = 池 **10447**，但 `overrides.json` 为二者显式配置
  `file: "zhoutian.toml"` / `"mowen.toml"`（沿用历史名，datamake.md 已有说明），
  转换器 `file_name = ov.file 或默认 <池id>.toml`，因此它们不会生成编号文件。
- 最后一次转换运行未覆盖这两个池（或生成后被清理）。

## 变更

- 重跑转换器恢复两个文件：
  ```sh
  cargo run -p json-to-toml -- --skills data/raw-src/skills.json \
      --dots data/raw-src/dots.json \
      --overrides server_tools/json-to-toml/overrides.json \
      --out data/shuxing --xinfa 10786 --xinfa 10447
  ```
  - `zhoutian.toml`：周天功/元气（atk_up 1.96 / pofang_up 0.3），196 条技能
    （含引窍·0~100点任脉 全形态、骤风→骤风令 rename）
  - `mowen.toml`：莫问/根骨（atk_up 1.96 / pofang_up 2.0），61 条技能
    （相依·1段/2段 等标无质）

## 验证

- `tomllib` 解析 OK；`cargo test -p jpcg_core` 20 项全过
- FFI `list_professions` 27 个心法含 `zhoutian`(周天功) + `mowen`(莫问)
- Python demo（JPCG_DATA_DIR=./data）端到端计算通过

## 留待处理

- 其余缺失池（10028 离经易道 / 10080 云裳心经 / 10176 补天诀 / 10448 相知 /
  10626 灵素）为治疗心法，按 datamake.md 约定跳过，无需生成
- 池 0（婆罗门/装备特效）按既有决策不并入职业技能池

### 来自 changes/2026-08-15-143820.md

# 修复武器伤害双算：AtkConfig::total() 只读基础攻击（分山劲断马摧城实测校准）

## 现象
分山劲断马摧城（无质）实测 536939，计算结果 61 万，偏高约 13.6%。

## 根因
`b_cal()` 使用 `AtkConfig::total()` = 基础攻击 **+ 武器伤害**，
`i_cal()` 又叠加 `武器×watk_xishu/100` 路径：

```
x = base_atk + (基础攻击+武器)×atk_xishu + 武器×watk_xishu/100
```

武器伤害被乘上技能攻击系数重复计入（断马：12062×16.78125 ≈ 20.2 万 I 段 → Q 多算约 7 万）。
数据源公式武器伤害仅经 `watk_cof` 参与（watk_xishu=0 的技能完全不参与）。

复现验证：
- 修复前：断马摧城 Q = 609776（与用户报告 61 万一致）
- 修复后：Q = 540119（去掉武器双算后）；实测 536939 处于 base 120~126 随机浮动区间内（±2.4%），判定为同一次样本的正常浮动，不再校准

## 改动
- `crates/jpcg_core/src/type_set/player.rs`：`AtkConfig::total()` 只返回 `base`（不含武器伤害），注释说明武器单独经 watk_xishu 参与
- `server_tools/json-to-toml/overrides.json`：池 10390（分山劲）`atk_up` 1.0 → 1.88
- `data/shuxing/10390.toml`：[xinfa] `atk_up = 1.88`（与 overrides 一致）

## 验证
- 引擎（用户面板：基础属性 18790/基础攻击 58197/会心 53354/会效 0/破防 134470/武器 12062；目标外防 15071/御劲 5047/化劲 62113）：
  N=458271, H=793247, Q=540119
- `cargo test --workspace` 全部通过（core 20 项 + app 1 项），无回归
- `total()` 唯一调用点即 `b_cal()`；derivatives 武器导数本就只走 watk_xishu 路径，无需联动

## 遗留
- 其余 24 个池的 `atk_up` 目前均为 1.0 占位（仅 10390=1.88/10447=1.96/10786=1.96 经确认），待逐一实物核对后更新 overrides 并重跑转换器

### 来自 changes/2026-08-15-145232.md

# 内防修复后回填 atkcal 金标准期望值

## What

`engine/atkcal.rs` 的用例在上一轮修复（内防门派改用 `guo_nfangyu_with`）后，金标准（golden）测试仍持有旧期望数值。本轮:

1. 统一 golden 用例的 skill 为 `&Skilltype`。
2. 用临时 `golden_dump_tmp` 测试 dump 出新数值，回填 6 组 golden 用例 + `combo_wuzhi` 用例的 `y/b/i/n/h/q` 与全部 6 项导数；dump 测试跑完即删。
3. `cargo fmt -p jpcg_core` 格式化受影响代码块。

## Why

Q 段数值依赖内防/外防分支，修复后各用例期望值整体下移（如 gong_default q=95725→91768、y=1355→1299）。不更新则 golden 全红且无法发现后续回归。

## Validation

- `cargo test --workspace` 全绿（atkcal 12 个 golden/combo 用例通过）。
- `cargo fmt -p jpcg_core --check` 通过。

## Notes

- 数值回填采用「dump→回填」而非手工推算，与「金标准不改行为」原则一致。
- 导数与 buff 有无无关（与修复前一致，非回归）。

### 来自 changes/2026-08-15-150000.md

# 版本号 bump：2.1.0-alpha.1 → 2.1.0-alpha.2

## What

- 根 workspace 版本（core 版本源）升至 `2.1.0-alpha.2`。
- `scripts/sync-version.sh` 同步 package.json / tauri.conf.json / 前端模拟版本串。

## Why

随 PR #6 累积修复（武器双算 + 内防分支 + 金标准回填）后发版前的最小版本步进。

### 来自 changes/2026-08-15-212429.md

# 创建 jpcg_combo 连招引擎 crate（排轴器后端重建，MC 击杀率 + hp 追加真伤）

日期：2026-08-15

## 背景
排轴器（ComboPage）伤害偏小 + dot 技能只算单跳 + 击杀率正态近似不可靠。此前讨论决定：
新建 `jpcg_combo` crate 承接连招编排，单技能公式/追加真伤公式留在 core（单源），
击杀率改用蒙特卡洛（50k 采样），hp 追加真伤按路径实际血量结算。

## 变更
### 新 crate `crates/jpcg_combo/`（cdylib + rlib）
- `engine.rs`：双通道模型
  - 期望通道：g/h/q/dot 期望 + 累计期望/方差（DTO 语义与迁移前一致）；追加真伤只加期望
  - 蒙特卡洛通道：`samples=50_000`（`DEFAULT_SAMPLES`，`ComboConfig` 可注入种子/采样数），
    每条路径实时血量推进，dot **逐跳独立**会心判定，击杀率 = 击杀路径占比，确定性无质连招精确
- `host.rs`：DTO 层计算入口 + 预设 CRUD + 导出/导入（存储委托 `jpcg_core::combo_io`/`config_io`）
- `conv.rs`：`skill_dto_to_skilltype` 等转换；ComboStep/ComboPreset 的 From 因孤儿规则留在 core
- `ffi.rs`：句柄 + JSON 协议（`jpcg_combo_call` 等，ABI_VERSION=1，与 core 一致）
- 测试：无质零方差、固定种子可复现、MC 击杀行为、真伤递增、dot 逐跳独立方差

### jpcg_core 收敛（删除）
- `engine/kill_prob.rs`、`host/combo.rs`、`engine/mod.rs`/`host/mod.rs` 的 mod 声明
- `ffi.rs` 7 个 combo 方法分支、`lib.rs` `calculate::start_combo` facade
- `atkcal.rs` golden 的两个 combo 测试迁至 jpcg_combo（新语义重写）

### 追加真伤语义（修复旧 bug）
- 旧：`lost=(target-(cum+mean)).max(0)` 把剩余血量当已损失 → 击杀率≠0 时真伤错误（用户报告的根源）
- 新（语义 A，core `lost_hp_append` 实现）：`lost = (max_hp - 结算后剩余).max(0) × 系数`，
  真伤同样扣血 → 期望通道递增（斩杀机制），MC 逐路径实时结算

### 修复会心率双计 bug
- 旧 kill_prob `crit_rate = guo_huixin() + huixin_up/100 + buff.huixin_pct/100`（重复计入 buff）
- 新：与 q_cal 一致 `guo_huixin() + huixin_up/100`

### DTO dot 字段链路（dot 只算单跳的根因）
- `SkillPoolItemDTO` 补 `dot_interval/dot_duration/dot_up/wushijianshang/zhenshishanghai`
- `load_skill_pool`、`skilltype_to_pool_item`、combo `skill_dto_to_skilltype` 全链路映射

### Tauri 双模式
- static 直调 `jpcg_combo::host`；dynamic 经 `ffi_bridge` 新增 `COMBO` OnceLock 加载
  `libjpcg_combo`（`call_combo*` 系列）；`get_module_versions` 增加 `combo` 字段

### 前端
- hostile 表单加「目标最大血量/当前血量」（0 = 未提供，回退 target_hp 满血模型）
- `HostileConfigDTO`/`ModuleVersions` 类型、`HOSTILE_FIELDS`、normalize 同步

### Makefile
- `build-modules`/`modules-dir` 纳入 jpcg_combo（dll 不进 modules_manifest 更新体系）

## 决策记录
- combo 依赖 jpcg_core（无环依赖）；损失公式唯一实现在 core（combo 经 `lost_hp_append` 调用）
- 无 hp 输入时（target_hp 也为 0）击杀率恒 1（迁移前行为）
- dot 的技能级 g/h 按 `dot_jumps` 等比因子拆分为逐跳 g/h（与期望通道同源），逐跳独立采样
- jpcg_combo 版本继承 workspace（2.1.0-alpha.2），不进 modules_manifest

## 验证
- `cargo test --workspace` 全绿；jpcg_combo 11 测试；dynamic e2e 冒烟绿
- `cargo build`（static+dynamic 双模式）、`cargo fmt --all --check`、前端 `npm run build` 绿

### 来自 changes/2026-08-18-140250.md

# 修复 macOS 技能编辑器底部裁剪

## 变更

- 让技能编辑页作为应用主区域的 flex 子项占用顶部工具栏之外的剩余高度。
- 允许右侧技能详情面板在受限高度内收缩，并由自身滚动显示完整编辑表单。

## 原因

技能编辑页原先使用 `height: 100%`，但它位于带顶部工具栏的纵向 flex 容器中，实际高度会叠加在工具栏之后。主容器的 `overflow: hidden` 会裁掉详情面板底部，导致 macOS 窗口中最后的字段无法滚动到可见区域。

## 决策

使用 `flex: 1` 与 `min-height: 0` 修正容器约束，保留现有详情面板内部滚动和页面布局，不改变字段、数据协议或保存逻辑。

### 来自 changes/2026-08-30-173635.md

# 修复全 workspace clippy 警告 + README 编译步骤

日期：2026-08-30

## 背景
dev 合并前需通过 CI（clippy 虽不 -D warnings，但要求清零）。同时更新 README 增加编译/构建步骤。

## 变更
### README
- 版本头更新 v2.1.0-alpha.2（2026年8月）
- 击杀概率曲线：CLT 正态 → 蒙特卡洛 50k 路径（示意代码）
- 目标设定输入项补「目标最大/当前血量」
- 主题描述修正为薄荷/松石绿；Roadmap 标记 AutoDiff 已实现
- 新增「🔧 编译与运行」章节（快速开始 / Tauri / 双构建模式 / 打包 / check-all / workspace 结构）

### clippy 清零（19 文件，~50 警告）
- `needless_borrow`×11：atkcal `&self.coeff` → `self.coeff`
- `collapsible_if`×多处：合并嵌套 if / `&& let`
- `missing_safety_doc`×9：core/update/const 的 FFI 函数补 `# Safety`
- 删除死代码 `default()`×3：player/hostilepile/xinfa（无调用方，与 Default trait 冲突）
- `manual_is_multiple_of`：`y.is_multiple_of(...)`
- `excessive_precision`×2：golden 字面量截断 `1.776_041_6`（f32 等值，行为不变）
- `ptr_arg`：`is_string_empty(&String)` 加 `#[allow]`（serde skip_serializing_if 需该签名）
- `map_or`→`is_some_and`、`for_kv_map`→`keys()`、`let_unit_value`、`match_single_binding`、`bind_instead_of_map`、`match_result_ok`
- `too_many_arguments`（download.rs 公开 API）、`type_complexity`（derivatives）→ 加 `#[allow]`/type 别名

## 决策
- `default()` 死代码直接删除（经全 workspace 搜索确认无调用方），非加 allow
- `is_string_empty` 因 serde 属性签名需 `&String`，用 `#[allow(clippy::ptr_arg)]` 保留

## 验证
- `cargo clippy --workspace --all-targets` 0 警告
- `cargo test --workspace` 全绿（金标准 15 + combo 11 不因字面量截断变值）
- `cargo fmt --all --check`、`make check-all`、前端 `npm run build` 全绿

### 来自 changes/2026-08-30-230349.md

# 等级常数编译期 const 模块（jpcg_const::level_constant）取代 CoefficientConfig 默认并补御劲

日期：2026-09-04（收尾）

## 背景与决策
- 目标模块定位：**常数计算模块，编译时由预设 TOML 解析并固化为 `const` 全局**，
  为后续"等级 → 常数"关系逆向（靠 level 推出所有常数）铺路。
- 弃用先前 WIP 的"运行时经 `data_dir()` 读 `normal_num` TOML"方案（引入 core 反向依赖 + 运行时 I/O）。
- 改为：`include_str!` 内嵌预设 + `const fn` 白名单解析，运行时零依赖、零 I/O；
  `level` 只作快照记录（`LEVEL`，预计算用），不进常量结构。
- 引擎 `CoefficientConfig` 仅保留"可配置载体"角色：字段集合补上缺失的御劲分母
  （原 `guo_yujin_huixiao_with` 错误复用 `huixin_xishu`，御劲会伤减免分母 55123.2 从未进入配置），
  默认值单一来源 = 本 const 模块；固定公式方法统一委托 `*_with(CoefficientConfig::default())`，
  引擎各处硬编码分母（126007.2/30115.8/55123.2/197703/72844.2/225957.6）全部移除。

## 变更
### jpcg_const（叶子 crate，运行时依赖清零）
- `preset/level_constant.toml`（新）：130 级·第 3 赛季快照
  （pofang/huixin/huixiao/yujin/yuhui/huajin/fangyu 分母 + pvp_global_jianshang）
- `src/level_constant.rs`（新）：`LevelConstant` 结构 + `LEVEL`/`CURRENT` const；
  `const fn parse_snapshot` 逐行白名单解析——未知 key/缺字段/坏值 → 编译期报错（防漂移）；
  f32 十进制用"整数尾数÷10^k"解析，与 Rust 字面量 bit 完全一致（有单测锁定）；
  const fn 受限说明：不支持切片，全程以 (bytes,start,end) 偏移扫描。
  测试 4 个：bit 一致 / 全量 roundtrip / 未知 key panic / 缺字段 panic。
- `lib.rs`：模块改挂 `level_constant`；删除 `player.rs`（运行时读取器，整文件删除）
- `Cargo.toml`：移除 jpcg_core/serde/toml/tempfile 全部依赖（含 dev-deps）

### jpcg_api
- `CoefficientConfigDTO` 新增 `yujin_xishu` / `yuhui_xishu`（`#[serde(default)]`，旧存档兼容）

### jpcg_core
- `Cargo.toml` 新增 `jpcg_const` 依赖；`lib.rs` 还原 `pub mod store` → `mod store`
- `type_set/coefficient.rs`：字段 +御劲两项；`Default` 取自 `jpcg_const::level_constant::CURRENT`；
  新增 `From<&CoefficientConfigDTO>`（分母 0/缺失 → 真源默认；pvp 减伤 0 语义合法不回退）
- `type_set/player.rs` / `hostilepile.rs`：固定公式改为委托 `*_with(&CoefficientConfig::default())`；
  `guo_yujin_huixiao_with` 改用 `yuhui_xishu`（55123.2，修复错接）、`guo_yujin_huixin_with` 改用 `yujin_xishu`
- `host/calc.rs`、`host/config.rs`：DTO→core 走 `CoefficientConfig::from`（旧档/缺字段安全回退）

### jpcg_combo
- `host.rs` conv 改用 `CoefficientConfig::from`；测试系数字面量补齐新字段
- `engine.rs` 无质期望回填：御劲会伤减免修正后 91768→90651、183536→181302（hostile yujin=5047）

### core 金标准
- `atkcal.rs` golden 7 组（gong/wei/zheng × default/buff + gong_wuzhi）H/Q/导数按新引擎回填
  （变化方向 = 御劲会伤减免增强，仅 yujin>0 目标生效）

### 前端（examples/jpcg_app）
- `types/index.ts`、`utils/constants.ts`（COEFFICIENT_FIELDS + DEFAULT_COEFFICIENT）、
  `utils/normalize.ts` 同步新增字段；其余经 `...DEFAULT_COEFFICIENT` 展开自动携带

## 验证
- `cargo fmt --all -- --check` 干净
- `cargo clippy -p jpcg_const -p jpcg_api -p jpcg_core -p jpcg_combo --all-targets` 零警告
- `cargo test -p jpcg_const`（4）/ `-p jpcg_core` / `-p jpcg_combo` 全部通过
- 前端 `tsc --noEmit` 通过
- python demo / FFI JSON 无需改动：缺字段零值由 `From` 回退真源默认

## 注意/待办
- 逆向进展后可扩展：预设增多级快照 → `[LevelConstant; N]` 表；关系已知后替换为
  `const fn from_level(level) -> LevelConstant` 推导（预设退化为系数）。
- `yujin_xishu` 当前与 `huixin_xishu` 同值(197703)，是否同一游戏常量待逆向确认；
  已拆独立字段避免届时改 schema。
- core 现编译期依赖 jpcg_const：动态模块更新语义下 core dll 会内嵌 const 数据副本，
  若未来要求"只更 const dll 即更新引擎默认系数"需另做运行时取数设计（当前阶段无此需求）。

## [2.1.0-beta.2] - 2026-09-08

### 来自 changes/2026-08-09-100000.md

# core/update P1 — 热路径去 clone 与求导去重

日期: 2026-08-09

## 做了什么

1. **`JpcgConfig` 引用化** (`cal/atkcal.rs`)
   - 结构体由持有 6 个 owned 配置改为持有 6 个引用（`&PlayerConfig` 等），增加生命周期参数。
   - 删除无调用方的 `new()`（此前已无人使用），`new_with_config` 全参数改引用。
   - 消除 `call_back`/`compute_derivatives`/`calculate_combo` 三个循环内对
     player/hostilepile/skill/xinfa/buff/coeff 的逐技能全量 clone。

2. **求导去掉二次全链** (`cal/atkcal.rs` `q_cal_with_derivatives`)
   - 原实现先调 y_cal/i_cal/g_cal/h_cal，再调 `q_cal()` 把整条链再算一遍（×2 全链）。
   - 现复用 forward 阶段中间结果，Q 段就地计算，数值公式与 `q_cal()` 完全等价：
     `q = G×(1-crit) + H×crit`，结果数组 `[y, i1, i2, G3, H4]`。

3. **顺带清理** (`type_set/player.rs`)
   - 删除未使用的 `use crate::log::error;` 导入。

## 为什么

- 每个技能重复 clone 6 个结构化配置并重跑全链，是纯冗余计算。引用化 + 复用后，大量 `derive(Clone)` 调用消失，求导路径计算量约减半。

## 约束与保障

- **不改变任何数值行为**：Q 段与结果数组构造逐条对照原 `q_cal()` 等价。
- 数值一致性将由后续 P4 基准单测锁定。
- 未触碰截断/取整逻辑（P2 另行显式化，仍是逐位一致）。

## 其他

- `jpcg_app`（Tauri）先前 cargo check 失败，原因是缺少 `src-tauri/binaries/jpcg_updater-aarch64-apple-darwin`（见 server_manifest.md 预置步骤），已按文档补上，workspace 全量 check 通过。
- 剩余 4 个 dead-code 警告为既有（`Config::load`、`data_load`、`DamageResultWithDerivatives.result` 等），本次未处理。

### 来自 changes/2026-08-09-103000.md

# core/update P2 — 截断逻辑显式化（数值逐位一致）

日期: 2026-08-09

## 做了什么

`cal/atkcal.rs` `g_cal()`:

原先一整行链式 `as u32` cast（嵌套 6 层括号），把游戏实测的 3 个截断点隐在其中。
现拆分为显式 `truncate()` 辅助函数 + 三步注释：

1. ① `I2 × (1+命中增益) × 伤害增益 × (Y/1024)` → 截断
2. ② `× (1 − 化劲/1024)` → 截断
3. ③ `× 全局PVP减伤 × (1 − 目标减伤比)` → 截断输出

新增 `truncate(v: f32) -> f32`：`v as u32 as f32`，与游戏实测截断语义一致。

## 为什么

把「游戏实际检测得出的截断顺序」从嵌套 cast 迷宫中显式化，配上中文注释固定语义，避免后续重构误改或误优化。**这是纯形式化重构。**

## 逐位一致保障

- 运算顺序与优先级完全复刻原表达式（乘左结合，`pvp` 在最后一步内部乘），无新增/移除任何运算。
- 每步取整点与原 `as u32` 出现位置一一对应。
- P4 将以改动前行为为基准做等价测试（快照），双重保险。

## 决策

- 未统一 buff 百分比换算的 `*1024/100`（1024 制系数）与 `/100`（0~1 比率）两种写法 —— 二者语义分别对应系数域与比率域，混写是有意区分，不存在不一致，故不动。
- `y_cal`/`h_cal`/`i_cal` 各自仅有一个 `as u32` 单点截断，本就显式，未触碰。

### 来自 changes/2026-08-09-110000.md

# core/update P3 — toml_input "none" 哨兵改 Option，清理 unwrap_or

日期: 2026-08-09

## 做了什么

1. **`io.rs` `toml_input`** 返回类型 `String` → `Option<String>`：
   - 不再用 `"none"` 字符串充当「文件不存在」哨兵；
   - 文件缺失/读取失败返回 `None`，成功返回 `Some(内容)`。

2. **调用点适配**（公共行为不变）：
   - `cal.rs` 步骤3：`"none"` 判断 → `None => TomlConfig::default()`；
   - `io.rs::load_config`：`None` 直接默认空配置（去掉原先对 `"none"` 再做 from_str 失败的误导日志）；
   - `io.rs::load_save_config`：`None` 时走既有的「未找到配置，使用默认值」warn 路径；
   - `type_set/jcsx_set.rs::data_load`（死代码，预留模块）：`unwrap_or_default()` 适配新签名。

3. **`io.rs::load_config`** `to_str().unwrap_or("").to_string()` + 空串再判断
   → 显式 `match` 处理非法 UTF-8（符合 AGENTS.md 库代码约束）。

## 为什么

- `"none"` 哨兵会误导后续维护：一旦调用方忘记判 `"none"`，会直接 `toml::from_str("none")` 拿到伪装错误；Option 类型强制穷举，编译器兜底。
- `unwrap_or("")` 把非法 UTF-8 路径折叠成空字符串再二次判断，绕弯且易漏。

## 影响面

- 仅为 `jpcg_core` 内部模块使用（`io` 是私有 mod，外部 crate 无直接依赖此符号）；Tauri 侧公共入口（load_config/save_config 等）签名均未变。
- feature 分支 `codex/attribute-config-editor` 不与 `toml_input` 调用冲突（已核实）。

### 来自 changes/2026-08-09-113000.md

# core/update P4 — 金标准基准测试

日期: 2026-08-09

## 做了什么

`cal/atkcal.rs` 新增 `#[cfg(test)] mod golden_tests`，6 个测试锁定五段伤害 + 6 属性导数：

| 测试 | 场景 |
|------|------|
| golden_gong_default / gong_buff | 莫问「宫」（base 160/200, atk 2.609375），默认 / 满 buff |
| golden_zheng_default / zheng_buff | 「徵·豪情」（hit_up=20），默认 / 满 buff |
| golden_wei_default / wei_buff | 「剑·徵·削竹」（watk_xishu=100, wushifangyu=90），默认 / 满 buff |

玩家样本: 根骨 18888 / 基础攻击 4666 / 会心 33000 / 会效 22000 / 破防 25000 / 武器 2800。
目标样本: 21000 防御 / 御劲 8500 / 化劲 35000 / 减伤 35%。
buff 场景: 基础攻击 +10%、会心 +5%、会效 +8%、破防 +3%、伤害 +6%。

## 基准来源（关键）

在**改动前的代码**（master `b12818f` 的 jpcg_core，git worktree 临时检出）上，用同输入跑同一探针程序，
得到全部金标准数值；再用同探针跑**改动后**代码（P1+P2 后），输出 `diff` 为**空**，
实证重构前后数值逐位一致，随后将数值固化进断言。

## 保障

- 六项断言对 Q 段精度 `1e-6`（f32 求导为连续近似，基准即原输出）。
- 若未来改动伤害公式/截断顺序，测试立即红。
- `cargo test -p jpcg_core`：6 passed。
- 探针代码已从工作树移除，临时 worktree 已删除。

### 来自 changes/2026-08-10-181440.md

# 同步远端技能编辑器的 TOML 处理边界

## 内容

- 合入远端的结构化技能编辑器接口 `load_skill_data` / `save_skill_data`。
- 移除旧属性编辑器的原始 TOML 文本读写链路，以及前端 `smol-toml` 解析和序列化逻辑。
- 清理仅服务于旧编辑器的页面、图标依赖和验证素材。

## 原因与决策

TOML 的解析、校验和写入统一由 Rust 核心完成；Tauri 在前端和后端之间只传输结构化 DTO。这样避免双端 TOML 实现产生格式和字段兼容性分歧，也与远端的技能编辑器实现保持一致。

### 来自 changes/2026-08-11-203000.md

# K1: jpcg_core 内部分组（io.rs → store/, cal/ → engine/）

## 什么
- `src/io.rs`（396 行）按职责拆分为 `src/store/` 子模块：
  - `store/paths.rs` — data_dir / combo_presets_dir（路径定位）
  - `store/toml.rs` — TomlConfig / toml_input / load_config / save_skill_toml
  - `store/config.rs` — SaveConfig / save_config / load_save_config / export / import
  - `store/combo.rs` — 连招预设 CRUD
  - `store/profession.rs` — 门派列表扫描
  - `store/mod.rs` — 声明 + 兼容重导出（保持 lib.rs 门面引用 `crate::store::`）
- `src/cal.rs` → `src/engine/mod.rs`，`src/cal/` → `src/engine/`（atkcal/derivatives/kill_prob 提升到 engine 下，不留嵌套）

## 为什么
- 为「core 即 host」架构铺路：store 与 engine 将成为 host 的独立子系统，未来可分别打包/替换
- 缩减单文件规模，职责边界清晰

## 关键决策
- `pub mod engine; pub use crate::engine as cal;` — 对外路径 `jpcg_core::cal::*` 保持不变，Tauri 侧零改动（上层的 `jpcg_core::cal::derivatives::` 等引用全部兼容）
- 删除死代码 `io::Config`（grep 确认无任何引用）
- `jcsx_set`/`all_updates`/`player.rs` 依决策保留（后续补充）
- store/mod.rs 顶层重导出全部函数，lib.rs 门面仅改 `io`→`store` 前缀，行为零变化

## 验证
- `cargo build --workspace`：0 error
- `cargo test -p jpcg_core`：6 个金标准全绿（数值逐位锁定，重构无行为变化）

## 备注
- 与 feature 分支 `codex/attribute-config-editor` 的 `io.rs`/`lib.rs` 冲突面已形成（该分支 +41/-? 行），合入时需手工协调

### 来自 changes/2026-08-11-205500.md

# K2: jpcg_api 纯类型契约 + core host/ JSON 入口 + FFI 重写 + 三 crate 双产物

## 什么
1. **新建 `crates/jpcg_api`**：仅含 serde DTO 类型的 rlib crate（22 个 DTO + ConfigDataDTO），零依赖，
   不引用 jpcg_core —— 作为 Tauri IPC 与 FFI JSON 的单一契约源（防漂移）
2. **core 新增 `host/` 模块**（JSON 契约层）：
   - `host/calc.rs` — calculate / compute_derivatives / into_core / skill_dto_to_skilltype
   - `host/combo.rs` — calculate_combo / 预设 CRUD / export/import config
   - `host/config.rs` — save/load_config / list_professions
   - `host/skill.rs` — load/save_skill_data / load_skill_pool
   - `host/conv.rs` — 原 Tauri types.rs 的全部 From 转换迁入（DTO↔core 双向）
   - `host/update.rs`（net 特性）— update 编排全权收归 core：check/download/验证/**发动
     jpcg_updater 子进程**/请求退出；异步经内嵌 tokio runtime 同步化；
     宿主行为通过 `HostEvents` trait（on_progress/request_exit/updater_path）注入
3. **`ffi.rs` 整体重写**（原 26 行裸指针 → 句柄 + JSON 协议）：
   - `jpcg_abi_version()` 版本协商、`jpcg_handle_create/free`、`jpcg_call(handle, method, request_json)`
   - `jpcg_free_string` / `jpcg_last_error` 错误传递
   - 所有实现经 `catch_unwind` 防护，panic 不跨 FFI
   - 覆盖 16 个业务方法（calculate/derivatives/combo/config/skill/预设 CRUD）
4. **三 crate 双产物**：`jpcg_core`/`jpcg_const`/`jpcg_update` 均 `crate-type = ["cdylib", "rlib"]`，
   产出 libjpcg_core.dylib / libjpcg_const.dylib / libjpcg_update.dylib
5. **`jpcg_const` FFI**：`jpcg_const_get_u32(key)` 按 key 查药物常量
6. **`jpcg_update` FFI**：`jpcg_update_check` / `jpcg_update_fetch_app_info` / `jpcg_update_file_sha256`
   （同步化封装，JSON 进出）
7. core 增加 feature `net`（默认开）：jpcg_update + tokio 为可选依赖

## 为什么
- B 模式（全动态 dlopen）的必要前提：core dll 对外只暴露 JSON 契约，Tauri 壳动态模式
  无需编译期依赖 core 类型
- 「更新时只更新某个 dll」依赖每 crate 独立 cdylib 产物

## 关键决策
- DTO 单源在 `jpcg_api`，From 转换单源在 `host/conv.rs`，Tauri 壳 types.rs 后续（K3）改为 `pub use`
- update 流程（含 updater 启动）从 Tauri 壳迁入 core 的 host::update，壳只实现 HostEvents
- FFI 方法名与 Tauri 命令一一对应，未来可直接生成桥接层

## 验证
- `cargo test -p jpcg_core`：12 passed（6 金标准 + 6 FFI 协议冒烟：句柄生命周期/JSON 往返/
  错误路径/版本协商/save_config 契约）
- `cargo build --workspace`：0 error
- 三个 dylib 产物确认生成

## 备注
- jcsx_set 死代码警告保留（既定决策：后续补充）
- FFI 冒烟测试写 saved_config.toml 到 CWD 后自行清理

### 来自 changes/2026-08-11-213000.md

# K3: Tauri 双模式（static/dynamic）+ 命令薄壳化 + FFI 桥接

## 什么
1. **Tauri Cargo features 双模式**：
   - `default = ["static"]` — 编译期链接 jpcg_core，Rust 直调 host 模块
   - `dynamic = ["libloading"]` — 不依赖 jpcg_core，dlopen libjpcg_core 经 jpcg_call JSON 调用
   - jpcg_update 改为两模式共同依赖（命令签名/进度事件 DTO 需要，计算逻辑仍在 core）
2. **types.rs 薄壳化**：511 行 DTO+From 实现 → 3 行 `pub use jpcg_api::*`
   （DTO 与转换单源在 jpcg_api / jpcg_core::host::conv）
3. **命令薄壳化**（calculate/optimize/config/data/combo/skill_editor/update）：
   每个命令 = tauri::command 签名不变 + `cfg(feature)` 两个 impl：
   - static：直调 `jpcg_core::host::*`
   - dynamic：`ffi_bridge::call(method, &req)`（DTO 自动 JSON 序列化往返）
4. **commands/ffi_bridge.rs**（dynamic 模式）：
   - 定位 libjpcg_core.{dylib,so,dll}：exe 目录 → target/{debug,release} → `JPCG_CORE_LIB` 环境变量
   - OnceLock 单例加载库 + 创建会话句柄 + Drop 释放句柄
   - `jpcg_set_host_events` 注册 C 回调表（进度 → Tauri emit "update-progress"；
     退出 → app.exit(0)；updater 路径 → JPCG_UPDATER_PATH 环境变量或 null）
5. **HostEventsTable 类型单源化**：定义移到 jpcg_api（repr(C) C ABI 契约），
   core ffi re-export，app 无链接即可构造回调表
6. **行为等价性修复**：`load_config` 的 ConfigDataDTO 改为完全镜像
   CalculateRequest 形状（player/hostile/**xinfa_config**/buff/coefficient），
   保持前端 loadConfig() 契约不变（原实现返回 CalculateRequest，且含 buff/coefficient）
7. **DTO 补 Deserialize**：SkillResultDTO / ComboResultDTO / ComboStepResultDTO /
   jpcg_update 的 UpdateProgressEvent / UpdateCheckResult（dynamic 模式 JSON 回读需要）
8. **update 编排所有权**：Tauri 壳 update.rs 从 160 行 → 双模式薄壳；
   perform_app_update（下载→验证→发动 updater→请求退出）已在 K2 迁入
   jpcg_core::host::update，壳只实现 HostEvents（static）或注册 C 回调（dynamic）

## 为什么
- B 模式（全动态更新）的 Tauri 侧基础：切换 feature 即可静态/动态编译，
  前端代码零改动（命令名、参数、返回形状全部不变）
- core dll 更新换代时 app 二进制无需重装；dev 时 JPCG_CORE_LIB 指向本地 dylib

## 关键决策
- 命令签名保持前端契约优先：tauri::command 参数/返回类型与 K2 之前完全一致
- jpcg_update 不并入 optional：其 DTO 是命令签名的一部分，逻辑轻量；
  真正要"可替换"的计算引擎是 core
- 动态模式句柄懒创建于首次 call（OnceLock 单例，进程级复用）
- 回调表幂等注册（OnceLock guard），避免重复注册

## 验证
- `cargo build -p jpcg_app`（static）与 `--no-default-features --features dynamic` 均 0 error 0 warning
- dynamic 端到端冒烟测试（ffi_bridge_tests::dynamic_call_roundtrip）：
  真实 dlopen libjpcg_core.dylib → jpcg_call("list_professions") JSON 往返 → 错误路径
- `cargo test --workspace` 全绿（13 passed）
- `cargo build --workspace` 仅剩既定 jcsx_set dead_code 警告

## 备注
- 前端 src/api/commands.ts 无需改动（命令契约不变）
- 动态模式实际运行需 libjpcg_core.dylib 与 app 同目录（打包时由 K5 构建矩阵保证）

### 来自 changes/2026-08-11-220000.md

# K5: 模块库（dll）增量更新 — modules_manifest.toml 全链路

## 什么
1. **manifest-gen 扩展**：新增 `--modules-dir <dll目录>`（可选）与 `--modules-output`
   （默认 modules_manifest.toml）与 `--platform`（默认按本机 OS 推断 darwin/linux/windows），
   产出：
   ```toml
   modules_version = "v2.1.0"
   platform = "darwin"
   [[files]] name/hash/hash_type/size
   ```
   与 data_manifest.toml 并行生成
2. **jpcg_update::modules（新模块）**：
   - `ModulesManifest` / `ModulesFileEntry` / `ModulesCheckResult` 类型
   - `fetch_modules_manifest`：stable → `files/JPCG/{app_version}/modules/…`，
     beta → `files/JPCG_beta/modules/…`（与 data 布局对齐）
   - `check_modules_update(beta, force)`：对比 exe 同目录 `modules/` 子目录，
     按 SHA256 差量判定需要更新的 dll
   - `download_and_install_modules`：逐文件下载 → 哈希校验 → 临时名原子替换
3. **UpdateCheckResult 扩展**：新增 `has_modules_update` / `modules_version` /
   `modules_files_to_update`；`check_updates` 在 force 或有 app 更新时一并检查模块
4. **core host::update::perform_modules_update**：下载 → 校验 → 安装 → 进度上报 →
   `request_exit()` 请求宿主重启（重启后 ffi_bridge 优先加载 modules/ 新 dll）
5. **FFI**：`update_modules` 方法（请求携带 beta/version/files，经回调表上报进度）
6. **Tauri**：`perform_modules_update` 命令（static 直调 / dynamic 走桥接）；
   **ffi_bridge 加载顺序调整**：exe 同目录 `modules/` 子目录 → exe 同目录 → target → env
7. **前端**：types 增加 ModulesFileEntry 与 UpdateCheckResult 三字段；
   commands.ts 增加 `performModulesUpdate` + checkUpdate/perform 模拟响应；
   ConfigPanel 更新流程新增模块更新分支（确认 → 进度 → 重启）

## 为什么
- B 模式（增量只换 dll，不重装 app）的服务端与客户端能力打通：
  服务器托管 modules_manifest.toml + dll，app 差量下载到 modules/ 并重启加载

## 关键决策
- 模块版本目录复用 app 版本号（app 更新与模块更新同版本目录，服务器部署简单）
- 本地模块落位 exe 同目录 modules/，与 data/ 平行；ffi_bridge 加载优先级
  modules/ 最高 → 旧 dll 原位替换后重启即生效
- beta 通道模块清单不带版本目录（与 data 的 beta 布局一致）

## 验证
- manifest-gen：真实 dylib 生成 modules_manifest.toml（3 个模块，darwin）
- `cargo test --workspace` 全绿（13 passed）
- `cargo build --workspace` 双模式 0 error；`make check-all` 全部通过
- `npx tsc --noEmit` 前端类型检查通过

## 备注
- 服务器部署步骤（参考 server_manifest.md）：`dist/<version>/modules/` 放 dll +
  modules_manifest.toml；beta 放 `dist/modules/`
- 动态模式 app 的 check_update 会请求服务器 modules 清单；服务器暂无 modules 目录时
  自动跳过（fetch 404 视为无更新）

### 来自 changes/2026-08-11-220100.md

# K4: 双构建矩阵 — Makefile + .cargo/config.toml 别名

## 什么
1. **`.cargo/config.toml` 新增 `[alias]`**：
   - `build-app-static`  → `build -p jpcg_app`（默认，编译期链接 core）
   - `build-app-dynamic` → `build -p jpcg_app --no-default-features --features dynamic`
   - `build-modules`     → `build -p jpcg_core -p jpcg_update -p jpcg_const`（三个 cdylib）
   - `test-core` / `test-app-dynamic`
2. **`Makefile`**：
   - `build-static` / `build-dynamic` / `build-modules`（支持 `BUILD=release`）
   - `modules-dir`：把三个 dll 复制到与 app 相同目录（动态模式运行所需）
   - `test` / `check-all`：双模式 + 全 workspace 测试全绿检查

## 为什么
- 把 K2/K3 的两种 app 构建模式固化为一条命令，消除手写 feature 组合的记忆负担

## 验证
- `make check-all` 全部通过（static + dynamic + modules 构建 + workspace 测试）
- `cargo build-modules` / `cargo build-app-dynamic` / `cargo test-core` 别名生效
- 首次失败点：`[alias]` 表头缺失导致 cargo 不识别别名，已补

### 来自 changes/2026-08-11-223000.md

# K6: ctypes Python demo + JPCG_DATA_DIR 环境变量覆盖

## 什么
1. **examples/python_demo/jpcg_demo.py**：ctypes 直连 libjpcg_core.{dylib,so,dll}：
   - 自动定位库（参数 / target/{debug,release}）
   - 完整协议演示：abi_version → handle_create → list_professions →
     calculate（莫问金标准输入）→ 错误路径 → 句柄释放
2. **store::paths::data_dir() 支持 `JPCG_DATA_DIR` 环境变量**：
   - 指向含 `shuxing/` 子目录的目录（或直接是 shuxing 目录）时优先返回
   - 解决 CLI/Python 等 current_exe 不可用场景的数据定位问题
   - Tauri 正常运行路径不受影响（无环境变量时行为不变）

## 为什么
- 跨语言使用 core 的真实场景验证：Python 直接经 JSON 协议调用获得完整计算结果
- 嵌入式（dll 宿主非 JPCG exe）时数据目录无法从 current_exe 推断，需要显式注入

## 验证
- `JPCG_DATA_DIR=./data python3 examples/python_demo/jpcg_demo.py`：
  列出 2 个门派（莫问/周天功），calculate 返回 28 个技能的完整伤害数据
- `cargo test -p jpcg_core` 连跑 3 次 12 passed（此前一次偶发失败已复测稳定）
- `cargo build --workspace` 0 error

### 来自 changes/2026-08-11-231500.md

# M1: 完整维护流程落地（git-flow + CI/CD + 组件独立版本）

## 什么
1. **版本模型（组件独立）**
   - data_version / jpcg_const 采用 `等级.赛季.日期`（如 `130.3.20260602`），
     映射现有 shuxing 数据 `{level, season, modified}`（modified=YYYYMMDD）
   - `jpcg_const` 独立版本（Cargo version = 130.3.20260602）；`jpcg_updater` 独立；
     其余 crates `version.workspace = true` 继承根 workspace（core 版本 = release tag / 安装包命名源）
   - `VersionInfo::compact()` / `label()`：数据版本紧凑串 + UI 美化（`130级第3赛季 (2026-06-02)`）
   - FFI 版本 getter：`jpcg_core_version` / `jpcg_const_version` / `jpcg_update_version` +
     各 crate `*_VERSION` 常量；Tauri `get_module_versions` 命令（static/dynamic 双模式），
     ConfigPanel 底部展示 App/Core/Update/Const 版本
2. **git-flow 分支初始化**：master(生产) + 新建 develop(集成交互) 并推送
3. **CI/CD（.github/workflows/）**
   - ci.yml：PR/push 触发，rustfmt + clippy + 双模式构建/测试 + 金标准 + 前端 build
   - release.yml：tag v* 触发，三平台矩阵构建（app/dll/updater）+ manifest 生成 + GitHub Release
   - deps.yml + dependabot.yml：依赖审计与每周更新
4. **模板与文档**：PULL_REQUEST_TEMPLATE / ISSUE_TEMPLATE(bug/feature) /
   CONTRIBUTING.md（git-flow + commit 规范 + 版本管理 + 发布流程） / CHANGELOG.md（Keep a Changelog）
5. **脚本与工具**：scripts/release.sh（测试→bump→聚合 CHANGELOG→tag→push）、
   scripts/sync-version.sh（同步 package.json/tauri.conf.json/commands.ts 模拟串）、
   rustfmt.toml、.pre-commit-config.yaml；安装 cargo-edit（cargo set-version）
6. **K5 增强**：modules_manifest 逐 dll 带 `version`+`sha256`，`check_modules_update`
   逐 dll 比较（缺条目/版本不同/哈希不同），本地 `modules/modules_manifest.toml` 快照，
   安装后合并写回快照；manifest-gen 从 dll 文件名推断 crate 版本
7. **clippy 基线**：修复 `jpcg_const_get_u32`（裸指针解引用标记 unsafe + Safety 文档，
   deny-by-default lint）；CI clippy 不 `-D warnings`（既有技术债众多）

## 为什么
- 从无 CI/无分支策略/版本散落 5 处的状态，建立标准化的协作与发布流程
- 组件独立版本契合 B 模式（dll 独立替换）与数据/常量频繁调整的现实
- release tag 跟 core，安装包命名用 core 版本，UI 独立展示

## 关键决策
- 采用 git-flow（master+develop）而非 GitHub Flow：版本化桌面软件 + 定时发布
- 版本用 cargo 原生机制（`version.workspace = true` + `cargo set-version`）而非
  cargo-workspace-version：后者不解析 glob 成员且会覆盖独立版本（const/updater），与组件独立模型冲突
- 模块更新改为逐 dll 版本/哈希判断，本地快照
- CI clippy 暂不 `-D warnings`（既有 ~30 处告警，逐步清理）；release 矩阵用免费额度

## 验证
- `make check-all` 全绿（双模式构建 + workspace 测试）
- 金标准 12 passed；`cargo clippy --workspace --all-targets` 无编译错误
- `cargo fmt --all -- --check` 通过；前端 `npx tsc --noEmit` 通过
- manifest-gen 正确生成逐 dll 版本（core=2.1.0-alpha.1、const=130.3.20260602）
- `cargo metadata` 确认各 crate 版本对齐

## 备注
- master/develop 分支保护（强 review + CI 绿 + squash + 禁直接 push）需在 GitHub Settings 手动开启
- develop 已从 master 创建并推送；core/update 待经 PR 合入 develop

### 来自 changes/2026-08-11-232000.md

# M2: 开源规划文档 — AGENTS.md 更新 + 新增 PLAN.md

## 什么
1. **AGENTS.md 更新**：
   - 删除过时行 "No CI, test suites, or formatter config exist"（与已落地的 CI/测试/rustfmt 矛盾）
   - 新增「开源与命名（当前决策）」段，记录：
     - 保留 `JPCG` 命名（改名"试剑/演武"等已搁置；范围仅产品名，内部 crate/FFI/env 不动）
     - `data/` 暂留仓库；未来联系其他项目负责人获取 lua 数据访问权限后再定发布方式
     - 开源状态：仓库私有（GitHub Free），分支保护需转公有或 Team
     - 场景扩展：core 引擎本就场景无关，PVE 支持主要是 UI 预设 + 数据模板
2. **新增 PLAN.md**：开源与发展路线图（背景 → 当前状态 → M0-M4 里程碑 → 风险合规 → 决策记录）
   - M1 转公有与分支保护、M2 数据授权与合规、M3 开源运营、M4 场景扩展

## 为什么
- 固化"先保留现状、未来再评估改名/数据授权"的决策，供后续协作与开源时遵循
- PLAN.md 作为开源路线的单一事实来源

## 关键决策
- 暂不改名、暂不动 `data/`；数据最终形态由 M2（上游 lua 授权结果）决定
- 开源 = 转公有（免费获得完整分支保护），而非升级 Team

## 备注
- 用户将据本 PLAN 执行转公有；M2 需主动联系上游项目负责人

### 来自 changes/2026-08-11-235000.md

# M3: 开源许可落地 — GPL-3.0 LICENSE + 各 manifest 统一

## 什么
- 新增根 `LICENSE`（GPL-3.0 全文，674 行）
- `jpcg_updater` / `jpcg_app` 补充 `license.workspace = true`（此前缺）
- 前端 `examples/jpcg_app/package.json` 补 `license: "GPL-3.0"`
- 全 8 个 Rust crate 经 `cargo metadata` 确认均解析为 `GPL-3.0`

## 为什么
- 开源前补齐许可；Cargo.toml 已声明 GPL-3.0，落地 LICENSE 文件并统一所有成员

## 关键决策
- 选 GPL-3.0（强 copyleft）：与社区工具(JX3Toy)一致，保证衍生品开源；
  与 PLAN M2 数据授权（上游多宽松/GPL）可兼容，后续拿到具体协议再复核

### 来自 changes/2026-08-12-000000.md

# M4: 三分支线性迁移（dev/beta/release）+ 发布脚本重写

## 什么
1. **分支重构**：从 git-flow（master/develop）迁移到三分支线性模型：
   - `develop` → `dev`（集成/最上游，版本 alpha.n）
   - 新建 `beta`（预发布/公测，版本 beta.n）
   - `master` → `release`（稳定/生产，版本 X.Y.Z）
   - 线性关系：`release ⊂ beta ⊂ dev`，单向超集链
2. **远端操作**：推送 dev/beta/release，设默认分支为 dev，删除旧 develop/master
   （先删保护规则再删分支）
3. **分支保护**：dev/beta/release 三条规则（review≥1 + 3 个 CI 检查 + linear +
   conversation + 禁强推/删除 + enforce_admins:true；beta/release 为 strict）
4. **PR 迁移**：原 #3（LICENSE）因 base 分支删除被自动关闭，重建为 #4（feature/license → dev）
5. **scripts/release.sh 重写**：`--stage alpha|beta|release`
   - alpha（dev）：不 tag；beta：tag vX.Y.Z-beta.n；release：tag vX.Y.Z
   - 校验所在分支与 stage 匹配；测试 → bump → 聚合 CHANGELOG → commit → tag → push
6. **CI/CD 适配**：ci.yml 触发改为 dev/beta/release；release.yml 按 tag 判定
   beta/stable 通道（tag 含 -beta → beta 通道）
7. **文档**：CONTRIBUTING / AGENTS / PLAN 更新为三分支模型

## 为什么
- 用户要求：所有更改融合到 dev → 出 beta.n 公测 → 稳定后 release，线性链式发布

## 关键决策
- release 初始指向旧 master 基线（65e3da1，最后稳定）；beta 同指向旧基线，
  首次提升 dev→beta 才填充
- enforce_admins 保持 true（最严格，需第二 reviewer 或临时降级）
- tag 区分通道：vX.Y.Z-beta.n→beta，vX.Y.Z→stable

## 备注
- PR #4（feature/license → dev）待合并；dev 已含全部 core/update 工作
- 后续出 beta/release 用 `scripts/release.sh beta|release`

### 来自 changes/2026-08-12-100000.md

# UI: 薄荷/松石绿清新主题 + 自绘 SVG 图标

## 什么
1. **配色重构**（`variables.css`）：从深色靛蓝/紫改为薄荷/松石绿
   - 主色 teal `#14b8a6` → sky `#0ea5e9` 渐变
   - **浅色为默认**（`:root`），保留深色主题切换
   - 更新 useTheme 默认主题为 light
2. **自绘 SVG 图标**（新增 `components/icons.tsx`，~18 个内联线条图标，
   stroke=currentColor 继承主题色，不依赖第三方库）：calc/globe/combo/trend/pencil/
   chevron/sun/moon/gear/trash/save/star/close/check/alert/bug
3. **替换全部 emoji**（App 反馈按钮/logo、ActivityBar 页签+折叠、ThemeToggle、
   Toast 状态、ForumPage 翻页、ComboPage 操作、OptimizePage 关闭等）
4. **排版/美术打磨**：卡片阴影、ActivityBar 激活药丸、Header Logo 图标、按钮/输入
   聚焦青色光晕、Toast 状态图标着色、StatusBar 状态点光晕
5. **清理硬编码旧色**：`rgba(99,102,241)`(indigo) / `rgba(168,85,247)`(purple) 统一为 teal/sky
6. **死代码清理**：删除过时 `src/index.html`、`src/css/style.css`（引不存在的 js/app.js、
   css/components.css，React 未使用）

## 为什么
- 用户要求"清新配色 + 排版美术"；采用薄荷/松石绿（A 方案），emoji 改自绘 SVG

## 关键决策
- 图标全部内联 SVG、无新依赖；currentColor 保证跟随浅/深主题
- 浅色为默认（清新观感），深色仍可切换
- 删除死代码降低混淆

## 验证
- `npm run build`（tsc + vite）通过
- `cargo build -p jpcg_app` 通过
- emoji 全量替换后无残留

### 来自 changes/2026-08-12-224904.md

# server_tools/data-import：打通 libpak 拆包（读 skills.tab + 提取 lua + base_damage）

## 什么
1. **修复并打通 PC Pakv4 拆包管线**（此前 `pak.rs` 一直失败）：
   - 正确的 libpak FFI：`init(path)` / `tab_init(path, indexs, fields)` 3 参数 /
     `tab_get(tabname, key, buf, len)` 4 参数（key 带尾随 `\t`，返回字节长度）/ `lua_get`
   - libpak 需 `./cache`（CWD 相对）jx3calc 缓存，缺失时 `init` 返回 -1；
     缓存源在 `~/Library/Application Support/JX3/AssetsDatCache/zsCache/`
   - `tab_init` 的 fields 必须**精确匹配 skills.tab 表头**（用 jx3calc `Skill` 枚举 20 列），
     传错字段名返回 -1；正确字段集下一次 tab_init + 连续 tab_get 数万技能稳定
2. **新增 `src/pakx.rs`**：封装上述正确用法 + `--pak-dump` 模式
   - 遍历 SkillID → 读 skills.tab 20 列 → 提取每个技能 lua 字节码到 `lua/` → 用
     patched luac 反编译解析 `tSkillData[].nDamageBase` → 输出 `skills.tsv` manifest
3. **`main.rs` 新增 `--pak-dump` / `--pak-out` / `--luac` 参数**

## 为什么
- 用户要求"跑通游戏文件拆包"。此前 `pak.rs` 用错 FFI 签名/字段名导致 init 失败、
  tab_init 卡死。本次彻底摸清 libpak 用法并跑通全流程。

## 关键决策 / 发现
- **base_damage 不在 skills.tab**，而在技能 lua 的 `tSkillData[level].nDamageBase`
  （满级值；真实伤害在"实际伤害子技能"如 14474 宫，父技能 14064 仅 L1=100）
- **atk_xishu 不在 skills.tab 或 lua 常量**：为 jx3calc 执行 `GetSkillLevelData` 后派生
  （内功 = frames/192，外功 = frames/160，frames 每技能整数），需 lua 执行引擎，
  当前未在本次落地（下一步 P0）
- `SkillCoefficient` 列仅 NPC/副本技能有值（固定 25600），玩家技能为空

## 验证
- 全量扫描：读取 35303 技能，提取 22115 个 lua，解析 22115 个 base_damage
- 抽查莫问：14474宫=160、14311商=110、14100羽=110，与现有 mowen.toml 一致
- `cargo build -p data-import` 0 error（仅既有/无害 warning）

## 备注
- 运行前需：`mkdir cache && cp ~/Library/Application Support/JX3/AssetsDatCache/zsCache/* cache/`
- atk_xishu（系数）依赖 lua 执行引擎，下一步用 mlua 复刻 jx3calc 的 GetSkillLevelData 派生

### 来自 changes/2026-08-12-230000.md

# data-import + 引擎：全等级技能数据 → 嵌套 levels TOML → 计算器选等级

## 什么
1. **Lua 执行器 `src/luaex.rs`**：用 mlua(Lua 5.1) 执行游戏技能 lua 的 `GetSkillLevelData`。
   - patch mlua vendored Lua 源码（lundump.c 跳过 header + 4字节字符串）加载魔改字节码
   - stub 游戏全局（Include/ATTRIBUTE_TYPE/ATTRIBUTE_EFFECT_MODE/tSkillData/...）
   - 捕获 `SKILL_*_DAMAGE`(base1)、`_RAND`(rand)、`nChannelInterval`(frames)
2. **全等级提取 `--gen-data`**：遍历所有技能×等级，算
   - `base_damage1 = SKILL_*_DAMAGE`、`base_damage2 = base1 + RAND`
   - `atk_xishu = nChannelInterval / 192(内功) 或 /160(外功)`（按 KindType）
   - 输出 TSV（skill_id/name/script_file/kind_type/max_level + 每级 base1/base2/atk）
   - 当前提取 1888 个有伤害数据的技能
3. **TOML 生成 `gen_toml.py`**：按门派目录分组 → 输出**嵌套 levels** TOML（14 门派）
   - `[[skill]]` 内嵌 `[[skill.level]]`（level/base_damage1/base_damage2/atk_xishu）
4. **引擎等级支持**：
   - `Skilltype` 加 `levels: Vec<SkillLevel>`（serde rename="level"）+ `current_level`
   - `SkillLevel` 结构 + `select_level()`/`max_level()` 方法
   - `SkillEditorItemDTO` 加 `levels`/`current_level`，conv.rs 双向映射
   - store 模块公开
5. **前端等级选择器**：SkillEditorPage 伤害系数区加等级下拉，选级应用 base/atk

## 为什么
- 用户要"输出全等级，让用户在计算器内自己选等级"。
- 解出核心公式：base_damage 在 lua tSkillData（执行 GetSkillLevelData），
  atk_xishu = frames/192(内功)/160(外功)，frames=nChannelInterval。
- 251203.json(2025.12) 系数已过时（技改 nerf，如宫 2.609→1.7325），必须从当前 pak 提取。

## 关键决策
- **纯解包，忽略 csv**：一切 base_damage/atk_xishu 从当前 lua 执行得出
- 嵌套 `[[skill.level]]` 格式（一个技能一条，内嵌多级）
- base_damage 需执行 lua（非满级常量）：商= tSkillData[1]*0.7=14
- 等级规则：暂用 skills.tab max_level（宫25/商3）

## 数据源限制
- **段氏/周天功、无相楼（幽罗引）不在本地 pak**：PC Pakv4 段氏有元数据无 lua，
  无相楼连 skills.tab 都没有；无界数据(2025.2)更早无此门派。需新数据源（Pakv5/新版客户端）。

## 验证
- `cargo test -p jpcg_core --test levels_parse`：解析嵌套 levels + select_level 通过
- 实际生成的 mowen.toml 能被引擎加载并选等级
- `cargo build --workspace` 0 error
- 前端 `npx tsc --noEmit` 通过

## 备注
- 生成的 TOML 用的是 skills.tab 原始技能名（如"宫伤害_实际伤害子技能"），
  含全部子技能，非玩家向精简清单——技能名/筛选需后续整理
- gen_data 中 13188 技能无 lua（NPC/被动/路径缺失），核心玩家伤害技能已覆盖

### 来自 changes/2026-08-13-101500.md

# 全门派 shuxing 扩充至 19 门派（老门派数据补全）

## 变更
- `data/shuxing/` 从 14 门派扩充至 **19 门派**，新增：
  - 老四门：纯阳(97)、天策(86)、明教(69)、少林(72)
  - 其他：丐帮(133)、七秀(86)、万花(98)、五毒(86)、唐门(5→112)
  - 扩充：藏剑(1→61)、纯阳(9→97)、天策(8→86)、明教(2→69)
- 全部 TOML `modified` 更新为 20260813

## 起因
- 老门派（纯阳四象轮回等）技能 lua 之前缺失：PakV4 客户端（zhcn_hd）经
  PakV4-Extract（jx3pak 组织工具，Engine_Lua5X64.dll + `KG_InitPakV4FileSystem`
  加载 Trunk.Dir）黑盒探测 1888 个已知技能路径，仅 6 个 NPC/test 命中——
  **决定性确认玩家门派技能 lua 不在 PakV4**。
- Trunk.Dir 为 HTree 哈希树（`[PakV4] Is Use HashName`，nHeight=5），无明文路径；
  StreamDownloader 日志证实客户端数据完整（无增量下载）。

## 数据来源
- 无界缓存 `~/Library/Application Support/JX3/AssetsDatCache/zsCache/` 更新后
  （用户 8/12-13 游戏运行写入新 cache），libpak 全量提取从 1888 → **3448 技能**。
- 老门派技能路径格式为 `纯阳\子技能_四象轮回_非定身.lua`（子技能_/门派名前缀），
  与旧格式不同（这是之前探测失败原因）。
- 长歌/霸刀/苍云/凌雪/药宗/衍天 6 门派数据与旧版完全一致（回归验证通过）。

## 验证
- `cargo test --workspace`：14 通过 0 失败（含 6 金标准）
- `parse_generated_mowen_and_select` 通过（新 TOML 解析 + 等级选择）
- 前端 `tsc --noEmit` 通过

## 遗留
- 纯阳/少林/天策/明教为**全技能**数据（含子技能），尚未按心法奇穴手动核对
- `server_tools/data-import/` 的 gen_toml.py 的 `modified` 日期为硬编码，后续可参数化

### 来自 changes/2026-08-13-114500.md

# 无相楼（幽罗引）加入 —— 20 门派完成

## 变更
- 新增 `data/shuxing/wuxianglou.toml`（无相楼/幽罗引，第 20 门派）
- 25 个伤害技能（skill_id 41116-42417 段）：挑丝(41342, atk=2.5)、绊线(41444, atk=3.75)、
  勾线(2.1979)、四边静(2.0833)、千里急一~四段(0.7291/1.0937/1.4583/2.1875)、生地狱、
  锁南枝、傀梦令、障幕虚影、应天长(7.2916) 等，均为单级（level=1）
- 心法：幽罗引，根骨阴性内功，atk_up=1.96 / pofang_up=2.0

## 无相楼数据特性
- **无 base_damage**（用户确认无相楼技能无 base，只有 atk 系数）
- atk_xishu 取自魔盒 skill API（node.jx3box.com/resource/std/skill/list include=parse）的
  技能描述 `[X*最终阴性内功攻击]`，系数准确

## 背景
- 无相楼技能 lua 不在本地 PakV4/无界缓存（zsCache dat 无，libpak 确认 41342 不在
  skills.tab 35303 行）
- PakV5 FileList 服务下载持续失败（vk_mb/mac_mb/ios_mb/android_mb 全试，CrossOver 与
  wine11 均 bad FileList）；KGPK5 hash 函数 wine 调用崩溃（日志未初始化）
- `~/Library/Application Support/JX3/cache/skill_m.cache`（34MB）含全部 179 个无相楼
  技能脚本路径（GBK 明文），验证了技能名与 ID 对应（挑丝=织心谣_丝线穿刺等）

## 验证
- `cargo test -p jpcg_core`：12+1 全过（含 6 金标准、parse_generated_mowen_and_select）
- modified = 20260813

## 遗留
- 无相楼仅 25 个有伤害系数的技能；纯控制/机制技能（夹线/替形幻生等）未收录（atk=0）
- base_damage=0（游戏设计如此，非缺失）
- 若后续打通 PakV5 数据下载，可补全子技能与等级数据

### 来自 changes/2026-08-13-160000.md

# 撤销 toml 数据与 data-import 解析器提交（等待完整新数据）

## 变更
- `feature/ui-fresh` 分支 `git reset --hard 571c098`，移除 5 个提交：
  - `fa21db8`/`248bd14`/`5ffc9e0`/`375d943`：data-import 生成器/解析器（libpak 提取、luaex、按心法分类）
  - `c564220`：全门派全等级 shuxing 数据 + 引擎/前端等级支持
- 删除工作区残留：`server_tools/data-import/`（探针工具）、`luac.out`
- 保留 4 个探索 changelog（2026-08-12-224904/230000、2026-08-13-101500/114500）作为历史记录
- PR #6（feature/ui-fresh → dev）force push 后仅含 UI 主题提交

## 起因
- 现有 TOML 数据与游戏当前版本数值不一致（宫 atk_xishu 1.7325 vs 实测 2.6095，差 1.506 倍）：
  数据源是 2025.2 旧缓存快照（AssetsDatCache），与正式服版本脱节
- 已获得完整的新数据来源，明天送达；届时平铺单等级数据直接替换 `data/shuxing/*.toml`，
  无需 data-import 工具与等级（levels）功能

## 遗留
- 等级功能（Skilltype.levels/select_level/current_level、DTO、SkillEditorPage 等级选择器）随提交移除，
  新数据不需要；如将来需要可从 git 历史恢复

### 来自 changes/2026-08-13-161500.md

# DOT 持续伤害计算实现（每跳等比递增 + 结果展开）

## 变更
- **数据契约**：`Skilltype.dot_num` 移除，新增 `dot_interval`（每跳间隔秒）/`dot_duration`（持续秒）；
  总跳数 = `dot_duration / dot_interval`（`Skilltype::dot_jump_count()`，非法数据返回 0）
  - 同步：`jpcg_api::SkillEditorItemDTO`、`host/conv.rs` 双向映射、前端 `types/index.ts`、SkillEditorPage（Dot跳数 → 每跳间隔/持续时长）
- **引擎**（`engine/atkcal.rs`）：
  - `dot_jump_expected()`：每跳期望 = 首跳期望 × 等比递增 `(1+dot_up)^(k-1)`（k=0..n-1）
  - `q_cal()`：dot 技能 `q_damage` = 各跳期望之和（总伤害），`DamageResult.dot_jumps` 返回每跳集合
  - `q_cal_with_derivatives()`：导数链整体 × 等比和因子 `((1+u)^n-1)/u`（u=0 退化为 n 倍）
- **输出**：`SkillResultDTO.dot_jumps: Vec<u32>`（计算面板）+ `ComboStepResultDTO.dot_jumps`（连招每步）
- **前端**：ResultTable 与 ComboPage 在期望列下展开每跳伤害（小字标签，hover 显示第 k 跳）
- **金标准**：`golden_gong_dot`（宫 + 6 跳等比 8%，手算期望锁定公式）

## 起因
- dot 字段（dot_flag/dot_up）此前仅定义未接线，dot 技能被当单次伤害计算（低估 6 倍）
- 用户实测：商 DOT 每 3 秒一跳共 18 秒（6 跳），面板每跳倍率 0.2080（正式服数据明天送达后校准）

## 验证
- `cargo test --workspace`：全过（含 7 金标准）
- `cargo clippy -p jpcg_core`：无新增警告
- 前端 `tsc --noEmit` 通过

## 遗留
- 正式服真实 dot_up/dot_interval/dot_duration 数值随明日新数据入库后按金标准校准
- 连招 kill_prob 的方差对 dot 技能按单跳近似（每跳独立会心未逐跳展开）

### 来自 changes/2026-08-13-170000.md

# 真实面板属性金标准 + Python 全量快测 + DOT 数据完善

## 变更
- **数据完善**（`data/shuxing/mowen.toml`）：商（dot）/角（dot）补 `dot_interval`/`dot_duration`
  - 普通版：3 秒一跳、18 秒（6 跳，不递增）
  - 疏曲版：2 秒一跳、18 秒（9 跳），`dot_up` 0.2 → **0.12**（每跳等比递增 12%）
- **金标准替换**（`crates/jpcg_core/src/engine/atkcal.rs`）：
  - 输入改为真实面板属性（2026-08-13 用户提供）：基础属 21371、基础攻击 64329、会心 61877、
    会效 2925、破防 109160；目标外防 15176、内防 21388、御劲 5047、化劲 59402、减伤 0；pvp 0.9
  - 期望值由引擎输出回填（与 python 快测输出完全一致），锁行为待实测校准
  - dot 金标准重写：`golden_shang_dot`（普通 6 跳相等 + q=Σ）与 `golden_shang_dot_shuqu`
    （疏曲 9 跳等比 1.12 + q=Σ）
- **新增 `examples/python_demo/quick_calc_test.py`**：直连 libjpcg_core 全量计算整个 toml
  - 预设真实属性（面板/木桩/pvp 0.9），输出全部技能 Y/B/I/N/H/Q + dot_jumps 集合
  - DOT 断言：普通/疏曲跳数与等比规则、q=Σ、非 dot 为空
  - 面板对照：化劲 76.32%、御劲 2.55%、外防 10.75%、内防 14.51%（引擎系数全部吻合）
  - `--data`/`--lib`/`--out` 参数；退出码 0/1

## 起因
- 旧金标准使用虚构属性（18888/4666），与实际游戏脱节；用户提供真实面板数据作为基准
- DOT 字段此前缺失 interval/duration，无法计算跳数

## 验证
- `cargo test --workspace` 全过（含 8 金标准，新值 = python 快测输出）
- `python3 examples/python_demo/quick_calc_test.py` ALL PASS
- `cargo clippy -p jpcg_core` 无新增警告；前端 `tsc --noEmit` 通过

## 遗留
- 金标准期望值为引擎自锁，待用户木桩实测（宫/商/徵 等）后校准关键值
- 明天完整新数据到货后：替换 `data/shuxing/*.toml`，`quick_calc_test.py --data` 指向新目录重跑全量

### 来自 changes/2026-08-14-105342.md

# json-to-toml 转换器 + mowen.toml 从数据源重生成（50 技能）+ 引擎 f32 化

## 变更
- **新增 `server_tools/json-to-toml`**：从 JPCG 数据源技能 JSON 目录转换生成 `data/shuxing/*.toml`
  - 用法：`cargo run -p json-to-toml -- --xinfa <技能id> --json-dir <JSON根目录>`
  - 数值字段按数据源原值写入（引擎侧已按 `X_MULT` 归一化，不做除法）
  - `dot_interval`/`dot_duration`：数据源毫秒 → 浮点秒（`ms/1000`），支持小数（青莲剑 0.25s）
  - `dot_up`：优先读 `dot_param` 公式中 `j => i*0.12` 的 `dot_up` 系数（疏曲 0.12），回退 `dot_up` 字段
  - **不再输出 `jihuoqixue = "疏曲"`**（用户要求，跳数由引擎按 duration/interval 推导）
  - `design_effect`/`kind_type` 按 JSON 属性自动推断；`cast_mode`/`guaranteed_hit`/`effect_type` 用安全默认
  - `**x` 幂运算符支持（dot_param 公式 `631 * (j ** 0.99592)`）
- **`data/shuxing/mowen.toml` 重生成**：旧 34 技能 → 新 **50 技能**（含 DotOut 目录完整展开：
  宫/商/角/徵/青莲剑·徵 + 疏曲变体、相依/无尽藏/阳春雪/超然/裂涛等奇穴激活输出）
  - 关键值核对（数据源为准）：宫 atk_xishu 1.0325 → 2.609375（引擎 X_MULT 归一化后一致），
    商 2.4479167、疏曲 dot_up=0.12、青莲剑 dot_interval=0.25、max_tick tick→1 修复、`(tick-1)/(max_tick-1)` 全浮点
- **引擎 f32 化**（`crates/jpcg_core`）：
  - `skilltype.rs`：`SkillUp.RateUp` 变体字段 `f32`（`utils.rs` `ReadWidgetOutputData` 列默认值对应改为 `0.0`）
  - `crates/jpcg_api`：`BankNote` 透传 f32（`compound_use` 走 serde `as`）
  - `atkcal.rs` 金标准（`obj_shang_damage1/2` 等）对应更新，全部按确定性指数公式计算
  - 动机：`atk_xishu` 等按数据源原值写入后为小数（2.609375），u32 放不下；后果：面板/buff/属性含义不变
- **`server_tools/manifest-gen`**：`golden_tests` 误引用已删除的 `engine::datamake` → 改为 `engine::atkcal`；去掉多余 `toml` 依赖
- **overrides.json 说明**：`--json-dir` 数据源含心眼技能目录共 3 个（10447 主目录 + 10781/ 10782/ 覆盖目录），overrides.json 当前未启用（可后续补心眼处理规则）

## 起因
- `data/shuxing/*.toml` 此前为手工维护，与数据源脱节；用户提供数据源目录，要求按数据源重生成
- `dot_interval`/`dot_duration` 此前仅支持整数秒，无法表达 0.25s 小数值

## 验证
- `cargo build` 全 workspace 通过；`cargo test` 全过：
  - `jpcg_core` 103 测试（含 2 个新 dot 金标准：普通 6 跳相等、疏曲 9 跳等比）
  - `json-to-toml` 输出 50 条技能，与旧 mowen.toml diff 检出核心值一致
- 转换器重跑幂等性核对（两次生成 diff 为空；疏曲/青莲剑/相依/无尽藏输出均与预期一致）

## 遗留
- 心眼练技能（10781/10782 覆盖目录）转换规则未实现，overrides.json 待补
- dot_up=1 的 dot 条目（阳春雪/modaowan）也能被引擎正确处理，数据源如此即保留
- data/shuxing/mowen.toml 已替换为新生成版本，正式发布前需以新数据跑 `quick_calc_test.py` 全量快测

### 来自 changes/2026-08-14-113755.md

# 修复 dot 基础伤害（base_damage）缺失 + quick_calc 断言泛化

## 变更
- **转换器 bug 修复**（`server_tools/json-to-toml/src/main.rs` `collect_dots`）：
  - 根因：base 提取 `damages[0]` 时漏了 `.get("source_attribute")` 层级，`extract_base`
    永远返回 (0,0)，渲染端 `if base1!=0||base2!=0` 因此不输出 → 7 个 dot 全丢 base
  - 修复：与 `extract_skill` 对齐，`damages[0].source_attribute` → `extract_base`
  - 结果：商/角（含疏曲）=58/58；青莲剑·商/角=26/126；青莲剑·徵=26/26
- **`data/shuxing/mowen.toml`**：重生成，diff 确认仅新增 7 处 dot 的 `base_damage1/2`（其余无变化）
- **`quick_calc_test.py` 断言泛化**：
  - `check_dot` 不再硬编码"第1次=6跳/第2次=9跳"，改为从 toml 按文件顺序读每个 dot 的
    `dot_duration`/`dot_interval`/`dot_up` 推导：跳数 `n=round(duration/interval)`
    （支持浮点秒，青莲剑·徵 2/0.25=8 跳、角 4/1=4 跳、商 1.5/1.5=1 跳），
    `dot_up>0` 等比 ×(1+up)^k 否则每跳相等；q=Σjumps；toml 条目必须全部被 results 覆盖
  - 无质断言：删除硬编码 `WUZHI_NAMES=("相依1","相依2")`，改为从 toml 自动收集
    `has_critical_strike=true` 技能（相依(lv3)/相依(lv4)），并校验其全部出现在结果中
  - 新增 `load_data()`：统一读取 mowen.toml（缺 tomllib/找不到时跳过数据依赖断言）

## 起因
- 用户实测发现 dot 基础伤害（base_atk）丢失：重生成的数据里 `base_damage1/2` 字段缺失，
  旧版手写数据有（商/角 = 58）
- quick_calc 旧断言假设所有 dot 仅 6/9 跳两种形态，与数据源青莲剑系（8/4/1 跳）不符

## 验证
- `cargo test --workspace` 全过（15 测试套件，含 dot 金标准 6/9 跳，金标准不读数据文件不受影响）
- `cargo clippy -p json-to-toml` 零警告
- `python3 examples/python_demo/quick_calc_test.py` → ALL PASS（EXIT=0）：
  - dot 跳数：商/角 6 跳相等、疏曲 9 跳等比、青莲剑 8/4/1 跳，全部按 toml 推导断言通过
  - 无质：相依(lv3)/(lv4) Q 固定=期望公式
  - 面板对照：化劲 76.32% / 御劲 2.55% / 外防 10.75% / 内防 14.51%
- 转换器重跑幂等：两次生成产物 diff 为空

## 遗留
- 金标准 dot 期望值基于旧 base（58），与新数据一致无回归；如后续校准以 `--out` 输出回填

### 来自 changes/2026-08-14-133145.md

# 全职业 toml 导出（27 个伤害心法）+ 全量加载验证

## 变更
- **全职业导出**：`data/shuxing/` 新增 26 个心法文件（10002/10003/10014/10015/10021/10026/
  10062/10081/10144/10175/10224/10225/10242/10243/10268/10389/10390/10464/10533/10585/
  10615/10627/10698/10756/10821/10786），连同 mowen.toml = 27 个伤害心法全覆盖
  - **overrides.json 全量生成**：`xinfa_name`/`xinfa_nom` 从 belongs.json 自动推断
    （`*_to_*_attack_power` 前缀映射：spirit→根骨、spunk→元气、strength→力道、agility→身法、
    vitality→体质），覆盖全部 27 池
  - 文件名沿用历史命名：10447→mowen.toml、10786→zhoutian.toml（保留周天功手写校准值
    atk_up=1.96/pofang_up=0.3）；其余按池 id 命名
  - 治疗心法（10028 离经/10080 云裳/10176 补天/10448 相知/10626 灵素）与通用池 0 在数据源中
    无伤害技能条目，不导出
- **datamake.md**：补充全职业导出命令、overrides.json 字段语义

## 决策
- `atk_up`/`pofang_up`：数据源无此数据（belongs.json 只有属性转化，无攻击/破防倍率）。
  莫问（1.96/2.0）、周天功（1.96/0.3）保留手写校准值，其余职业暂用 1.0 占位，待逐职业校准
- `wuzhi`（无质名单）：无数据支撑，仅莫问（相依）保留手写，其余职业空
- 各职业跳过技能（无伤害数据）少量且为真实无伤害条目（御鸿于天/地坼/幽都判/织翠 等），符合预期

## 验证
- FFI 全量加载：27 个职业逐一 `calculate` 成功，技能数与转换器输出一致
- `cargo test --workspace` 全过（16 金标准 + dot 金标准不受数据文件影响）
- quick_calc_test.py（莫问）ALL PASS

## 遗留
- 其余 25 职业 atk_up/pofang_up/wuzhi 为占位值，需在游戏中实测属性换算后校准
- 纯 base 伤害技能（无 atk_xishu）数量较多（如笑尘诀 33 条），为数据源原始值

### 来自 changes/2026-08-14-144008.md

# 修复连招击杀概率恒为 0（target_hp 单位 ×10000 残留）

## 变更
- **修复根因**：`engine/kill_prob.rs` 中 `target_hp` 仍按旧单位乘以 10000（
  `hostilepile.target_hp as f64 * 10000.0`），而前端已按"个位"输入血量（如 2915274）。
  旧逻辑把血量放大成 291.5 亿，累计期望伤害永远打不死 → 击杀概率恒为 0
- **修复**：`target_hp` 直接使用 `hostilepile.target_hp as f64`，与血量输入单位对齐
- **确定性分支**：当 `cum_std == 0`（会心率 0% 或全无质连招）时，期望伤害 ≥ 血量即
  必杀（kill_prob = 1.0），否则 0.0，避免纯确定性连招误走正态分支
- **无质技能**：`has_critical_strike`（无质）技能伤害固定为期望 Q，方差为 0
- **DTO 透传**：`ComboStepResult` 输出 `has_critical_strike`/`zhenshishanghai` 字段，
  conv.rs 映射到 `ComboStepResultDTO`

## 验证
- FFI 复现用户场景（20×风来吴山、血量 2915274、会心 48.6%）：击杀概率从第 5 步开始
  爬升（0.02%），第 9 步 89.84%，第 13 步起 100%，final = 99.99999999999998%
  （修复前全部恒 0）
- `cargo test --workspace` 全过（16 金标准 + dot 金标准不受影响）

## 遗留
- 用户侧需更新 core 二进制：dynamic 模式替换 `modules/` 或 exe 同目录 dll；
  static 模式需重新编译 App（`make build-app-dynamic`）

### 来自 changes/2026-08-14-150000.md

# 技能形态全量输出（放开连续 lv 压缩）+ 技能池搜索/折叠

## 变更
### 数据层（server_tools/json-to-toml）
- **放开 `is_series()` 压缩**：所有技能的全部 lv 形态（等级/层数/点数/距离/目标数等）
  全部输出，不再只取最大 lv。全库 990 → 1627 条目
  - 段氏（zhoutian.toml）：引窍 101 形态（0~100 点任脉）、破 51 形态（0~50 点能量）、
    绝脉 15 形态（1~15 层）——段氏靠回能技能回能、引窍打出伤害，需逐形态可选
  - 其他门派同类：布泽/叠刃/悬象/流血/入月/鬼宿/破绽/缠绞/勾线(40级)/展缓(99级) 等
- **显示名语义化**：同名多形态优先用数据源 comment 生成 `名·形态` 后缀
  （如 `引窍·50点任脉`、`相依·1段`），无 comment 退回 `名(lv{N})`
- `is_series()` 函数删除

### 后端（jpcg_api / jpcg_core）
- `SkillPoolItemDTO` 新增 `sub_id` 字段（同 skill_id 不同形态区分），
  `host/skill.rs` load_skill_pool 与 `host/calc.rs` skill_dto_to_skilltype 透传
- 预设 `ComboStep` 新增 `sub_id`（serde default，旧存档兼容），
  conv.rs 预设往返保留 sub_id

### 前端（examples/jpcg_app）
- `ComboPage` 技能池：
  - **搜索框**：按技能名实时过滤（如搜"0点任脉"定位形态）
  - **折叠分组**：按基础技能名（去掉 `·形态` 后缀）分组，组头显示形态数，
    可展开/折叠，支持全部展开/折叠
  - **收藏/React key 唯一化**：改用 `skill_id-sub_id` 复合 key，
    修复同 id 多形态（相依 lv3/lv4）收藏与渲染混叠
  - 旧收藏 localStorage 数据兼容转换
- types/index.ts 补 `sub_id`；commands.ts mock 补形态示例（引窍 0/50/100 点任脉等）

## 验证
- 27 池重生成：zhoutian 196 条（引窍 101/绝脉 15/破 51 形态命名正确）、
  10821 勾线 100 条、10175 展缓 99 条、dot 技能保留
- FFI：load_skill_pool 返回 sub_id + 各形态 atk_xishu 正确
- FFI 连招回归：引窍·0点任脉 q=46.5万 vs 引窍·100点任脉 q=232.7万，
  final_kill_prob 计算正常
- `cargo test --workspace` 全过（16 金标准 + 各 crate）
- 前端 `npx tsc --noEmit` + `npm run build` 通过

## 遗留
- 骤风令：已解决（数据源更新推入，见 changes/2026-08-14-163023.md）
- `雾刃飞光·雾/刃/飞/光`、`劈风令·骤` 等名称本身带 `·` 的技能会被归入基础名组
  （如"雾刃飞光"组），语义上属于关联技能折叠，可接受

### 来自 changes/2026-08-14-163023.md

# 2026-08-14 163023 — 数据源更新（骤风令）+ 无质批量标注 + 追加真伤（已损失生命值）

## 背景

IcyTide/Generator 推送更新：段氏「骤风令」本体进入 skills.json（原缺失，仅 37779/37804
名为「横驱风靡」的旧条目），并**成批给无质技能打上 `critical_strike/critical_power = "0"`
占位标记**（此前仅剑飞惊天 1 个）。数据源结构同步变化：critical 从 node 层移入
damages 数组元素内（node 层不再有 critical 对象）。

## 变更

### 数据源脚本与文档
- 新增 `server_tools/json-to-toml/fetch-json.sh`：curl 拉取 IcyTide/Generator
  `assets/json/{skills,dots,belongs}.json` → `data/raw-src/`（已加入 .gitignore，原始
  JSON 不入库），末尾打印转换命令提示
- `data/shuxing/datamake.md`：新增「数据源获取」小节；转换器规则更新（新结构/
  无质 0/O 判定/追加真伤/rename）

### 转换器（json-to-toml）
- **适配新结构**：critical_strike/critical_power 从 damages[0] 提取（兼容旧 node.critical）
- **无质 0/O 规则**：cs/cp 任一为 `"0"/"O"` 占位 → `has_critical_strike = true`
  （数据源官方标注，约 19 技能/65 形态：果报/众境/剑飞惊天/临源/令怖/圣裁/崇光斩恶/
  涤罪/纵遇善缘/断马摧城/相依/不愧君/斩长鲸/镇星入舆/领胡 等）；
  overrides `wuzhi` 名单保留兜底（涤罪/临源 补入名单）
- **追加真伤形态展开**：overrides 新增 `lost_hp_zhenshi: [{name, per_layer, max_layers}]`，
  命中技能展开为 破绽0..=N 层形态并写 `lost_hp_zhenshishanghai` 字段
  （怒锋倾涛：单持/双持 × 破绽0~3层 = 8 形态，每层 6% 已损失生命值）
- **rename 能力**：overrides 新增 `rename` 映射，数据源名 → 显示名
  （10786「骤风」→「骤风令」，对齐 belongs.json 名）
- **sid 级 name 计数**：同名跨 sub（单持/双持等多 sub 同 sid）也能正确加 comment 后缀

### 引擎追加真伤（怒锋倾涛破绽机制）
- `Skilltype.lost_hp_zhenshishanghai: f32`（追加真伤 = 目标已损失生命值 × 系数，无视防御）
- `kill_prob.rs` 动态结算：`追加 = max(target_hp - 前序累计期望 - 本步主Q, 0) × 系数`
  ——**确定性伤害只加期望不加方差**（与无质一致），连招中随血量损耗几何收敛
  （追加本身也计入已损失），击杀 CDF 框架不变
- `ComboStepResult.lost_hp_zhenshi_damage: f64` 暴露每步追加值
- 单技能面板（q_cal）不含追加（满血目标追加为 0，正确语义）
- DTO 全链路透传：jpcg_api（SkillPoolItemDTO/SkillResultDTO/ComboStepResultDTO/
  SkillEditorItemDTO）→ host conv/skill/calc → 前端 types/index.ts +
  SkillEditorPage 新增「追加真伤(已损失×系数)」输入框

### 测试
- 新增金标准 `combo_lost_hp_zhenshi_dynamic`：首击追加 = 已损失(主Q)×系数、
  追加几何收敛、确定性方差 0、8 连击击穿目标血且不超杀
- `cargo test --workspace` 全过（17+1）；前端 tsc + build 通过

## 数值验证（FFI，怒锋倾涛·单持·破绽3层，目标 200 万血）

```
Q=152535 追加=332544  累计=485079   击杀率 0.0%
Q=152535 追加=245230  累计=882843   击杀率 0.0%
Q=152535 追加=173632  累计=1209010  击杀率 0.0%
Q=152535 追加=114922  累计=1476467  击杀率 0.0%
Q=152535 追加=66780   累计=1695782  击杀率 0.1%
Q=152535 追加=27303   累计=1875620  击杀率 12.2%
Q=152535 追加=0       累计=2028155  击杀率 59.7%
Q=152535 追加=0       累计=2180690  击杀率 92.9%
```

## 决策记录
- 追加结算采用「主伤害扣血后的一次性已损失」（不迭代）：`(target_hp - 前序 - 主Q) × 系数`
- 无质判定：数据源 0/O 占位为主，overrides 名单兜底（不相依依赖代码层 0/O 猜测）
- 装备特效池（池 0）的 custom_damage_base 真伤 23 条不并入职业池（维持现状）

## 遗留
- 其余职业 atk_up/pofang_up 仍为 1.0 占位（需逐职业实测校准）
- 池 0 装备特效（巽/御·击破/刃凌等真伤）未纳入职业技能池

### 来自 changes/2026-08-14-205054.md

# 2026-08-14 205054 — 恢复被误删的周天功/莫问数据文件

## 背景

数据迁移（编号 TOML 化）后 `data/shuxing/zhoutian.toml` 与 `data/shuxing/mowen.toml`
从工作区被删除（git 状态 D），但这两个池无编号文件替代，导致周天功/莫问数据缺失。

## 原因

- 周天功 = 数据源池 **10786**，莫问 = 池 **10447**，但 `overrides.json` 为二者显式配置
  `file: "zhoutian.toml"` / `"mowen.toml"`（沿用历史名，datamake.md 已有说明），
  转换器 `file_name = ov.file 或默认 <池id>.toml`，因此它们不会生成编号文件。
- 最后一次转换运行未覆盖这两个池（或生成后被清理）。

## 变更

- 重跑转换器恢复两个文件：
  ```sh
  cargo run -p json-to-toml -- --skills data/raw-src/skills.json \
      --dots data/raw-src/dots.json \
      --overrides server_tools/json-to-toml/overrides.json \
      --out data/shuxing --xinfa 10786 --xinfa 10447
  ```
  - `zhoutian.toml`：周天功/元气（atk_up 1.96 / pofang_up 0.3），196 条技能
    （含引窍·0~100点任脉 全形态、骤风→骤风令 rename）
  - `mowen.toml`：莫问/根骨（atk_up 1.96 / pofang_up 2.0），61 条技能
    （相依·1段/2段 等标无质）

## 验证

- `tomllib` 解析 OK；`cargo test -p jpcg_core` 20 项全过
- FFI `list_professions` 27 个心法含 `zhoutian`(周天功) + `mowen`(莫问)
- Python demo（JPCG_DATA_DIR=./data）端到端计算通过

## 留待处理

- 其余缺失池（10028 离经易道 / 10080 云裳心经 / 10176 补天诀 / 10448 相知 /
  10626 灵素）为治疗心法，按 datamake.md 约定跳过，无需生成
- 池 0（婆罗门/装备特效）按既有决策不并入职业技能池

### 来自 changes/2026-08-15-143820.md

# 修复武器伤害双算：AtkConfig::total() 只读基础攻击（分山劲断马摧城实测校准）

## 现象
分山劲断马摧城（无质）实测 536939，计算结果 61 万，偏高约 13.6%。

## 根因
`b_cal()` 使用 `AtkConfig::total()` = 基础攻击 **+ 武器伤害**，
`i_cal()` 又叠加 `武器×watk_xishu/100` 路径：

```
x = base_atk + (基础攻击+武器)×atk_xishu + 武器×watk_xishu/100
```

武器伤害被乘上技能攻击系数重复计入（断马：12062×16.78125 ≈ 20.2 万 I 段 → Q 多算约 7 万）。
数据源公式武器伤害仅经 `watk_cof` 参与（watk_xishu=0 的技能完全不参与）。

复现验证：
- 修复前：断马摧城 Q = 609776（与用户报告 61 万一致）
- 修复后：Q = 540119（去掉武器双算后）；实测 536939 处于 base 120~126 随机浮动区间内（±2.4%），判定为同一次样本的正常浮动，不再校准

## 改动
- `crates/jpcg_core/src/type_set/player.rs`：`AtkConfig::total()` 只返回 `base`（不含武器伤害），注释说明武器单独经 watk_xishu 参与
- `server_tools/json-to-toml/overrides.json`：池 10390（分山劲）`atk_up` 1.0 → 1.88
- `data/shuxing/10390.toml`：[xinfa] `atk_up = 1.88`（与 overrides 一致）

## 验证
- 引擎（用户面板：基础属性 18790/基础攻击 58197/会心 53354/会效 0/破防 134470/武器 12062；目标外防 15071/御劲 5047/化劲 62113）：
  N=458271, H=793247, Q=540119
- `cargo test --workspace` 全部通过（core 20 项 + app 1 项），无回归
- `total()` 唯一调用点即 `b_cal()`；derivatives 武器导数本就只走 watk_xishu 路径，无需联动

## 遗留
- 其余 24 个池的 `atk_up` 目前均为 1.0 占位（仅 10390=1.88/10447=1.96/10786=1.96 经确认），待逐一实物核对后更新 overrides 并重跑转换器

### 来自 changes/2026-08-15-145232.md

# 内防修复后回填 atkcal 金标准期望值

## What

`engine/atkcal.rs` 的用例在上一轮修复（内防门派改用 `guo_nfangyu_with`）后，金标准（golden）测试仍持有旧期望数值。本轮:

1. 统一 golden 用例的 skill 为 `&Skilltype`。
2. 用临时 `golden_dump_tmp` 测试 dump 出新数值，回填 6 组 golden 用例 + `combo_wuzhi` 用例的 `y/b/i/n/h/q` 与全部 6 项导数；dump 测试跑完即删。
3. `cargo fmt -p jpcg_core` 格式化受影响代码块。

## Why

Q 段数值依赖内防/外防分支，修复后各用例期望值整体下移（如 gong_default q=95725→91768、y=1355→1299）。不更新则 golden 全红且无法发现后续回归。

## Validation

- `cargo test --workspace` 全绿（atkcal 12 个 golden/combo 用例通过）。
- `cargo fmt -p jpcg_core --check` 通过。

## Notes

- 数值回填采用「dump→回填」而非手工推算，与「金标准不改行为」原则一致。
- 导数与 buff 有无无关（与修复前一致，非回归）。

### 来自 changes/2026-08-15-150000.md

# 版本号 bump：2.1.0-alpha.1 → 2.1.0-alpha.2

## What

- 根 workspace 版本（core 版本源）升至 `2.1.0-alpha.2`。
- `scripts/sync-version.sh` 同步 package.json / tauri.conf.json / 前端模拟版本串。

## Why

随 PR #6 累积修复（武器双算 + 内防分支 + 金标准回填）后发版前的最小版本步进。

### 来自 changes/2026-08-15-212429.md

# 创建 jpcg_combo 连招引擎 crate（排轴器后端重建，MC 击杀率 + hp 追加真伤）

日期：2026-08-15

## 背景
排轴器（ComboPage）伤害偏小 + dot 技能只算单跳 + 击杀率正态近似不可靠。此前讨论决定：
新建 `jpcg_combo` crate 承接连招编排，单技能公式/追加真伤公式留在 core（单源），
击杀率改用蒙特卡洛（50k 采样），hp 追加真伤按路径实际血量结算。

## 变更
### 新 crate `crates/jpcg_combo/`（cdylib + rlib）
- `engine.rs`：双通道模型
  - 期望通道：g/h/q/dot 期望 + 累计期望/方差（DTO 语义与迁移前一致）；追加真伤只加期望
  - 蒙特卡洛通道：`samples=50_000`（`DEFAULT_SAMPLES`，`ComboConfig` 可注入种子/采样数），
    每条路径实时血量推进，dot **逐跳独立**会心判定，击杀率 = 击杀路径占比，确定性无质连招精确
- `host.rs`：DTO 层计算入口 + 预设 CRUD + 导出/导入（存储委托 `jpcg_core::combo_io`/`config_io`）
- `conv.rs`：`skill_dto_to_skilltype` 等转换；ComboStep/ComboPreset 的 From 因孤儿规则留在 core
- `ffi.rs`：句柄 + JSON 协议（`jpcg_combo_call` 等，ABI_VERSION=1，与 core 一致）
- 测试：无质零方差、固定种子可复现、MC 击杀行为、真伤递增、dot 逐跳独立方差

### jpcg_core 收敛（删除）
- `engine/kill_prob.rs`、`host/combo.rs`、`engine/mod.rs`/`host/mod.rs` 的 mod 声明
- `ffi.rs` 7 个 combo 方法分支、`lib.rs` `calculate::start_combo` facade
- `atkcal.rs` golden 的两个 combo 测试迁至 jpcg_combo（新语义重写）

### 追加真伤语义（修复旧 bug）
- 旧：`lost=(target-(cum+mean)).max(0)` 把剩余血量当已损失 → 击杀率≠0 时真伤错误（用户报告的根源）
- 新（语义 A，core `lost_hp_append` 实现）：`lost = (max_hp - 结算后剩余).max(0) × 系数`，
  真伤同样扣血 → 期望通道递增（斩杀机制），MC 逐路径实时结算

### 修复会心率双计 bug
- 旧 kill_prob `crit_rate = guo_huixin() + huixin_up/100 + buff.huixin_pct/100`（重复计入 buff）
- 新：与 q_cal 一致 `guo_huixin() + huixin_up/100`

### DTO dot 字段链路（dot 只算单跳的根因）
- `SkillPoolItemDTO` 补 `dot_interval/dot_duration/dot_up/wushijianshang/zhenshishanghai`
- `load_skill_pool`、`skilltype_to_pool_item`、combo `skill_dto_to_skilltype` 全链路映射

### Tauri 双模式
- static 直调 `jpcg_combo::host`；dynamic 经 `ffi_bridge` 新增 `COMBO` OnceLock 加载
  `libjpcg_combo`（`call_combo*` 系列）；`get_module_versions` 增加 `combo` 字段

### 前端
- hostile 表单加「目标最大血量/当前血量」（0 = 未提供，回退 target_hp 满血模型）
- `HostileConfigDTO`/`ModuleVersions` 类型、`HOSTILE_FIELDS`、normalize 同步

### Makefile
- `build-modules`/`modules-dir` 纳入 jpcg_combo（dll 不进 modules_manifest 更新体系）

## 决策记录
- combo 依赖 jpcg_core（无环依赖）；损失公式唯一实现在 core（combo 经 `lost_hp_append` 调用）
- 无 hp 输入时（target_hp 也为 0）击杀率恒 1（迁移前行为）
- dot 的技能级 g/h 按 `dot_jumps` 等比因子拆分为逐跳 g/h（与期望通道同源），逐跳独立采样
- jpcg_combo 版本继承 workspace（2.1.0-alpha.2），不进 modules_manifest

## 验证
- `cargo test --workspace` 全绿；jpcg_combo 11 测试；dynamic e2e 冒烟绿
- `cargo build`（static+dynamic 双模式）、`cargo fmt --all --check`、前端 `npm run build` 绿

### 来自 changes/2026-08-18-140250.md

# 修复 macOS 技能编辑器底部裁剪

## 变更

- 让技能编辑页作为应用主区域的 flex 子项占用顶部工具栏之外的剩余高度。
- 允许右侧技能详情面板在受限高度内收缩，并由自身滚动显示完整编辑表单。

## 原因

技能编辑页原先使用 `height: 100%`，但它位于带顶部工具栏的纵向 flex 容器中，实际高度会叠加在工具栏之后。主容器的 `overflow: hidden` 会裁掉详情面板底部，导致 macOS 窗口中最后的字段无法滚动到可见区域。

## 决策

使用 `flex: 1` 与 `min-height: 0` 修正容器约束，保留现有详情面板内部滚动和页面布局，不改变字段、数据协议或保存逻辑。

### 来自 changes/2026-08-30-173635.md

# 修复全 workspace clippy 警告 + README 编译步骤

日期：2026-08-30

## 背景
dev 合并前需通过 CI（clippy 虽不 -D warnings，但要求清零）。同时更新 README 增加编译/构建步骤。

## 变更
### README
- 版本头更新 v2.1.0-alpha.2（2026年8月）
- 击杀概率曲线：CLT 正态 → 蒙特卡洛 50k 路径（示意代码）
- 目标设定输入项补「目标最大/当前血量」
- 主题描述修正为薄荷/松石绿；Roadmap 标记 AutoDiff 已实现
- 新增「🔧 编译与运行」章节（快速开始 / Tauri / 双构建模式 / 打包 / check-all / workspace 结构）

### clippy 清零（19 文件，~50 警告）
- `needless_borrow`×11：atkcal `&self.coeff` → `self.coeff`
- `collapsible_if`×多处：合并嵌套 if / `&& let`
- `missing_safety_doc`×9：core/update/const 的 FFI 函数补 `# Safety`
- 删除死代码 `default()`×3：player/hostilepile/xinfa（无调用方，与 Default trait 冲突）
- `manual_is_multiple_of`：`y.is_multiple_of(...)`
- `excessive_precision`×2：golden 字面量截断 `1.776_041_6`（f32 等值，行为不变）
- `ptr_arg`：`is_string_empty(&String)` 加 `#[allow]`（serde skip_serializing_if 需该签名）
- `map_or`→`is_some_and`、`for_kv_map`→`keys()`、`let_unit_value`、`match_single_binding`、`bind_instead_of_map`、`match_result_ok`
- `too_many_arguments`（download.rs 公开 API）、`type_complexity`（derivatives）→ 加 `#[allow]`/type 别名

## 决策
- `default()` 死代码直接删除（经全 workspace 搜索确认无调用方），非加 allow
- `is_string_empty` 因 serde 属性签名需 `&String`，用 `#[allow(clippy::ptr_arg)]` 保留

## 验证
- `cargo clippy --workspace --all-targets` 0 警告
- `cargo test --workspace` 全绿（金标准 15 + combo 11 不因字面量截断变值）
- `cargo fmt --all --check`、`make check-all`、前端 `npm run build` 全绿

### 来自 changes/2026-08-30-230349.md

# 等级常数编译期 const 模块（jpcg_const::level_constant）取代 CoefficientConfig 默认并补御劲

日期：2026-09-04（收尾）

## 背景与决策
- 目标模块定位：**常数计算模块，编译时由预设 TOML 解析并固化为 `const` 全局**，
  为后续"等级 → 常数"关系逆向（靠 level 推出所有常数）铺路。
- 弃用先前 WIP 的"运行时经 `data_dir()` 读 `normal_num` TOML"方案（引入 core 反向依赖 + 运行时 I/O）。
- 改为：`include_str!` 内嵌预设 + `const fn` 白名单解析，运行时零依赖、零 I/O；
  `level` 只作快照记录（`LEVEL`，预计算用），不进常量结构。
- 引擎 `CoefficientConfig` 仅保留"可配置载体"角色：字段集合补上缺失的御劲分母
  （原 `guo_yujin_huixiao_with` 错误复用 `huixin_xishu`，御劲会伤减免分母 55123.2 从未进入配置），
  默认值单一来源 = 本 const 模块；固定公式方法统一委托 `*_with(CoefficientConfig::default())`，
  引擎各处硬编码分母（126007.2/30115.8/55123.2/197703/72844.2/225957.6）全部移除。

## 变更
### jpcg_const（叶子 crate，运行时依赖清零）
- `preset/level_constant.toml`（新）：130 级·第 3 赛季快照
  （pofang/huixin/huixiao/yujin/yuhui/huajin/fangyu 分母 + pvp_global_jianshang）
- `src/level_constant.rs`（新）：`LevelConstant` 结构 + `LEVEL`/`CURRENT` const；
  `const fn parse_snapshot` 逐行白名单解析——未知 key/缺字段/坏值 → 编译期报错（防漂移）；
  f32 十进制用"整数尾数÷10^k"解析，与 Rust 字面量 bit 完全一致（有单测锁定）；
  const fn 受限说明：不支持切片，全程以 (bytes,start,end) 偏移扫描。
  测试 4 个：bit 一致 / 全量 roundtrip / 未知 key panic / 缺字段 panic。
- `lib.rs`：模块改挂 `level_constant`；删除 `player.rs`（运行时读取器，整文件删除）
- `Cargo.toml`：移除 jpcg_core/serde/toml/tempfile 全部依赖（含 dev-deps）

### jpcg_api
- `CoefficientConfigDTO` 新增 `yujin_xishu` / `yuhui_xishu`（`#[serde(default)]`，旧存档兼容）

### jpcg_core
- `Cargo.toml` 新增 `jpcg_const` 依赖；`lib.rs` 还原 `pub mod store` → `mod store`
- `type_set/coefficient.rs`：字段 +御劲两项；`Default` 取自 `jpcg_const::level_constant::CURRENT`；
  新增 `From<&CoefficientConfigDTO>`（分母 0/缺失 → 真源默认；pvp 减伤 0 语义合法不回退）
- `type_set/player.rs` / `hostilepile.rs`：固定公式改为委托 `*_with(&CoefficientConfig::default())`；
  `guo_yujin_huixiao_with` 改用 `yuhui_xishu`（55123.2，修复错接）、`guo_yujin_huixin_with` 改用 `yujin_xishu`
- `host/calc.rs`、`host/config.rs`：DTO→core 走 `CoefficientConfig::from`（旧档/缺字段安全回退）

### jpcg_combo
- `host.rs` conv 改用 `CoefficientConfig::from`；测试系数字面量补齐新字段
- `engine.rs` 无质期望回填：御劲会伤减免修正后 91768→90651、183536→181302（hostile yujin=5047）

### core 金标准
- `atkcal.rs` golden 7 组（gong/wei/zheng × default/buff + gong_wuzhi）H/Q/导数按新引擎回填
  （变化方向 = 御劲会伤减免增强，仅 yujin>0 目标生效）

### 前端（examples/jpcg_app）
- `types/index.ts`、`utils/constants.ts`（COEFFICIENT_FIELDS + DEFAULT_COEFFICIENT）、
  `utils/normalize.ts` 同步新增字段；其余经 `...DEFAULT_COEFFICIENT` 展开自动携带

## 验证
- `cargo fmt --all -- --check` 干净
- `cargo clippy -p jpcg_const -p jpcg_api -p jpcg_core -p jpcg_combo --all-targets` 零警告
- `cargo test -p jpcg_const`（4）/ `-p jpcg_core` / `-p jpcg_combo` 全部通过
- 前端 `tsc --noEmit` 通过
- python demo / FFI JSON 无需改动：缺字段零值由 `From` 回退真源默认

## 注意/待办
- 逆向进展后可扩展：预设增多级快照 → `[LevelConstant; N]` 表；关系已知后替换为
  `const fn from_level(level) -> LevelConstant` 推导（预设退化为系数）。
- `yujin_xishu` 当前与 `huixin_xishu` 同值(197703)，是否同一游戏常量待逆向确认；
  已拆独立字段避免届时改 schema。
- core 现编译期依赖 jpcg_const：动态模块更新语义下 core dll 会内嵌 const 数据副本，
  若未来要求"只更 const dll 即更新引擎默认系数"需另做运行时取数设计（当前阶段无此需求）。

### 来自 changes/2026-09-04-200540.md

# 修复 release.sh：独立版本降级中断 + prep 分支命名冲突

日期：2026-09-04

## 背景
首次执行 `scripts/release.sh beta`（2.1.0-beta.1）暴露两处缺陷：

1. `cargo set-version "$VERSION"` 会把 workspace 内**所有**成员都尝试改成新版本，
   但 `jpcg_const` 使用独立版本号（`130.3.20260602`，等级.赛季.日期），
   直接被判定 "Cannot downgrade from 130.3.20260602 to 2.1.0-beta.1" 中断脚本。
2. prep 分支名 `release/prep-${VERSION}` 与已有分支 `release` 冲突
   （GitHub 禁止 `release/xxx` 与分支 `release` 共存），push 被拒。

## 变更
- `scripts/release.sh`：
  - 版本 bump 改为**仅改根 `[workspace.package]` version**（继承成员自动生效，
    独立版本 crate 不动），随后 `sync-version.sh` 同步前端/tauri/模拟串，
    再 `cargo metadata` 刷新 Cargo.lock
  - prep 分支改为 `prep/${VERSION}`，避开 `release/` 命名空间
- `scripts/README.md`：同步两处描述

## 验证
- beta.1 实际发布时手工等价步骤跑通（根版本改 + sync-version + lock 刷新 + prep/v2.1.0-beta.1 + tag）
- 本修复让 `release.sh beta` 可全自动复现

## 注意
- 更新器 `jpcg_updater` 同样为独立版本（2.1.0），不受 workspace bump 影响

### 来自 changes/2026-09-04-210000.md

# 修复 release.yml：Windows runner 两步缺 shell: bash

日期：2026-09-04

## 背景
`v2.1.0-beta.1` 触发 release.yml 时 windows-latest job 在"判断更新通道"步骤失败：
该步（及"提取版本号"）未指定 `shell: bash`，Windows runner 默认用 PowerShell，
`[[ "$GITHUB_REF_NAME" == *-beta* ]]` 为 bash 语法 → 报错，导致三平台只产出 darwin/linux 资产。

## 变更
- `.github/workflows/release.yml`：为"提取版本号（tag）"与"判断更新通道"两步显式加 `shell: bash`

## 验证
- 失败 run 335...（33870657978）三 job 结论：darwin/linux success、windows 唯一失败步 = 判断更新通道
- 修复后重建 tag 重跑，windows 应产出 app exe / dll / 清单

### 来自 changes/2026-09-04-220000.md

# feat: 以 GitHub Release 为载具的更新自动部署（deploy-gen + package job + 模块平台过滤）

日期：2026-09-04

## 背景
beta.1（v2.1.0-beta.1）发布与服务器落盘全程手工。目标：tag 构建成功后自动产出
服务器可直接应用的"通道 tgz"，随 Release 推送；服务器侧以 webhook 拉取落位
（服务器接收端为独立私有仓库实现，不随本仓库发布）。

## 变更（本仓库 jpcg）
- 新增 `server_tools/deploy-gen`（workspace 成员）：
  - 输入：GitHub Release 资产目录 + 仓库 data + version/channel
  - 输出：更新服务器 downloads 根布局 stage（beta 根式 / stable 版本目录式）
  - 生成 update.toml / manifest.toml（app 二进制 sha256）/ 三平台合并
    modules_manifest.toml（platform="multi"，core/update 版本随 app、
    const 反查自身 Cargo.toml 独立版本）/ data 布局（按 data_manifest 列出的文件拷贝）
  - beta 与 stable 双通道本地试跑通过，与 beta.1 手工部署布局一致
- `.github/workflows/release.yml`：`deploy`(SSH 直推) 方案废弃，改为新增
  `package` job（needs: build，仅 tag push）：下载资产 → deploy-gen → tar 为
  `jpcg-<channel>-<tag>.tgz` 并附 `.sha256` → `gh release upload` 到同一 Release
- `crates/jpcg_update/src/modules.rs`：模块更新按本机平台扩展名过滤
  （mac=.dylib / linux=.so / windows=.dll），适配多平台合并清单，避免下载无关 dll

## 验证
- `cargo build -p deploy-gen` / beta、stable 双通道试跑通过
- `cargo check -p jpcg_update` 通过

## 待办（服务器接收端，独立私有仓库，不在本仓库实现）
- 服务器 webhook 接收（验签）→ 拉取该 tgz → 校验 .sha256 →
  原子切换（beta 覆写根 / stable 合并保留 3 版）
- 下载服务目录列表处理（stable 客户端依赖 /updates/JPCG/ 的 HTML 目录列表）

### 来自 changes/2026-09-08-214000.md

# fix: 更新检查/执行无限等待 + Tauri v2 中 window.confirm 失效导致无法确认更新

日期：2026-09-08

## 背景
- beta.1 构建中点击"检查更新"卡在"正在检查更新..."不返回（Beta 开关开/关都卡）。
  服务器侧所有被请求 URL 均 200，定位为客户端代码层。
- 修复后 beta 仍"点了无反应"：Tauri v2 的 WebView **禁用 window.confirm**（恒返 false），
  检测到新版本时的确认框被当"取消"处理 → 界面直接复位、既不更新也不提示。

## 变更
- `crates/jpcg_update/src/download.rs`：新增 `http_client()`（统一 60s 超时），
  全部 5 处 `reqwest::Client::new()` 改用之
- `crates/jpcg_update/src/modules.rs`：模块清单请求同样走带超时客户端
- `examples/jpcg_app/src-tauri/src/commands/update.rs`：`check_update` / `perform_update` /
  `perform_app_update` / `perform_modules_update` 同步实现移入
  `tauri::async_runtime::spawn_blocking`，避免在 async 命令线程内嵌套新建 tokio runtime
- `examples/jpcg_app/src/components/ConfigPanel.tsx`：废弃 `window.confirm`，改为应用内
  确认区（发现应用/模块更新时显示"下载更新/取消"按钮），确认后走原下载→验证→重启流程。
  补充状态修正：命中确认时 `updating` 保持 true（禁用重复触发）、进度框与确认框互斥避免
  叠显；取消有 toast 反馈；确保点"下载更新"后进度条可见（否则又像"无反应"）

## 验证
- `cargo check -p jpcg_update` / `-p jpcg_app` 通过；前端 `tsc --noEmit` 通过
- 效果：任何网络卡点 60s 内转为明确报错；命令不再无界等待；更新确认不再被静默吞掉

### 来自 changes/2026-09-08-223000.md

# fix: macOS 产物架构错标 + 全面切到 Apple Silicon（aarch64）单架构

日期：2026-09-08

## 背景
- GitHub 标准 macOS runner 已全部迁移到 arm64（macOS 26，Intel 已被官方淘汰），
  `macos-latest` 上 `cargo build` 产出的就是原生 aarch64 二进制，但此前产物名与清单
  一直写死 `jpcg-app-darwin-x86_64` / `arch = "x86_64"`——**错标**。
- 更隐蔽的是 manifest `os` 字段写的是 `"darwin"`，而客户端按
  `std::env::consts::OS` 比对（= `"macos"`），导致 macOS 客户端永远匹配不到二进制，
  报 "未找到适用于 macos aarch64 的二进制文件"（Apple Silicon 上必现）。

## 变更
- `.github/workflows/release.yml`：matrix 增 `arch`；mac 资产命名为
  `jpcg-app-macos-aarch64`，linux/windows 保持 `x86_64`；命名不再硬编码
- `server_tools/deploy-gen/src/main.rs`：`APP_BINS` → macos/aarch64，清单 `os` 字段
  用客户端词汇（`macos` 而非 `darwin`）
- `server_tools/manifest-gen/src/main.rs`：`detect_platform` darwin → macos
- `server_manifest.md`：目录/清单示例修正为实际 `{os}-{arch}` 命名，去掉 Intel mac

## 决策
- **不保留 Intel macOS**：苹果已淘汰全部 Intel Mac，不为旧机型维护 x86_64 构建；
  每平台单架构，模块 dylib 亦无同名冲突
- 产物命名/清单 `os+arch` 与客户端 `std::env::consts::{OS,ARCH}` 严格对齐

## 验证
- 本机（arm64）dev 构建此前因清单缺 macos/aarch64 命中报错；beta.2 发布后此路径应打通

### 来自 changes/2026-09-08-223500.md

# chore: dev 补齐 release.sh 修复（#11）+ 清理已聚合 changes

日期：2026-09-08

## 背景
- dev 分支此前缺失 #11 的 `scripts/release.sh` 修复（仅 beta 有）：dev 版仍是
  `cargo set-version`（会因 jpcg_const 独立版本降级而中断）+ `release/prep-` 分支名
  （与已有 `release` 分支冲突）。dev 作为最上游分支必须与修复同步，避免后续提升把
  未修版脚本带回 beta/release。
- `changes/` 累积了 56 个已聚合进 beta 分支 `[2.1.0-beta.1]` CHANGELOG 的旧条目，
  若不清除，下次发布会把这些历史再聚合一遍（重复）。

## 变更
- `scripts/release.sh` / `scripts/README.md`：从 beta 同步 #11 修复
  （版本 bump 仅改根 `[workspace.package]` + `sync-version.sh` + `cargo metadata`
  刷 lock；prep 分支名改 `prep/<版本>`）
- `changes/`：删除 56 个已聚合条目，保留未发布条目
  （2026-09-04-200540/210000/220000、2026-09-08-214000 与本仓库内新条目）

## 验证
- 与 origin/beta 对比 `scripts/` 差异为空（完全对齐）

## [2.1.0-beta.3] - 2026-09-12

### 来自 changes/2026-08-09-100000.md

# core/update P1 — 热路径去 clone 与求导去重

日期: 2026-08-09

## 做了什么

1. **`JpcgConfig` 引用化** (`cal/atkcal.rs`)
   - 结构体由持有 6 个 owned 配置改为持有 6 个引用（`&PlayerConfig` 等），增加生命周期参数。
   - 删除无调用方的 `new()`（此前已无人使用），`new_with_config` 全参数改引用。
   - 消除 `call_back`/`compute_derivatives`/`calculate_combo` 三个循环内对
     player/hostilepile/skill/xinfa/buff/coeff 的逐技能全量 clone。

2. **求导去掉二次全链** (`cal/atkcal.rs` `q_cal_with_derivatives`)
   - 原实现先调 y_cal/i_cal/g_cal/h_cal，再调 `q_cal()` 把整条链再算一遍（×2 全链）。
   - 现复用 forward 阶段中间结果，Q 段就地计算，数值公式与 `q_cal()` 完全等价：
     `q = G×(1-crit) + H×crit`，结果数组 `[y, i1, i2, G3, H4]`。

3. **顺带清理** (`type_set/player.rs`)
   - 删除未使用的 `use crate::log::error;` 导入。

## 为什么

- 每个技能重复 clone 6 个结构化配置并重跑全链，是纯冗余计算。引用化 + 复用后，大量 `derive(Clone)` 调用消失，求导路径计算量约减半。

## 约束与保障

- **不改变任何数值行为**：Q 段与结果数组构造逐条对照原 `q_cal()` 等价。
- 数值一致性将由后续 P4 基准单测锁定。
- 未触碰截断/取整逻辑（P2 另行显式化，仍是逐位一致）。

## 其他

- `jpcg_app`（Tauri）先前 cargo check 失败，原因是缺少 `src-tauri/binaries/jpcg_updater-aarch64-apple-darwin`（见 server_manifest.md 预置步骤），已按文档补上，workspace 全量 check 通过。
- 剩余 4 个 dead-code 警告为既有（`Config::load`、`data_load`、`DamageResultWithDerivatives.result` 等），本次未处理。

### 来自 changes/2026-08-09-103000.md

# core/update P2 — 截断逻辑显式化（数值逐位一致）

日期: 2026-08-09

## 做了什么

`cal/atkcal.rs` `g_cal()`:

原先一整行链式 `as u32` cast（嵌套 6 层括号），把游戏实测的 3 个截断点隐在其中。
现拆分为显式 `truncate()` 辅助函数 + 三步注释：

1. ① `I2 × (1+命中增益) × 伤害增益 × (Y/1024)` → 截断
2. ② `× (1 − 化劲/1024)` → 截断
3. ③ `× 全局PVP减伤 × (1 − 目标减伤比)` → 截断输出

新增 `truncate(v: f32) -> f32`：`v as u32 as f32`，与游戏实测截断语义一致。

## 为什么

把「游戏实际检测得出的截断顺序」从嵌套 cast 迷宫中显式化，配上中文注释固定语义，避免后续重构误改或误优化。**这是纯形式化重构。**

## 逐位一致保障

- 运算顺序与优先级完全复刻原表达式（乘左结合，`pvp` 在最后一步内部乘），无新增/移除任何运算。
- 每步取整点与原 `as u32` 出现位置一一对应。
- P4 将以改动前行为为基准做等价测试（快照），双重保险。

## 决策

- 未统一 buff 百分比换算的 `*1024/100`（1024 制系数）与 `/100`（0~1 比率）两种写法 —— 二者语义分别对应系数域与比率域，混写是有意区分，不存在不一致，故不动。
- `y_cal`/`h_cal`/`i_cal` 各自仅有一个 `as u32` 单点截断，本就显式，未触碰。

### 来自 changes/2026-08-09-110000.md

# core/update P3 — toml_input "none" 哨兵改 Option，清理 unwrap_or

日期: 2026-08-09

## 做了什么

1. **`io.rs` `toml_input`** 返回类型 `String` → `Option<String>`：
   - 不再用 `"none"` 字符串充当「文件不存在」哨兵；
   - 文件缺失/读取失败返回 `None`，成功返回 `Some(内容)`。

2. **调用点适配**（公共行为不变）：
   - `cal.rs` 步骤3：`"none"` 判断 → `None => TomlConfig::default()`；
   - `io.rs::load_config`：`None` 直接默认空配置（去掉原先对 `"none"` 再做 from_str 失败的误导日志）；
   - `io.rs::load_save_config`：`None` 时走既有的「未找到配置，使用默认值」warn 路径；
   - `type_set/jcsx_set.rs::data_load`（死代码，预留模块）：`unwrap_or_default()` 适配新签名。

3. **`io.rs::load_config`** `to_str().unwrap_or("").to_string()` + 空串再判断
   → 显式 `match` 处理非法 UTF-8（符合 AGENTS.md 库代码约束）。

## 为什么

- `"none"` 哨兵会误导后续维护：一旦调用方忘记判 `"none"`，会直接 `toml::from_str("none")` 拿到伪装错误；Option 类型强制穷举，编译器兜底。
- `unwrap_or("")` 把非法 UTF-8 路径折叠成空字符串再二次判断，绕弯且易漏。

## 影响面

- 仅为 `jpcg_core` 内部模块使用（`io` 是私有 mod，外部 crate 无直接依赖此符号）；Tauri 侧公共入口（load_config/save_config 等）签名均未变。
- feature 分支 `codex/attribute-config-editor` 不与 `toml_input` 调用冲突（已核实）。

### 来自 changes/2026-08-09-113000.md

# core/update P4 — 金标准基准测试

日期: 2026-08-09

## 做了什么

`cal/atkcal.rs` 新增 `#[cfg(test)] mod golden_tests`，6 个测试锁定五段伤害 + 6 属性导数：

| 测试 | 场景 |
|------|------|
| golden_gong_default / gong_buff | 莫问「宫」（base 160/200, atk 2.609375），默认 / 满 buff |
| golden_zheng_default / zheng_buff | 「徵·豪情」（hit_up=20），默认 / 满 buff |
| golden_wei_default / wei_buff | 「剑·徵·削竹」（watk_xishu=100, wushifangyu=90），默认 / 满 buff |

玩家样本: 根骨 18888 / 基础攻击 4666 / 会心 33000 / 会效 22000 / 破防 25000 / 武器 2800。
目标样本: 21000 防御 / 御劲 8500 / 化劲 35000 / 减伤 35%。
buff 场景: 基础攻击 +10%、会心 +5%、会效 +8%、破防 +3%、伤害 +6%。

## 基准来源（关键）

在**改动前的代码**（master `b12818f` 的 jpcg_core，git worktree 临时检出）上，用同输入跑同一探针程序，
得到全部金标准数值；再用同探针跑**改动后**代码（P1+P2 后），输出 `diff` 为**空**，
实证重构前后数值逐位一致，随后将数值固化进断言。

## 保障

- 六项断言对 Q 段精度 `1e-6`（f32 求导为连续近似，基准即原输出）。
- 若未来改动伤害公式/截断顺序，测试立即红。
- `cargo test -p jpcg_core`：6 passed。
- 探针代码已从工作树移除，临时 worktree 已删除。

### 来自 changes/2026-08-10-181440.md

# 同步远端技能编辑器的 TOML 处理边界

## 内容

- 合入远端的结构化技能编辑器接口 `load_skill_data` / `save_skill_data`。
- 移除旧属性编辑器的原始 TOML 文本读写链路，以及前端 `smol-toml` 解析和序列化逻辑。
- 清理仅服务于旧编辑器的页面、图标依赖和验证素材。

## 原因与决策

TOML 的解析、校验和写入统一由 Rust 核心完成；Tauri 在前端和后端之间只传输结构化 DTO。这样避免双端 TOML 实现产生格式和字段兼容性分歧，也与远端的技能编辑器实现保持一致。

### 来自 changes/2026-08-11-203000.md

# K1: jpcg_core 内部分组（io.rs → store/, cal/ → engine/）

## 什么
- `src/io.rs`（396 行）按职责拆分为 `src/store/` 子模块：
  - `store/paths.rs` — data_dir / combo_presets_dir（路径定位）
  - `store/toml.rs` — TomlConfig / toml_input / load_config / save_skill_toml
  - `store/config.rs` — SaveConfig / save_config / load_save_config / export / import
  - `store/combo.rs` — 连招预设 CRUD
  - `store/profession.rs` — 门派列表扫描
  - `store/mod.rs` — 声明 + 兼容重导出（保持 lib.rs 门面引用 `crate::store::`）
- `src/cal.rs` → `src/engine/mod.rs`，`src/cal/` → `src/engine/`（atkcal/derivatives/kill_prob 提升到 engine 下，不留嵌套）

## 为什么
- 为「core 即 host」架构铺路：store 与 engine 将成为 host 的独立子系统，未来可分别打包/替换
- 缩减单文件规模，职责边界清晰

## 关键决策
- `pub mod engine; pub use crate::engine as cal;` — 对外路径 `jpcg_core::cal::*` 保持不变，Tauri 侧零改动（上层的 `jpcg_core::cal::derivatives::` 等引用全部兼容）
- 删除死代码 `io::Config`（grep 确认无任何引用）
- `jcsx_set`/`all_updates`/`player.rs` 依决策保留（后续补充）
- store/mod.rs 顶层重导出全部函数，lib.rs 门面仅改 `io`→`store` 前缀，行为零变化

## 验证
- `cargo build --workspace`：0 error
- `cargo test -p jpcg_core`：6 个金标准全绿（数值逐位锁定，重构无行为变化）

## 备注
- 与 feature 分支 `codex/attribute-config-editor` 的 `io.rs`/`lib.rs` 冲突面已形成（该分支 +41/-? 行），合入时需手工协调

### 来自 changes/2026-08-11-205500.md

# K2: jpcg_api 纯类型契约 + core host/ JSON 入口 + FFI 重写 + 三 crate 双产物

## 什么
1. **新建 `crates/jpcg_api`**：仅含 serde DTO 类型的 rlib crate（22 个 DTO + ConfigDataDTO），零依赖，
   不引用 jpcg_core —— 作为 Tauri IPC 与 FFI JSON 的单一契约源（防漂移）
2. **core 新增 `host/` 模块**（JSON 契约层）：
   - `host/calc.rs` — calculate / compute_derivatives / into_core / skill_dto_to_skilltype
   - `host/combo.rs` — calculate_combo / 预设 CRUD / export/import config
   - `host/config.rs` — save/load_config / list_professions
   - `host/skill.rs` — load/save_skill_data / load_skill_pool
   - `host/conv.rs` — 原 Tauri types.rs 的全部 From 转换迁入（DTO↔core 双向）
   - `host/update.rs`（net 特性）— update 编排全权收归 core：check/download/验证/**发动
     jpcg_updater 子进程**/请求退出；异步经内嵌 tokio runtime 同步化；
     宿主行为通过 `HostEvents` trait（on_progress/request_exit/updater_path）注入
3. **`ffi.rs` 整体重写**（原 26 行裸指针 → 句柄 + JSON 协议）：
   - `jpcg_abi_version()` 版本协商、`jpcg_handle_create/free`、`jpcg_call(handle, method, request_json)`
   - `jpcg_free_string` / `jpcg_last_error` 错误传递
   - 所有实现经 `catch_unwind` 防护，panic 不跨 FFI
   - 覆盖 16 个业务方法（calculate/derivatives/combo/config/skill/预设 CRUD）
4. **三 crate 双产物**：`jpcg_core`/`jpcg_const`/`jpcg_update` 均 `crate-type = ["cdylib", "rlib"]`，
   产出 libjpcg_core.dylib / libjpcg_const.dylib / libjpcg_update.dylib
5. **`jpcg_const` FFI**：`jpcg_const_get_u32(key)` 按 key 查药物常量
6. **`jpcg_update` FFI**：`jpcg_update_check` / `jpcg_update_fetch_app_info` / `jpcg_update_file_sha256`
   （同步化封装，JSON 进出）
7. core 增加 feature `net`（默认开）：jpcg_update + tokio 为可选依赖

## 为什么
- B 模式（全动态 dlopen）的必要前提：core dll 对外只暴露 JSON 契约，Tauri 壳动态模式
  无需编译期依赖 core 类型
- 「更新时只更新某个 dll」依赖每 crate 独立 cdylib 产物

## 关键决策
- DTO 单源在 `jpcg_api`，From 转换单源在 `host/conv.rs`，Tauri 壳 types.rs 后续（K3）改为 `pub use`
- update 流程（含 updater 启动）从 Tauri 壳迁入 core 的 host::update，壳只实现 HostEvents
- FFI 方法名与 Tauri 命令一一对应，未来可直接生成桥接层

## 验证
- `cargo test -p jpcg_core`：12 passed（6 金标准 + 6 FFI 协议冒烟：句柄生命周期/JSON 往返/
  错误路径/版本协商/save_config 契约）
- `cargo build --workspace`：0 error
- 三个 dylib 产物确认生成

## 备注
- jcsx_set 死代码警告保留（既定决策：后续补充）
- FFI 冒烟测试写 saved_config.toml 到 CWD 后自行清理

### 来自 changes/2026-08-11-213000.md

# K3: Tauri 双模式（static/dynamic）+ 命令薄壳化 + FFI 桥接

## 什么
1. **Tauri Cargo features 双模式**：
   - `default = ["static"]` — 编译期链接 jpcg_core，Rust 直调 host 模块
   - `dynamic = ["libloading"]` — 不依赖 jpcg_core，dlopen libjpcg_core 经 jpcg_call JSON 调用
   - jpcg_update 改为两模式共同依赖（命令签名/进度事件 DTO 需要，计算逻辑仍在 core）
2. **types.rs 薄壳化**：511 行 DTO+From 实现 → 3 行 `pub use jpcg_api::*`
   （DTO 与转换单源在 jpcg_api / jpcg_core::host::conv）
3. **命令薄壳化**（calculate/optimize/config/data/combo/skill_editor/update）：
   每个命令 = tauri::command 签名不变 + `cfg(feature)` 两个 impl：
   - static：直调 `jpcg_core::host::*`
   - dynamic：`ffi_bridge::call(method, &req)`（DTO 自动 JSON 序列化往返）
4. **commands/ffi_bridge.rs**（dynamic 模式）：
   - 定位 libjpcg_core.{dylib,so,dll}：exe 目录 → target/{debug,release} → `JPCG_CORE_LIB` 环境变量
   - OnceLock 单例加载库 + 创建会话句柄 + Drop 释放句柄
   - `jpcg_set_host_events` 注册 C 回调表（进度 → Tauri emit "update-progress"；
     退出 → app.exit(0)；updater 路径 → JPCG_UPDATER_PATH 环境变量或 null）
5. **HostEventsTable 类型单源化**：定义移到 jpcg_api（repr(C) C ABI 契约），
   core ffi re-export，app 无链接即可构造回调表
6. **行为等价性修复**：`load_config` 的 ConfigDataDTO 改为完全镜像
   CalculateRequest 形状（player/hostile/**xinfa_config**/buff/coefficient），
   保持前端 loadConfig() 契约不变（原实现返回 CalculateRequest，且含 buff/coefficient）
7. **DTO 补 Deserialize**：SkillResultDTO / ComboResultDTO / ComboStepResultDTO /
   jpcg_update 的 UpdateProgressEvent / UpdateCheckResult（dynamic 模式 JSON 回读需要）
8. **update 编排所有权**：Tauri 壳 update.rs 从 160 行 → 双模式薄壳；
   perform_app_update（下载→验证→发动 updater→请求退出）已在 K2 迁入
   jpcg_core::host::update，壳只实现 HostEvents（static）或注册 C 回调（dynamic）

## 为什么
- B 模式（全动态更新）的 Tauri 侧基础：切换 feature 即可静态/动态编译，
  前端代码零改动（命令名、参数、返回形状全部不变）
- core dll 更新换代时 app 二进制无需重装；dev 时 JPCG_CORE_LIB 指向本地 dylib

## 关键决策
- 命令签名保持前端契约优先：tauri::command 参数/返回类型与 K2 之前完全一致
- jpcg_update 不并入 optional：其 DTO 是命令签名的一部分，逻辑轻量；
  真正要"可替换"的计算引擎是 core
- 动态模式句柄懒创建于首次 call（OnceLock 单例，进程级复用）
- 回调表幂等注册（OnceLock guard），避免重复注册

## 验证
- `cargo build -p jpcg_app`（static）与 `--no-default-features --features dynamic` 均 0 error 0 warning
- dynamic 端到端冒烟测试（ffi_bridge_tests::dynamic_call_roundtrip）：
  真实 dlopen libjpcg_core.dylib → jpcg_call("list_professions") JSON 往返 → 错误路径
- `cargo test --workspace` 全绿（13 passed）
- `cargo build --workspace` 仅剩既定 jcsx_set dead_code 警告

## 备注
- 前端 src/api/commands.ts 无需改动（命令契约不变）
- 动态模式实际运行需 libjpcg_core.dylib 与 app 同目录（打包时由 K5 构建矩阵保证）

### 来自 changes/2026-08-11-220000.md

# K5: 模块库（dll）增量更新 — modules_manifest.toml 全链路

## 什么
1. **manifest-gen 扩展**：新增 `--modules-dir <dll目录>`（可选）与 `--modules-output`
   （默认 modules_manifest.toml）与 `--platform`（默认按本机 OS 推断 darwin/linux/windows），
   产出：
   ```toml
   modules_version = "v2.1.0"
   platform = "darwin"
   [[files]] name/hash/hash_type/size
   ```
   与 data_manifest.toml 并行生成
2. **jpcg_update::modules（新模块）**：
   - `ModulesManifest` / `ModulesFileEntry` / `ModulesCheckResult` 类型
   - `fetch_modules_manifest`：stable → `files/JPCG/{app_version}/modules/…`，
     beta → `files/JPCG_beta/modules/…`（与 data 布局对齐）
   - `check_modules_update(beta, force)`：对比 exe 同目录 `modules/` 子目录，
     按 SHA256 差量判定需要更新的 dll
   - `download_and_install_modules`：逐文件下载 → 哈希校验 → 临时名原子替换
3. **UpdateCheckResult 扩展**：新增 `has_modules_update` / `modules_version` /
   `modules_files_to_update`；`check_updates` 在 force 或有 app 更新时一并检查模块
4. **core host::update::perform_modules_update**：下载 → 校验 → 安装 → 进度上报 →
   `request_exit()` 请求宿主重启（重启后 ffi_bridge 优先加载 modules/ 新 dll）
5. **FFI**：`update_modules` 方法（请求携带 beta/version/files，经回调表上报进度）
6. **Tauri**：`perform_modules_update` 命令（static 直调 / dynamic 走桥接）；
   **ffi_bridge 加载顺序调整**：exe 同目录 `modules/` 子目录 → exe 同目录 → target → env
7. **前端**：types 增加 ModulesFileEntry 与 UpdateCheckResult 三字段；
   commands.ts 增加 `performModulesUpdate` + checkUpdate/perform 模拟响应；
   ConfigPanel 更新流程新增模块更新分支（确认 → 进度 → 重启）

## 为什么
- B 模式（增量只换 dll，不重装 app）的服务端与客户端能力打通：
  服务器托管 modules_manifest.toml + dll，app 差量下载到 modules/ 并重启加载

## 关键决策
- 模块版本目录复用 app 版本号（app 更新与模块更新同版本目录，服务器部署简单）
- 本地模块落位 exe 同目录 modules/，与 data/ 平行；ffi_bridge 加载优先级
  modules/ 最高 → 旧 dll 原位替换后重启即生效
- beta 通道模块清单不带版本目录（与 data 的 beta 布局一致）

## 验证
- manifest-gen：真实 dylib 生成 modules_manifest.toml（3 个模块，darwin）
- `cargo test --workspace` 全绿（13 passed）
- `cargo build --workspace` 双模式 0 error；`make check-all` 全部通过
- `npx tsc --noEmit` 前端类型检查通过

## 备注
- 服务器部署步骤（参考 server_manifest.md）：`dist/<version>/modules/` 放 dll +
  modules_manifest.toml；beta 放 `dist/modules/`
- 动态模式 app 的 check_update 会请求服务器 modules 清单；服务器暂无 modules 目录时
  自动跳过（fetch 404 视为无更新）

### 来自 changes/2026-08-11-220100.md

# K4: 双构建矩阵 — Makefile + .cargo/config.toml 别名

## 什么
1. **`.cargo/config.toml` 新增 `[alias]`**：
   - `build-app-static`  → `build -p jpcg_app`（默认，编译期链接 core）
   - `build-app-dynamic` → `build -p jpcg_app --no-default-features --features dynamic`
   - `build-modules`     → `build -p jpcg_core -p jpcg_update -p jpcg_const`（三个 cdylib）
   - `test-core` / `test-app-dynamic`
2. **`Makefile`**：
   - `build-static` / `build-dynamic` / `build-modules`（支持 `BUILD=release`）
   - `modules-dir`：把三个 dll 复制到与 app 相同目录（动态模式运行所需）
   - `test` / `check-all`：双模式 + 全 workspace 测试全绿检查

## 为什么
- 把 K2/K3 的两种 app 构建模式固化为一条命令，消除手写 feature 组合的记忆负担

## 验证
- `make check-all` 全部通过（static + dynamic + modules 构建 + workspace 测试）
- `cargo build-modules` / `cargo build-app-dynamic` / `cargo test-core` 别名生效
- 首次失败点：`[alias]` 表头缺失导致 cargo 不识别别名，已补

### 来自 changes/2026-08-11-223000.md

# K6: ctypes Python demo + JPCG_DATA_DIR 环境变量覆盖

## 什么
1. **examples/python_demo/jpcg_demo.py**：ctypes 直连 libjpcg_core.{dylib,so,dll}：
   - 自动定位库（参数 / target/{debug,release}）
   - 完整协议演示：abi_version → handle_create → list_professions →
     calculate（莫问金标准输入）→ 错误路径 → 句柄释放
2. **store::paths::data_dir() 支持 `JPCG_DATA_DIR` 环境变量**：
   - 指向含 `shuxing/` 子目录的目录（或直接是 shuxing 目录）时优先返回
   - 解决 CLI/Python 等 current_exe 不可用场景的数据定位问题
   - Tauri 正常运行路径不受影响（无环境变量时行为不变）

## 为什么
- 跨语言使用 core 的真实场景验证：Python 直接经 JSON 协议调用获得完整计算结果
- 嵌入式（dll 宿主非 JPCG exe）时数据目录无法从 current_exe 推断，需要显式注入

## 验证
- `JPCG_DATA_DIR=./data python3 examples/python_demo/jpcg_demo.py`：
  列出 2 个门派（莫问/周天功），calculate 返回 28 个技能的完整伤害数据
- `cargo test -p jpcg_core` 连跑 3 次 12 passed（此前一次偶发失败已复测稳定）
- `cargo build --workspace` 0 error

### 来自 changes/2026-08-11-231500.md

# M1: 完整维护流程落地（git-flow + CI/CD + 组件独立版本）

## 什么
1. **版本模型（组件独立）**
   - data_version / jpcg_const 采用 `等级.赛季.日期`（如 `130.3.20260602`），
     映射现有 shuxing 数据 `{level, season, modified}`（modified=YYYYMMDD）
   - `jpcg_const` 独立版本（Cargo version = 130.3.20260602）；`jpcg_updater` 独立；
     其余 crates `version.workspace = true` 继承根 workspace（core 版本 = release tag / 安装包命名源）
   - `VersionInfo::compact()` / `label()`：数据版本紧凑串 + UI 美化（`130级第3赛季 (2026-06-02)`）
   - FFI 版本 getter：`jpcg_core_version` / `jpcg_const_version` / `jpcg_update_version` +
     各 crate `*_VERSION` 常量；Tauri `get_module_versions` 命令（static/dynamic 双模式），
     ConfigPanel 底部展示 App/Core/Update/Const 版本
2. **git-flow 分支初始化**：master(生产) + 新建 develop(集成交互) 并推送
3. **CI/CD（.github/workflows/）**
   - ci.yml：PR/push 触发，rustfmt + clippy + 双模式构建/测试 + 金标准 + 前端 build
   - release.yml：tag v* 触发，三平台矩阵构建（app/dll/updater）+ manifest 生成 + GitHub Release
   - deps.yml + dependabot.yml：依赖审计与每周更新
4. **模板与文档**：PULL_REQUEST_TEMPLATE / ISSUE_TEMPLATE(bug/feature) /
   CONTRIBUTING.md（git-flow + commit 规范 + 版本管理 + 发布流程） / CHANGELOG.md（Keep a Changelog）
5. **脚本与工具**：scripts/release.sh（测试→bump→聚合 CHANGELOG→tag→push）、
   scripts/sync-version.sh（同步 package.json/tauri.conf.json/commands.ts 模拟串）、
   rustfmt.toml、.pre-commit-config.yaml；安装 cargo-edit（cargo set-version）
6. **K5 增强**：modules_manifest 逐 dll 带 `version`+`sha256`，`check_modules_update`
   逐 dll 比较（缺条目/版本不同/哈希不同），本地 `modules/modules_manifest.toml` 快照，
   安装后合并写回快照；manifest-gen 从 dll 文件名推断 crate 版本
7. **clippy 基线**：修复 `jpcg_const_get_u32`（裸指针解引用标记 unsafe + Safety 文档，
   deny-by-default lint）；CI clippy 不 `-D warnings`（既有技术债众多）

## 为什么
- 从无 CI/无分支策略/版本散落 5 处的状态，建立标准化的协作与发布流程
- 组件独立版本契合 B 模式（dll 独立替换）与数据/常量频繁调整的现实
- release tag 跟 core，安装包命名用 core 版本，UI 独立展示

## 关键决策
- 采用 git-flow（master+develop）而非 GitHub Flow：版本化桌面软件 + 定时发布
- 版本用 cargo 原生机制（`version.workspace = true` + `cargo set-version`）而非
  cargo-workspace-version：后者不解析 glob 成员且会覆盖独立版本（const/updater），与组件独立模型冲突
- 模块更新改为逐 dll 版本/哈希判断，本地快照
- CI clippy 暂不 `-D warnings`（既有 ~30 处告警，逐步清理）；release 矩阵用免费额度

## 验证
- `make check-all` 全绿（双模式构建 + workspace 测试）
- 金标准 12 passed；`cargo clippy --workspace --all-targets` 无编译错误
- `cargo fmt --all -- --check` 通过；前端 `npx tsc --noEmit` 通过
- manifest-gen 正确生成逐 dll 版本（core=2.1.0-alpha.1、const=130.3.20260602）
- `cargo metadata` 确认各 crate 版本对齐

## 备注
- master/develop 分支保护（强 review + CI 绿 + squash + 禁直接 push）需在 GitHub Settings 手动开启
- develop 已从 master 创建并推送；core/update 待经 PR 合入 develop

### 来自 changes/2026-08-11-232000.md

# M2: 开源规划文档 — AGENTS.md 更新 + 新增 PLAN.md

## 什么
1. **AGENTS.md 更新**：
   - 删除过时行 "No CI, test suites, or formatter config exist"（与已落地的 CI/测试/rustfmt 矛盾）
   - 新增「开源与命名（当前决策）」段，记录：
     - 保留 `JPCG` 命名（改名"试剑/演武"等已搁置；范围仅产品名，内部 crate/FFI/env 不动）
     - `data/` 暂留仓库；未来联系其他项目负责人获取 lua 数据访问权限后再定发布方式
     - 开源状态：仓库私有（GitHub Free），分支保护需转公有或 Team
     - 场景扩展：core 引擎本就场景无关，PVE 支持主要是 UI 预设 + 数据模板
2. **新增 PLAN.md**：开源与发展路线图（背景 → 当前状态 → M0-M4 里程碑 → 风险合规 → 决策记录）
   - M1 转公有与分支保护、M2 数据授权与合规、M3 开源运营、M4 场景扩展

## 为什么
- 固化"先保留现状、未来再评估改名/数据授权"的决策，供后续协作与开源时遵循
- PLAN.md 作为开源路线的单一事实来源

## 关键决策
- 暂不改名、暂不动 `data/`；数据最终形态由 M2（上游 lua 授权结果）决定
- 开源 = 转公有（免费获得完整分支保护），而非升级 Team

## 备注
- 用户将据本 PLAN 执行转公有；M2 需主动联系上游项目负责人

### 来自 changes/2026-08-11-235000.md

# M3: 开源许可落地 — GPL-3.0 LICENSE + 各 manifest 统一

## 什么
- 新增根 `LICENSE`（GPL-3.0 全文，674 行）
- `jpcg_updater` / `jpcg_app` 补充 `license.workspace = true`（此前缺）
- 前端 `examples/jpcg_app/package.json` 补 `license: "GPL-3.0"`
- 全 8 个 Rust crate 经 `cargo metadata` 确认均解析为 `GPL-3.0`

## 为什么
- 开源前补齐许可；Cargo.toml 已声明 GPL-3.0，落地 LICENSE 文件并统一所有成员

## 关键决策
- 选 GPL-3.0（强 copyleft）：与社区工具(JX3Toy)一致，保证衍生品开源；
  与 PLAN M2 数据授权（上游多宽松/GPL）可兼容，后续拿到具体协议再复核

### 来自 changes/2026-08-12-000000.md

# M4: 三分支线性迁移（dev/beta/release）+ 发布脚本重写

## 什么
1. **分支重构**：从 git-flow（master/develop）迁移到三分支线性模型：
   - `develop` → `dev`（集成/最上游，版本 alpha.n）
   - 新建 `beta`（预发布/公测，版本 beta.n）
   - `master` → `release`（稳定/生产，版本 X.Y.Z）
   - 线性关系：`release ⊂ beta ⊂ dev`，单向超集链
2. **远端操作**：推送 dev/beta/release，设默认分支为 dev，删除旧 develop/master
   （先删保护规则再删分支）
3. **分支保护**：dev/beta/release 三条规则（review≥1 + 3 个 CI 检查 + linear +
   conversation + 禁强推/删除 + enforce_admins:true；beta/release 为 strict）
4. **PR 迁移**：原 #3（LICENSE）因 base 分支删除被自动关闭，重建为 #4（feature/license → dev）
5. **scripts/release.sh 重写**：`--stage alpha|beta|release`
   - alpha（dev）：不 tag；beta：tag vX.Y.Z-beta.n；release：tag vX.Y.Z
   - 校验所在分支与 stage 匹配；测试 → bump → 聚合 CHANGELOG → commit → tag → push
6. **CI/CD 适配**：ci.yml 触发改为 dev/beta/release；release.yml 按 tag 判定
   beta/stable 通道（tag 含 -beta → beta 通道）
7. **文档**：CONTRIBUTING / AGENTS / PLAN 更新为三分支模型

## 为什么
- 用户要求：所有更改融合到 dev → 出 beta.n 公测 → 稳定后 release，线性链式发布

## 关键决策
- release 初始指向旧 master 基线（65e3da1，最后稳定）；beta 同指向旧基线，
  首次提升 dev→beta 才填充
- enforce_admins 保持 true（最严格，需第二 reviewer 或临时降级）
- tag 区分通道：vX.Y.Z-beta.n→beta，vX.Y.Z→stable

## 备注
- PR #4（feature/license → dev）待合并；dev 已含全部 core/update 工作
- 后续出 beta/release 用 `scripts/release.sh beta|release`

### 来自 changes/2026-08-12-100000.md

# UI: 薄荷/松石绿清新主题 + 自绘 SVG 图标

## 什么
1. **配色重构**（`variables.css`）：从深色靛蓝/紫改为薄荷/松石绿
   - 主色 teal `#14b8a6` → sky `#0ea5e9` 渐变
   - **浅色为默认**（`:root`），保留深色主题切换
   - 更新 useTheme 默认主题为 light
2. **自绘 SVG 图标**（新增 `components/icons.tsx`，~18 个内联线条图标，
   stroke=currentColor 继承主题色，不依赖第三方库）：calc/globe/combo/trend/pencil/
   chevron/sun/moon/gear/trash/save/star/close/check/alert/bug
3. **替换全部 emoji**（App 反馈按钮/logo、ActivityBar 页签+折叠、ThemeToggle、
   Toast 状态、ForumPage 翻页、ComboPage 操作、OptimizePage 关闭等）
4. **排版/美术打磨**：卡片阴影、ActivityBar 激活药丸、Header Logo 图标、按钮/输入
   聚焦青色光晕、Toast 状态图标着色、StatusBar 状态点光晕
5. **清理硬编码旧色**：`rgba(99,102,241)`(indigo) / `rgba(168,85,247)`(purple) 统一为 teal/sky
6. **死代码清理**：删除过时 `src/index.html`、`src/css/style.css`（引不存在的 js/app.js、
   css/components.css，React 未使用）

## 为什么
- 用户要求"清新配色 + 排版美术"；采用薄荷/松石绿（A 方案），emoji 改自绘 SVG

## 关键决策
- 图标全部内联 SVG、无新依赖；currentColor 保证跟随浅/深主题
- 浅色为默认（清新观感），深色仍可切换
- 删除死代码降低混淆

## 验证
- `npm run build`（tsc + vite）通过
- `cargo build -p jpcg_app` 通过
- emoji 全量替换后无残留

### 来自 changes/2026-08-12-224904.md

# server_tools/data-import：打通 libpak 拆包（读 skills.tab + 提取 lua + base_damage）

## 什么
1. **修复并打通 PC Pakv4 拆包管线**（此前 `pak.rs` 一直失败）：
   - 正确的 libpak FFI：`init(path)` / `tab_init(path, indexs, fields)` 3 参数 /
     `tab_get(tabname, key, buf, len)` 4 参数（key 带尾随 `\t`，返回字节长度）/ `lua_get`
   - libpak 需 `./cache`（CWD 相对）jx3calc 缓存，缺失时 `init` 返回 -1；
     缓存源在 `~/Library/Application Support/JX3/AssetsDatCache/zsCache/`
   - `tab_init` 的 fields 必须**精确匹配 skills.tab 表头**（用 jx3calc `Skill` 枚举 20 列），
     传错字段名返回 -1；正确字段集下一次 tab_init + 连续 tab_get 数万技能稳定
2. **新增 `src/pakx.rs`**：封装上述正确用法 + `--pak-dump` 模式
   - 遍历 SkillID → 读 skills.tab 20 列 → 提取每个技能 lua 字节码到 `lua/` → 用
     patched luac 反编译解析 `tSkillData[].nDamageBase` → 输出 `skills.tsv` manifest
3. **`main.rs` 新增 `--pak-dump` / `--pak-out` / `--luac` 参数**

## 为什么
- 用户要求"跑通游戏文件拆包"。此前 `pak.rs` 用错 FFI 签名/字段名导致 init 失败、
  tab_init 卡死。本次彻底摸清 libpak 用法并跑通全流程。

## 关键决策 / 发现
- **base_damage 不在 skills.tab**，而在技能 lua 的 `tSkillData[level].nDamageBase`
  （满级值；真实伤害在"实际伤害子技能"如 14474 宫，父技能 14064 仅 L1=100）
- **atk_xishu 不在 skills.tab 或 lua 常量**：为 jx3calc 执行 `GetSkillLevelData` 后派生
  （内功 = frames/192，外功 = frames/160，frames 每技能整数），需 lua 执行引擎，
  当前未在本次落地（下一步 P0）
- `SkillCoefficient` 列仅 NPC/副本技能有值（固定 25600），玩家技能为空

## 验证
- 全量扫描：读取 35303 技能，提取 22115 个 lua，解析 22115 个 base_damage
- 抽查莫问：14474宫=160、14311商=110、14100羽=110，与现有 mowen.toml 一致
- `cargo build -p data-import` 0 error（仅既有/无害 warning）

## 备注
- 运行前需：`mkdir cache && cp ~/Library/Application Support/JX3/AssetsDatCache/zsCache/* cache/`
- atk_xishu（系数）依赖 lua 执行引擎，下一步用 mlua 复刻 jx3calc 的 GetSkillLevelData 派生

### 来自 changes/2026-08-12-230000.md

# data-import + 引擎：全等级技能数据 → 嵌套 levels TOML → 计算器选等级

## 什么
1. **Lua 执行器 `src/luaex.rs`**：用 mlua(Lua 5.1) 执行游戏技能 lua 的 `GetSkillLevelData`。
   - patch mlua vendored Lua 源码（lundump.c 跳过 header + 4字节字符串）加载魔改字节码
   - stub 游戏全局（Include/ATTRIBUTE_TYPE/ATTRIBUTE_EFFECT_MODE/tSkillData/...）
   - 捕获 `SKILL_*_DAMAGE`(base1)、`_RAND`(rand)、`nChannelInterval`(frames)
2. **全等级提取 `--gen-data`**：遍历所有技能×等级，算
   - `base_damage1 = SKILL_*_DAMAGE`、`base_damage2 = base1 + RAND`
   - `atk_xishu = nChannelInterval / 192(内功) 或 /160(外功)`（按 KindType）
   - 输出 TSV（skill_id/name/script_file/kind_type/max_level + 每级 base1/base2/atk）
   - 当前提取 1888 个有伤害数据的技能
3. **TOML 生成 `gen_toml.py`**：按门派目录分组 → 输出**嵌套 levels** TOML（14 门派）
   - `[[skill]]` 内嵌 `[[skill.level]]`（level/base_damage1/base_damage2/atk_xishu）
4. **引擎等级支持**：
   - `Skilltype` 加 `levels: Vec<SkillLevel>`（serde rename="level"）+ `current_level`
   - `SkillLevel` 结构 + `select_level()`/`max_level()` 方法
   - `SkillEditorItemDTO` 加 `levels`/`current_level`，conv.rs 双向映射
   - store 模块公开
5. **前端等级选择器**：SkillEditorPage 伤害系数区加等级下拉，选级应用 base/atk

## 为什么
- 用户要"输出全等级，让用户在计算器内自己选等级"。
- 解出核心公式：base_damage 在 lua tSkillData（执行 GetSkillLevelData），
  atk_xishu = frames/192(内功)/160(外功)，frames=nChannelInterval。
- 251203.json(2025.12) 系数已过时（技改 nerf，如宫 2.609→1.7325），必须从当前 pak 提取。

## 关键决策
- **纯解包，忽略 csv**：一切 base_damage/atk_xishu 从当前 lua 执行得出
- 嵌套 `[[skill.level]]` 格式（一个技能一条，内嵌多级）
- base_damage 需执行 lua（非满级常量）：商= tSkillData[1]*0.7=14
- 等级规则：暂用 skills.tab max_level（宫25/商3）

## 数据源限制
- **段氏/周天功、无相楼（幽罗引）不在本地 pak**：PC Pakv4 段氏有元数据无 lua，
  无相楼连 skills.tab 都没有；无界数据(2025.2)更早无此门派。需新数据源（Pakv5/新版客户端）。

## 验证
- `cargo test -p jpcg_core --test levels_parse`：解析嵌套 levels + select_level 通过
- 实际生成的 mowen.toml 能被引擎加载并选等级
- `cargo build --workspace` 0 error
- 前端 `npx tsc --noEmit` 通过

## 备注
- 生成的 TOML 用的是 skills.tab 原始技能名（如"宫伤害_实际伤害子技能"），
  含全部子技能，非玩家向精简清单——技能名/筛选需后续整理
- gen_data 中 13188 技能无 lua（NPC/被动/路径缺失），核心玩家伤害技能已覆盖

### 来自 changes/2026-08-13-101500.md

# 全门派 shuxing 扩充至 19 门派（老门派数据补全）

## 变更
- `data/shuxing/` 从 14 门派扩充至 **19 门派**，新增：
  - 老四门：纯阳(97)、天策(86)、明教(69)、少林(72)
  - 其他：丐帮(133)、七秀(86)、万花(98)、五毒(86)、唐门(5→112)
  - 扩充：藏剑(1→61)、纯阳(9→97)、天策(8→86)、明教(2→69)
- 全部 TOML `modified` 更新为 20260813

## 起因
- 老门派（纯阳四象轮回等）技能 lua 之前缺失：PakV4 客户端（zhcn_hd）经
  PakV4-Extract（jx3pak 组织工具，Engine_Lua5X64.dll + `KG_InitPakV4FileSystem`
  加载 Trunk.Dir）黑盒探测 1888 个已知技能路径，仅 6 个 NPC/test 命中——
  **决定性确认玩家门派技能 lua 不在 PakV4**。
- Trunk.Dir 为 HTree 哈希树（`[PakV4] Is Use HashName`，nHeight=5），无明文路径；
  StreamDownloader 日志证实客户端数据完整（无增量下载）。

## 数据来源
- 无界缓存 `~/Library/Application Support/JX3/AssetsDatCache/zsCache/` 更新后
  （用户 8/12-13 游戏运行写入新 cache），libpak 全量提取从 1888 → **3448 技能**。
- 老门派技能路径格式为 `纯阳\子技能_四象轮回_非定身.lua`（子技能_/门派名前缀），
  与旧格式不同（这是之前探测失败原因）。
- 长歌/霸刀/苍云/凌雪/药宗/衍天 6 门派数据与旧版完全一致（回归验证通过）。

## 验证
- `cargo test --workspace`：14 通过 0 失败（含 6 金标准）
- `parse_generated_mowen_and_select` 通过（新 TOML 解析 + 等级选择）
- 前端 `tsc --noEmit` 通过

## 遗留
- 纯阳/少林/天策/明教为**全技能**数据（含子技能），尚未按心法奇穴手动核对
- `server_tools/data-import/` 的 gen_toml.py 的 `modified` 日期为硬编码，后续可参数化

### 来自 changes/2026-08-13-114500.md

# 无相楼（幽罗引）加入 —— 20 门派完成

## 变更
- 新增 `data/shuxing/wuxianglou.toml`（无相楼/幽罗引，第 20 门派）
- 25 个伤害技能（skill_id 41116-42417 段）：挑丝(41342, atk=2.5)、绊线(41444, atk=3.75)、
  勾线(2.1979)、四边静(2.0833)、千里急一~四段(0.7291/1.0937/1.4583/2.1875)、生地狱、
  锁南枝、傀梦令、障幕虚影、应天长(7.2916) 等，均为单级（level=1）
- 心法：幽罗引，根骨阴性内功，atk_up=1.96 / pofang_up=2.0

## 无相楼数据特性
- **无 base_damage**（用户确认无相楼技能无 base，只有 atk 系数）
- atk_xishu 取自魔盒 skill API（node.jx3box.com/resource/std/skill/list include=parse）的
  技能描述 `[X*最终阴性内功攻击]`，系数准确

## 背景
- 无相楼技能 lua 不在本地 PakV4/无界缓存（zsCache dat 无，libpak 确认 41342 不在
  skills.tab 35303 行）
- PakV5 FileList 服务下载持续失败（vk_mb/mac_mb/ios_mb/android_mb 全试，CrossOver 与
  wine11 均 bad FileList）；KGPK5 hash 函数 wine 调用崩溃（日志未初始化）
- `~/Library/Application Support/JX3/cache/skill_m.cache`（34MB）含全部 179 个无相楼
  技能脚本路径（GBK 明文），验证了技能名与 ID 对应（挑丝=织心谣_丝线穿刺等）

## 验证
- `cargo test -p jpcg_core`：12+1 全过（含 6 金标准、parse_generated_mowen_and_select）
- modified = 20260813

## 遗留
- 无相楼仅 25 个有伤害系数的技能；纯控制/机制技能（夹线/替形幻生等）未收录（atk=0）
- base_damage=0（游戏设计如此，非缺失）
- 若后续打通 PakV5 数据下载，可补全子技能与等级数据

### 来自 changes/2026-08-13-160000.md

# 撤销 toml 数据与 data-import 解析器提交（等待完整新数据）

## 变更
- `feature/ui-fresh` 分支 `git reset --hard 571c098`，移除 5 个提交：
  - `fa21db8`/`248bd14`/`5ffc9e0`/`375d943`：data-import 生成器/解析器（libpak 提取、luaex、按心法分类）
  - `c564220`：全门派全等级 shuxing 数据 + 引擎/前端等级支持
- 删除工作区残留：`server_tools/data-import/`（探针工具）、`luac.out`
- 保留 4 个探索 changelog（2026-08-12-224904/230000、2026-08-13-101500/114500）作为历史记录
- PR #6（feature/ui-fresh → dev）force push 后仅含 UI 主题提交

## 起因
- 现有 TOML 数据与游戏当前版本数值不一致（宫 atk_xishu 1.7325 vs 实测 2.6095，差 1.506 倍）：
  数据源是 2025.2 旧缓存快照（AssetsDatCache），与正式服版本脱节
- 已获得完整的新数据来源，明天送达；届时平铺单等级数据直接替换 `data/shuxing/*.toml`，
  无需 data-import 工具与等级（levels）功能

## 遗留
- 等级功能（Skilltype.levels/select_level/current_level、DTO、SkillEditorPage 等级选择器）随提交移除，
  新数据不需要；如将来需要可从 git 历史恢复

### 来自 changes/2026-08-13-161500.md

# DOT 持续伤害计算实现（每跳等比递增 + 结果展开）

## 变更
- **数据契约**：`Skilltype.dot_num` 移除，新增 `dot_interval`（每跳间隔秒）/`dot_duration`（持续秒）；
  总跳数 = `dot_duration / dot_interval`（`Skilltype::dot_jump_count()`，非法数据返回 0）
  - 同步：`jpcg_api::SkillEditorItemDTO`、`host/conv.rs` 双向映射、前端 `types/index.ts`、SkillEditorPage（Dot跳数 → 每跳间隔/持续时长）
- **引擎**（`engine/atkcal.rs`）：
  - `dot_jump_expected()`：每跳期望 = 首跳期望 × 等比递增 `(1+dot_up)^(k-1)`（k=0..n-1）
  - `q_cal()`：dot 技能 `q_damage` = 各跳期望之和（总伤害），`DamageResult.dot_jumps` 返回每跳集合
  - `q_cal_with_derivatives()`：导数链整体 × 等比和因子 `((1+u)^n-1)/u`（u=0 退化为 n 倍）
- **输出**：`SkillResultDTO.dot_jumps: Vec<u32>`（计算面板）+ `ComboStepResultDTO.dot_jumps`（连招每步）
- **前端**：ResultTable 与 ComboPage 在期望列下展开每跳伤害（小字标签，hover 显示第 k 跳）
- **金标准**：`golden_gong_dot`（宫 + 6 跳等比 8%，手算期望锁定公式）

## 起因
- dot 字段（dot_flag/dot_up）此前仅定义未接线，dot 技能被当单次伤害计算（低估 6 倍）
- 用户实测：商 DOT 每 3 秒一跳共 18 秒（6 跳），面板每跳倍率 0.2080（正式服数据明天送达后校准）

## 验证
- `cargo test --workspace`：全过（含 7 金标准）
- `cargo clippy -p jpcg_core`：无新增警告
- 前端 `tsc --noEmit` 通过

## 遗留
- 正式服真实 dot_up/dot_interval/dot_duration 数值随明日新数据入库后按金标准校准
- 连招 kill_prob 的方差对 dot 技能按单跳近似（每跳独立会心未逐跳展开）

### 来自 changes/2026-08-13-170000.md

# 真实面板属性金标准 + Python 全量快测 + DOT 数据完善

## 变更
- **数据完善**（`data/shuxing/mowen.toml`）：商（dot）/角（dot）补 `dot_interval`/`dot_duration`
  - 普通版：3 秒一跳、18 秒（6 跳，不递增）
  - 疏曲版：2 秒一跳、18 秒（9 跳），`dot_up` 0.2 → **0.12**（每跳等比递增 12%）
- **金标准替换**（`crates/jpcg_core/src/engine/atkcal.rs`）：
  - 输入改为真实面板属性（2026-08-13 用户提供）：基础属 21371、基础攻击 64329、会心 61877、
    会效 2925、破防 109160；目标外防 15176、内防 21388、御劲 5047、化劲 59402、减伤 0；pvp 0.9
  - 期望值由引擎输出回填（与 python 快测输出完全一致），锁行为待实测校准
  - dot 金标准重写：`golden_shang_dot`（普通 6 跳相等 + q=Σ）与 `golden_shang_dot_shuqu`
    （疏曲 9 跳等比 1.12 + q=Σ）
- **新增 `examples/python_demo/quick_calc_test.py`**：直连 libjpcg_core 全量计算整个 toml
  - 预设真实属性（面板/木桩/pvp 0.9），输出全部技能 Y/B/I/N/H/Q + dot_jumps 集合
  - DOT 断言：普通/疏曲跳数与等比规则、q=Σ、非 dot 为空
  - 面板对照：化劲 76.32%、御劲 2.55%、外防 10.75%、内防 14.51%（引擎系数全部吻合）
  - `--data`/`--lib`/`--out` 参数；退出码 0/1

## 起因
- 旧金标准使用虚构属性（18888/4666），与实际游戏脱节；用户提供真实面板数据作为基准
- DOT 字段此前缺失 interval/duration，无法计算跳数

## 验证
- `cargo test --workspace` 全过（含 8 金标准，新值 = python 快测输出）
- `python3 examples/python_demo/quick_calc_test.py` ALL PASS
- `cargo clippy -p jpcg_core` 无新增警告；前端 `tsc --noEmit` 通过

## 遗留
- 金标准期望值为引擎自锁，待用户木桩实测（宫/商/徵 等）后校准关键值
- 明天完整新数据到货后：替换 `data/shuxing/*.toml`，`quick_calc_test.py --data` 指向新目录重跑全量

### 来自 changes/2026-08-14-105342.md

# json-to-toml 转换器 + mowen.toml 从数据源重生成（50 技能）+ 引擎 f32 化

## 变更
- **新增 `server_tools/json-to-toml`**：从 JPCG 数据源技能 JSON 目录转换生成 `data/shuxing/*.toml`
  - 用法：`cargo run -p json-to-toml -- --xinfa <技能id> --json-dir <JSON根目录>`
  - 数值字段按数据源原值写入（引擎侧已按 `X_MULT` 归一化，不做除法）
  - `dot_interval`/`dot_duration`：数据源毫秒 → 浮点秒（`ms/1000`），支持小数（青莲剑 0.25s）
  - `dot_up`：优先读 `dot_param` 公式中 `j => i*0.12` 的 `dot_up` 系数（疏曲 0.12），回退 `dot_up` 字段
  - **不再输出 `jihuoqixue = "疏曲"`**（用户要求，跳数由引擎按 duration/interval 推导）
  - `design_effect`/`kind_type` 按 JSON 属性自动推断；`cast_mode`/`guaranteed_hit`/`effect_type` 用安全默认
  - `**x` 幂运算符支持（dot_param 公式 `631 * (j ** 0.99592)`）
- **`data/shuxing/mowen.toml` 重生成**：旧 34 技能 → 新 **50 技能**（含 DotOut 目录完整展开：
  宫/商/角/徵/青莲剑·徵 + 疏曲变体、相依/无尽藏/阳春雪/超然/裂涛等奇穴激活输出）
  - 关键值核对（数据源为准）：宫 atk_xishu 1.0325 → 2.609375（引擎 X_MULT 归一化后一致），
    商 2.4479167、疏曲 dot_up=0.12、青莲剑 dot_interval=0.25、max_tick tick→1 修复、`(tick-1)/(max_tick-1)` 全浮点
- **引擎 f32 化**（`crates/jpcg_core`）：
  - `skilltype.rs`：`SkillUp.RateUp` 变体字段 `f32`（`utils.rs` `ReadWidgetOutputData` 列默认值对应改为 `0.0`）
  - `crates/jpcg_api`：`BankNote` 透传 f32（`compound_use` 走 serde `as`）
  - `atkcal.rs` 金标准（`obj_shang_damage1/2` 等）对应更新，全部按确定性指数公式计算
  - 动机：`atk_xishu` 等按数据源原值写入后为小数（2.609375），u32 放不下；后果：面板/buff/属性含义不变
- **`server_tools/manifest-gen`**：`golden_tests` 误引用已删除的 `engine::datamake` → 改为 `engine::atkcal`；去掉多余 `toml` 依赖
- **overrides.json 说明**：`--json-dir` 数据源含心眼技能目录共 3 个（10447 主目录 + 10781/ 10782/ 覆盖目录），overrides.json 当前未启用（可后续补心眼处理规则）

## 起因
- `data/shuxing/*.toml` 此前为手工维护，与数据源脱节；用户提供数据源目录，要求按数据源重生成
- `dot_interval`/`dot_duration` 此前仅支持整数秒，无法表达 0.25s 小数值

## 验证
- `cargo build` 全 workspace 通过；`cargo test` 全过：
  - `jpcg_core` 103 测试（含 2 个新 dot 金标准：普通 6 跳相等、疏曲 9 跳等比）
  - `json-to-toml` 输出 50 条技能，与旧 mowen.toml diff 检出核心值一致
- 转换器重跑幂等性核对（两次生成 diff 为空；疏曲/青莲剑/相依/无尽藏输出均与预期一致）

## 遗留
- 心眼练技能（10781/10782 覆盖目录）转换规则未实现，overrides.json 待补
- dot_up=1 的 dot 条目（阳春雪/modaowan）也能被引擎正确处理，数据源如此即保留
- data/shuxing/mowen.toml 已替换为新生成版本，正式发布前需以新数据跑 `quick_calc_test.py` 全量快测

### 来自 changes/2026-08-14-113755.md

# 修复 dot 基础伤害（base_damage）缺失 + quick_calc 断言泛化

## 变更
- **转换器 bug 修复**（`server_tools/json-to-toml/src/main.rs` `collect_dots`）：
  - 根因：base 提取 `damages[0]` 时漏了 `.get("source_attribute")` 层级，`extract_base`
    永远返回 (0,0)，渲染端 `if base1!=0||base2!=0` 因此不输出 → 7 个 dot 全丢 base
  - 修复：与 `extract_skill` 对齐，`damages[0].source_attribute` → `extract_base`
  - 结果：商/角（含疏曲）=58/58；青莲剑·商/角=26/126；青莲剑·徵=26/26
- **`data/shuxing/mowen.toml`**：重生成，diff 确认仅新增 7 处 dot 的 `base_damage1/2`（其余无变化）
- **`quick_calc_test.py` 断言泛化**：
  - `check_dot` 不再硬编码"第1次=6跳/第2次=9跳"，改为从 toml 按文件顺序读每个 dot 的
    `dot_duration`/`dot_interval`/`dot_up` 推导：跳数 `n=round(duration/interval)`
    （支持浮点秒，青莲剑·徵 2/0.25=8 跳、角 4/1=4 跳、商 1.5/1.5=1 跳），
    `dot_up>0` 等比 ×(1+up)^k 否则每跳相等；q=Σjumps；toml 条目必须全部被 results 覆盖
  - 无质断言：删除硬编码 `WUZHI_NAMES=("相依1","相依2")`，改为从 toml 自动收集
    `has_critical_strike=true` 技能（相依(lv3)/相依(lv4)），并校验其全部出现在结果中
  - 新增 `load_data()`：统一读取 mowen.toml（缺 tomllib/找不到时跳过数据依赖断言）

## 起因
- 用户实测发现 dot 基础伤害（base_atk）丢失：重生成的数据里 `base_damage1/2` 字段缺失，
  旧版手写数据有（商/角 = 58）
- quick_calc 旧断言假设所有 dot 仅 6/9 跳两种形态，与数据源青莲剑系（8/4/1 跳）不符

## 验证
- `cargo test --workspace` 全过（15 测试套件，含 dot 金标准 6/9 跳，金标准不读数据文件不受影响）
- `cargo clippy -p json-to-toml` 零警告
- `python3 examples/python_demo/quick_calc_test.py` → ALL PASS（EXIT=0）：
  - dot 跳数：商/角 6 跳相等、疏曲 9 跳等比、青莲剑 8/4/1 跳，全部按 toml 推导断言通过
  - 无质：相依(lv3)/(lv4) Q 固定=期望公式
  - 面板对照：化劲 76.32% / 御劲 2.55% / 外防 10.75% / 内防 14.51%
- 转换器重跑幂等：两次生成产物 diff 为空

## 遗留
- 金标准 dot 期望值基于旧 base（58），与新数据一致无回归；如后续校准以 `--out` 输出回填

### 来自 changes/2026-08-14-133145.md

# 全职业 toml 导出（27 个伤害心法）+ 全量加载验证

## 变更
- **全职业导出**：`data/shuxing/` 新增 26 个心法文件（10002/10003/10014/10015/10021/10026/
  10062/10081/10144/10175/10224/10225/10242/10243/10268/10389/10390/10464/10533/10585/
  10615/10627/10698/10756/10821/10786），连同 mowen.toml = 27 个伤害心法全覆盖
  - **overrides.json 全量生成**：`xinfa_name`/`xinfa_nom` 从 belongs.json 自动推断
    （`*_to_*_attack_power` 前缀映射：spirit→根骨、spunk→元气、strength→力道、agility→身法、
    vitality→体质），覆盖全部 27 池
  - 文件名沿用历史命名：10447→mowen.toml、10786→zhoutian.toml（保留周天功手写校准值
    atk_up=1.96/pofang_up=0.3）；其余按池 id 命名
  - 治疗心法（10028 离经/10080 云裳/10176 补天/10448 相知/10626 灵素）与通用池 0 在数据源中
    无伤害技能条目，不导出
- **datamake.md**：补充全职业导出命令、overrides.json 字段语义

## 决策
- `atk_up`/`pofang_up`：数据源无此数据（belongs.json 只有属性转化，无攻击/破防倍率）。
  莫问（1.96/2.0）、周天功（1.96/0.3）保留手写校准值，其余职业暂用 1.0 占位，待逐职业校准
- `wuzhi`（无质名单）：无数据支撑，仅莫问（相依）保留手写，其余职业空
- 各职业跳过技能（无伤害数据）少量且为真实无伤害条目（御鸿于天/地坼/幽都判/织翠 等），符合预期

## 验证
- FFI 全量加载：27 个职业逐一 `calculate` 成功，技能数与转换器输出一致
- `cargo test --workspace` 全过（16 金标准 + dot 金标准不受数据文件影响）
- quick_calc_test.py（莫问）ALL PASS

## 遗留
- 其余 25 职业 atk_up/pofang_up/wuzhi 为占位值，需在游戏中实测属性换算后校准
- 纯 base 伤害技能（无 atk_xishu）数量较多（如笑尘诀 33 条），为数据源原始值

### 来自 changes/2026-08-14-144008.md

# 修复连招击杀概率恒为 0（target_hp 单位 ×10000 残留）

## 变更
- **修复根因**：`engine/kill_prob.rs` 中 `target_hp` 仍按旧单位乘以 10000（
  `hostilepile.target_hp as f64 * 10000.0`），而前端已按"个位"输入血量（如 2915274）。
  旧逻辑把血量放大成 291.5 亿，累计期望伤害永远打不死 → 击杀概率恒为 0
- **修复**：`target_hp` 直接使用 `hostilepile.target_hp as f64`，与血量输入单位对齐
- **确定性分支**：当 `cum_std == 0`（会心率 0% 或全无质连招）时，期望伤害 ≥ 血量即
  必杀（kill_prob = 1.0），否则 0.0，避免纯确定性连招误走正态分支
- **无质技能**：`has_critical_strike`（无质）技能伤害固定为期望 Q，方差为 0
- **DTO 透传**：`ComboStepResult` 输出 `has_critical_strike`/`zhenshishanghai` 字段，
  conv.rs 映射到 `ComboStepResultDTO`

## 验证
- FFI 复现用户场景（20×风来吴山、血量 2915274、会心 48.6%）：击杀概率从第 5 步开始
  爬升（0.02%），第 9 步 89.84%，第 13 步起 100%，final = 99.99999999999998%
  （修复前全部恒 0）
- `cargo test --workspace` 全过（16 金标准 + dot 金标准不受影响）

## 遗留
- 用户侧需更新 core 二进制：dynamic 模式替换 `modules/` 或 exe 同目录 dll；
  static 模式需重新编译 App（`make build-app-dynamic`）

### 来自 changes/2026-08-14-150000.md

# 技能形态全量输出（放开连续 lv 压缩）+ 技能池搜索/折叠

## 变更
### 数据层（server_tools/json-to-toml）
- **放开 `is_series()` 压缩**：所有技能的全部 lv 形态（等级/层数/点数/距离/目标数等）
  全部输出，不再只取最大 lv。全库 990 → 1627 条目
  - 段氏（zhoutian.toml）：引窍 101 形态（0~100 点任脉）、破 51 形态（0~50 点能量）、
    绝脉 15 形态（1~15 层）——段氏靠回能技能回能、引窍打出伤害，需逐形态可选
  - 其他门派同类：布泽/叠刃/悬象/流血/入月/鬼宿/破绽/缠绞/勾线(40级)/展缓(99级) 等
- **显示名语义化**：同名多形态优先用数据源 comment 生成 `名·形态` 后缀
  （如 `引窍·50点任脉`、`相依·1段`），无 comment 退回 `名(lv{N})`
- `is_series()` 函数删除

### 后端（jpcg_api / jpcg_core）
- `SkillPoolItemDTO` 新增 `sub_id` 字段（同 skill_id 不同形态区分），
  `host/skill.rs` load_skill_pool 与 `host/calc.rs` skill_dto_to_skilltype 透传
- 预设 `ComboStep` 新增 `sub_id`（serde default，旧存档兼容），
  conv.rs 预设往返保留 sub_id

### 前端（examples/jpcg_app）
- `ComboPage` 技能池：
  - **搜索框**：按技能名实时过滤（如搜"0点任脉"定位形态）
  - **折叠分组**：按基础技能名（去掉 `·形态` 后缀）分组，组头显示形态数，
    可展开/折叠，支持全部展开/折叠
  - **收藏/React key 唯一化**：改用 `skill_id-sub_id` 复合 key，
    修复同 id 多形态（相依 lv3/lv4）收藏与渲染混叠
  - 旧收藏 localStorage 数据兼容转换
- types/index.ts 补 `sub_id`；commands.ts mock 补形态示例（引窍 0/50/100 点任脉等）

## 验证
- 27 池重生成：zhoutian 196 条（引窍 101/绝脉 15/破 51 形态命名正确）、
  10821 勾线 100 条、10175 展缓 99 条、dot 技能保留
- FFI：load_skill_pool 返回 sub_id + 各形态 atk_xishu 正确
- FFI 连招回归：引窍·0点任脉 q=46.5万 vs 引窍·100点任脉 q=232.7万，
  final_kill_prob 计算正常
- `cargo test --workspace` 全过（16 金标准 + 各 crate）
- 前端 `npx tsc --noEmit` + `npm run build` 通过

## 遗留
- 骤风令：已解决（数据源更新推入，见 changes/2026-08-14-163023.md）
- `雾刃飞光·雾/刃/飞/光`、`劈风令·骤` 等名称本身带 `·` 的技能会被归入基础名组
  （如"雾刃飞光"组），语义上属于关联技能折叠，可接受

### 来自 changes/2026-08-14-163023.md

# 2026-08-14 163023 — 数据源更新（骤风令）+ 无质批量标注 + 追加真伤（已损失生命值）

## 背景

IcyTide/Generator 推送更新：段氏「骤风令」本体进入 skills.json（原缺失，仅 37779/37804
名为「横驱风靡」的旧条目），并**成批给无质技能打上 `critical_strike/critical_power = "0"`
占位标记**（此前仅剑飞惊天 1 个）。数据源结构同步变化：critical 从 node 层移入
damages 数组元素内（node 层不再有 critical 对象）。

## 变更

### 数据源脚本与文档
- 新增 `server_tools/json-to-toml/fetch-json.sh`：curl 拉取 IcyTide/Generator
  `assets/json/{skills,dots,belongs}.json` → `data/raw-src/`（已加入 .gitignore，原始
  JSON 不入库），末尾打印转换命令提示
- `data/shuxing/datamake.md`：新增「数据源获取」小节；转换器规则更新（新结构/
  无质 0/O 判定/追加真伤/rename）

### 转换器（json-to-toml）
- **适配新结构**：critical_strike/critical_power 从 damages[0] 提取（兼容旧 node.critical）
- **无质 0/O 规则**：cs/cp 任一为 `"0"/"O"` 占位 → `has_critical_strike = true`
  （数据源官方标注，约 19 技能/65 形态：果报/众境/剑飞惊天/临源/令怖/圣裁/崇光斩恶/
  涤罪/纵遇善缘/断马摧城/相依/不愧君/斩长鲸/镇星入舆/领胡 等）；
  overrides `wuzhi` 名单保留兜底（涤罪/临源 补入名单）
- **追加真伤形态展开**：overrides 新增 `lost_hp_zhenshi: [{name, per_layer, max_layers}]`，
  命中技能展开为 破绽0..=N 层形态并写 `lost_hp_zhenshishanghai` 字段
  （怒锋倾涛：单持/双持 × 破绽0~3层 = 8 形态，每层 6% 已损失生命值）
- **rename 能力**：overrides 新增 `rename` 映射，数据源名 → 显示名
  （10786「骤风」→「骤风令」，对齐 belongs.json 名）
- **sid 级 name 计数**：同名跨 sub（单持/双持等多 sub 同 sid）也能正确加 comment 后缀

### 引擎追加真伤（怒锋倾涛破绽机制）
- `Skilltype.lost_hp_zhenshishanghai: f32`（追加真伤 = 目标已损失生命值 × 系数，无视防御）
- `kill_prob.rs` 动态结算：`追加 = max(target_hp - 前序累计期望 - 本步主Q, 0) × 系数`
  ——**确定性伤害只加期望不加方差**（与无质一致），连招中随血量损耗几何收敛
  （追加本身也计入已损失），击杀 CDF 框架不变
- `ComboStepResult.lost_hp_zhenshi_damage: f64` 暴露每步追加值
- 单技能面板（q_cal）不含追加（满血目标追加为 0，正确语义）
- DTO 全链路透传：jpcg_api（SkillPoolItemDTO/SkillResultDTO/ComboStepResultDTO/
  SkillEditorItemDTO）→ host conv/skill/calc → 前端 types/index.ts +
  SkillEditorPage 新增「追加真伤(已损失×系数)」输入框

### 测试
- 新增金标准 `combo_lost_hp_zhenshi_dynamic`：首击追加 = 已损失(主Q)×系数、
  追加几何收敛、确定性方差 0、8 连击击穿目标血且不超杀
- `cargo test --workspace` 全过（17+1）；前端 tsc + build 通过

## 数值验证（FFI，怒锋倾涛·单持·破绽3层，目标 200 万血）

```
Q=152535 追加=332544  累计=485079   击杀率 0.0%
Q=152535 追加=245230  累计=882843   击杀率 0.0%
Q=152535 追加=173632  累计=1209010  击杀率 0.0%
Q=152535 追加=114922  累计=1476467  击杀率 0.0%
Q=152535 追加=66780   累计=1695782  击杀率 0.1%
Q=152535 追加=27303   累计=1875620  击杀率 12.2%
Q=152535 追加=0       累计=2028155  击杀率 59.7%
Q=152535 追加=0       累计=2180690  击杀率 92.9%
```

## 决策记录
- 追加结算采用「主伤害扣血后的一次性已损失」（不迭代）：`(target_hp - 前序 - 主Q) × 系数`
- 无质判定：数据源 0/O 占位为主，overrides 名单兜底（不相依依赖代码层 0/O 猜测）
- 装备特效池（池 0）的 custom_damage_base 真伤 23 条不并入职业池（维持现状）

## 遗留
- 其余职业 atk_up/pofang_up 仍为 1.0 占位（需逐职业实测校准）
- 池 0 装备特效（巽/御·击破/刃凌等真伤）未纳入职业技能池

### 来自 changes/2026-08-14-205054.md

# 2026-08-14 205054 — 恢复被误删的周天功/莫问数据文件

## 背景

数据迁移（编号 TOML 化）后 `data/shuxing/zhoutian.toml` 与 `data/shuxing/mowen.toml`
从工作区被删除（git 状态 D），但这两个池无编号文件替代，导致周天功/莫问数据缺失。

## 原因

- 周天功 = 数据源池 **10786**，莫问 = 池 **10447**，但 `overrides.json` 为二者显式配置
  `file: "zhoutian.toml"` / `"mowen.toml"`（沿用历史名，datamake.md 已有说明），
  转换器 `file_name = ov.file 或默认 <池id>.toml`，因此它们不会生成编号文件。
- 最后一次转换运行未覆盖这两个池（或生成后被清理）。

## 变更

- 重跑转换器恢复两个文件：
  ```sh
  cargo run -p json-to-toml -- --skills data/raw-src/skills.json \
      --dots data/raw-src/dots.json \
      --overrides server_tools/json-to-toml/overrides.json \
      --out data/shuxing --xinfa 10786 --xinfa 10447
  ```
  - `zhoutian.toml`：周天功/元气（atk_up 1.96 / pofang_up 0.3），196 条技能
    （含引窍·0~100点任脉 全形态、骤风→骤风令 rename）
  - `mowen.toml`：莫问/根骨（atk_up 1.96 / pofang_up 2.0），61 条技能
    （相依·1段/2段 等标无质）

## 验证

- `tomllib` 解析 OK；`cargo test -p jpcg_core` 20 项全过
- FFI `list_professions` 27 个心法含 `zhoutian`(周天功) + `mowen`(莫问)
- Python demo（JPCG_DATA_DIR=./data）端到端计算通过

## 留待处理

- 其余缺失池（10028 离经易道 / 10080 云裳心经 / 10176 补天诀 / 10448 相知 /
  10626 灵素）为治疗心法，按 datamake.md 约定跳过，无需生成
- 池 0（婆罗门/装备特效）按既有决策不并入职业技能池

### 来自 changes/2026-08-15-143820.md

# 修复武器伤害双算：AtkConfig::total() 只读基础攻击（分山劲断马摧城实测校准）

## 现象
分山劲断马摧城（无质）实测 536939，计算结果 61 万，偏高约 13.6%。

## 根因
`b_cal()` 使用 `AtkConfig::total()` = 基础攻击 **+ 武器伤害**，
`i_cal()` 又叠加 `武器×watk_xishu/100` 路径：

```
x = base_atk + (基础攻击+武器)×atk_xishu + 武器×watk_xishu/100
```

武器伤害被乘上技能攻击系数重复计入（断马：12062×16.78125 ≈ 20.2 万 I 段 → Q 多算约 7 万）。
数据源公式武器伤害仅经 `watk_cof` 参与（watk_xishu=0 的技能完全不参与）。

复现验证：
- 修复前：断马摧城 Q = 609776（与用户报告 61 万一致）
- 修复后：Q = 540119（去掉武器双算后）；实测 536939 处于 base 120~126 随机浮动区间内（±2.4%），判定为同一次样本的正常浮动，不再校准

## 改动
- `crates/jpcg_core/src/type_set/player.rs`：`AtkConfig::total()` 只返回 `base`（不含武器伤害），注释说明武器单独经 watk_xishu 参与
- `server_tools/json-to-toml/overrides.json`：池 10390（分山劲）`atk_up` 1.0 → 1.88
- `data/shuxing/10390.toml`：[xinfa] `atk_up = 1.88`（与 overrides 一致）

## 验证
- 引擎（用户面板：基础属性 18790/基础攻击 58197/会心 53354/会效 0/破防 134470/武器 12062；目标外防 15071/御劲 5047/化劲 62113）：
  N=458271, H=793247, Q=540119
- `cargo test --workspace` 全部通过（core 20 项 + app 1 项），无回归
- `total()` 唯一调用点即 `b_cal()`；derivatives 武器导数本就只走 watk_xishu 路径，无需联动

## 遗留
- 其余 24 个池的 `atk_up` 目前均为 1.0 占位（仅 10390=1.88/10447=1.96/10786=1.96 经确认），待逐一实物核对后更新 overrides 并重跑转换器

### 来自 changes/2026-08-15-145232.md

# 内防修复后回填 atkcal 金标准期望值

## What

`engine/atkcal.rs` 的用例在上一轮修复（内防门派改用 `guo_nfangyu_with`）后，金标准（golden）测试仍持有旧期望数值。本轮:

1. 统一 golden 用例的 skill 为 `&Skilltype`。
2. 用临时 `golden_dump_tmp` 测试 dump 出新数值，回填 6 组 golden 用例 + `combo_wuzhi` 用例的 `y/b/i/n/h/q` 与全部 6 项导数；dump 测试跑完即删。
3. `cargo fmt -p jpcg_core` 格式化受影响代码块。

## Why

Q 段数值依赖内防/外防分支，修复后各用例期望值整体下移（如 gong_default q=95725→91768、y=1355→1299）。不更新则 golden 全红且无法发现后续回归。

## Validation

- `cargo test --workspace` 全绿（atkcal 12 个 golden/combo 用例通过）。
- `cargo fmt -p jpcg_core --check` 通过。

## Notes

- 数值回填采用「dump→回填」而非手工推算，与「金标准不改行为」原则一致。
- 导数与 buff 有无无关（与修复前一致，非回归）。

### 来自 changes/2026-08-15-150000.md

# 版本号 bump：2.1.0-alpha.1 → 2.1.0-alpha.2

## What

- 根 workspace 版本（core 版本源）升至 `2.1.0-alpha.2`。
- `scripts/sync-version.sh` 同步 package.json / tauri.conf.json / 前端模拟版本串。

## Why

随 PR #6 累积修复（武器双算 + 内防分支 + 金标准回填）后发版前的最小版本步进。

### 来自 changes/2026-08-15-212429.md

# 创建 jpcg_combo 连招引擎 crate（排轴器后端重建，MC 击杀率 + hp 追加真伤）

日期：2026-08-15

## 背景
排轴器（ComboPage）伤害偏小 + dot 技能只算单跳 + 击杀率正态近似不可靠。此前讨论决定：
新建 `jpcg_combo` crate 承接连招编排，单技能公式/追加真伤公式留在 core（单源），
击杀率改用蒙特卡洛（50k 采样），hp 追加真伤按路径实际血量结算。

## 变更
### 新 crate `crates/jpcg_combo/`（cdylib + rlib）
- `engine.rs`：双通道模型
  - 期望通道：g/h/q/dot 期望 + 累计期望/方差（DTO 语义与迁移前一致）；追加真伤只加期望
  - 蒙特卡洛通道：`samples=50_000`（`DEFAULT_SAMPLES`，`ComboConfig` 可注入种子/采样数），
    每条路径实时血量推进，dot **逐跳独立**会心判定，击杀率 = 击杀路径占比，确定性无质连招精确
- `host.rs`：DTO 层计算入口 + 预设 CRUD + 导出/导入（存储委托 `jpcg_core::combo_io`/`config_io`）
- `conv.rs`：`skill_dto_to_skilltype` 等转换；ComboStep/ComboPreset 的 From 因孤儿规则留在 core
- `ffi.rs`：句柄 + JSON 协议（`jpcg_combo_call` 等，ABI_VERSION=1，与 core 一致）
- 测试：无质零方差、固定种子可复现、MC 击杀行为、真伤递增、dot 逐跳独立方差

### jpcg_core 收敛（删除）
- `engine/kill_prob.rs`、`host/combo.rs`、`engine/mod.rs`/`host/mod.rs` 的 mod 声明
- `ffi.rs` 7 个 combo 方法分支、`lib.rs` `calculate::start_combo` facade
- `atkcal.rs` golden 的两个 combo 测试迁至 jpcg_combo（新语义重写）

### 追加真伤语义（修复旧 bug）
- 旧：`lost=(target-(cum+mean)).max(0)` 把剩余血量当已损失 → 击杀率≠0 时真伤错误（用户报告的根源）
- 新（语义 A，core `lost_hp_append` 实现）：`lost = (max_hp - 结算后剩余).max(0) × 系数`，
  真伤同样扣血 → 期望通道递增（斩杀机制），MC 逐路径实时结算

### 修复会心率双计 bug
- 旧 kill_prob `crit_rate = guo_huixin() + huixin_up/100 + buff.huixin_pct/100`（重复计入 buff）
- 新：与 q_cal 一致 `guo_huixin() + huixin_up/100`

### DTO dot 字段链路（dot 只算单跳的根因）
- `SkillPoolItemDTO` 补 `dot_interval/dot_duration/dot_up/wushijianshang/zhenshishanghai`
- `load_skill_pool`、`skilltype_to_pool_item`、combo `skill_dto_to_skilltype` 全链路映射

### Tauri 双模式
- static 直调 `jpcg_combo::host`；dynamic 经 `ffi_bridge` 新增 `COMBO` OnceLock 加载
  `libjpcg_combo`（`call_combo*` 系列）；`get_module_versions` 增加 `combo` 字段

### 前端
- hostile 表单加「目标最大血量/当前血量」（0 = 未提供，回退 target_hp 满血模型）
- `HostileConfigDTO`/`ModuleVersions` 类型、`HOSTILE_FIELDS`、normalize 同步

### Makefile
- `build-modules`/`modules-dir` 纳入 jpcg_combo（dll 不进 modules_manifest 更新体系）

## 决策记录
- combo 依赖 jpcg_core（无环依赖）；损失公式唯一实现在 core（combo 经 `lost_hp_append` 调用）
- 无 hp 输入时（target_hp 也为 0）击杀率恒 1（迁移前行为）
- dot 的技能级 g/h 按 `dot_jumps` 等比因子拆分为逐跳 g/h（与期望通道同源），逐跳独立采样
- jpcg_combo 版本继承 workspace（2.1.0-alpha.2），不进 modules_manifest

## 验证
- `cargo test --workspace` 全绿；jpcg_combo 11 测试；dynamic e2e 冒烟绿
- `cargo build`（static+dynamic 双模式）、`cargo fmt --all --check`、前端 `npm run build` 绿

### 来自 changes/2026-08-18-140250.md

# 修复 macOS 技能编辑器底部裁剪

## 变更

- 让技能编辑页作为应用主区域的 flex 子项占用顶部工具栏之外的剩余高度。
- 允许右侧技能详情面板在受限高度内收缩，并由自身滚动显示完整编辑表单。

## 原因

技能编辑页原先使用 `height: 100%`，但它位于带顶部工具栏的纵向 flex 容器中，实际高度会叠加在工具栏之后。主容器的 `overflow: hidden` 会裁掉详情面板底部，导致 macOS 窗口中最后的字段无法滚动到可见区域。

## 决策

使用 `flex: 1` 与 `min-height: 0` 修正容器约束，保留现有详情面板内部滚动和页面布局，不改变字段、数据协议或保存逻辑。

### 来自 changes/2026-08-30-173635.md

# 修复全 workspace clippy 警告 + README 编译步骤

日期：2026-08-30

## 背景
dev 合并前需通过 CI（clippy 虽不 -D warnings，但要求清零）。同时更新 README 增加编译/构建步骤。

## 变更
### README
- 版本头更新 v2.1.0-alpha.2（2026年8月）
- 击杀概率曲线：CLT 正态 → 蒙特卡洛 50k 路径（示意代码）
- 目标设定输入项补「目标最大/当前血量」
- 主题描述修正为薄荷/松石绿；Roadmap 标记 AutoDiff 已实现
- 新增「🔧 编译与运行」章节（快速开始 / Tauri / 双构建模式 / 打包 / check-all / workspace 结构）

### clippy 清零（19 文件，~50 警告）
- `needless_borrow`×11：atkcal `&self.coeff` → `self.coeff`
- `collapsible_if`×多处：合并嵌套 if / `&& let`
- `missing_safety_doc`×9：core/update/const 的 FFI 函数补 `# Safety`
- 删除死代码 `default()`×3：player/hostilepile/xinfa（无调用方，与 Default trait 冲突）
- `manual_is_multiple_of`：`y.is_multiple_of(...)`
- `excessive_precision`×2：golden 字面量截断 `1.776_041_6`（f32 等值，行为不变）
- `ptr_arg`：`is_string_empty(&String)` 加 `#[allow]`（serde skip_serializing_if 需该签名）
- `map_or`→`is_some_and`、`for_kv_map`→`keys()`、`let_unit_value`、`match_single_binding`、`bind_instead_of_map`、`match_result_ok`
- `too_many_arguments`（download.rs 公开 API）、`type_complexity`（derivatives）→ 加 `#[allow]`/type 别名

## 决策
- `default()` 死代码直接删除（经全 workspace 搜索确认无调用方），非加 allow
- `is_string_empty` 因 serde 属性签名需 `&String`，用 `#[allow(clippy::ptr_arg)]` 保留

## 验证
- `cargo clippy --workspace --all-targets` 0 警告
- `cargo test --workspace` 全绿（金标准 15 + combo 11 不因字面量截断变值）
- `cargo fmt --all --check`、`make check-all`、前端 `npm run build` 全绿

### 来自 changes/2026-08-30-230349.md

# 等级常数编译期 const 模块（jpcg_const::level_constant）取代 CoefficientConfig 默认并补御劲

日期：2026-09-04（收尾）

## 背景与决策
- 目标模块定位：**常数计算模块，编译时由预设 TOML 解析并固化为 `const` 全局**，
  为后续"等级 → 常数"关系逆向（靠 level 推出所有常数）铺路。
- 弃用先前 WIP 的"运行时经 `data_dir()` 读 `normal_num` TOML"方案（引入 core 反向依赖 + 运行时 I/O）。
- 改为：`include_str!` 内嵌预设 + `const fn` 白名单解析，运行时零依赖、零 I/O；
  `level` 只作快照记录（`LEVEL`，预计算用），不进常量结构。
- 引擎 `CoefficientConfig` 仅保留"可配置载体"角色：字段集合补上缺失的御劲分母
  （原 `guo_yujin_huixiao_with` 错误复用 `huixin_xishu`，御劲会伤减免分母 55123.2 从未进入配置），
  默认值单一来源 = 本 const 模块；固定公式方法统一委托 `*_with(CoefficientConfig::default())`，
  引擎各处硬编码分母（126007.2/30115.8/55123.2/197703/72844.2/225957.6）全部移除。

## 变更
### jpcg_const（叶子 crate，运行时依赖清零）
- `preset/level_constant.toml`（新）：130 级·第 3 赛季快照
  （pofang/huixin/huixiao/yujin/yuhui/huajin/fangyu 分母 + pvp_global_jianshang）
- `src/level_constant.rs`（新）：`LevelConstant` 结构 + `LEVEL`/`CURRENT` const；
  `const fn parse_snapshot` 逐行白名单解析——未知 key/缺字段/坏值 → 编译期报错（防漂移）；
  f32 十进制用"整数尾数÷10^k"解析，与 Rust 字面量 bit 完全一致（有单测锁定）；
  const fn 受限说明：不支持切片，全程以 (bytes,start,end) 偏移扫描。
  测试 4 个：bit 一致 / 全量 roundtrip / 未知 key panic / 缺字段 panic。
- `lib.rs`：模块改挂 `level_constant`；删除 `player.rs`（运行时读取器，整文件删除）
- `Cargo.toml`：移除 jpcg_core/serde/toml/tempfile 全部依赖（含 dev-deps）

### jpcg_api
- `CoefficientConfigDTO` 新增 `yujin_xishu` / `yuhui_xishu`（`#[serde(default)]`，旧存档兼容）

### jpcg_core
- `Cargo.toml` 新增 `jpcg_const` 依赖；`lib.rs` 还原 `pub mod store` → `mod store`
- `type_set/coefficient.rs`：字段 +御劲两项；`Default` 取自 `jpcg_const::level_constant::CURRENT`；
  新增 `From<&CoefficientConfigDTO>`（分母 0/缺失 → 真源默认；pvp 减伤 0 语义合法不回退）
- `type_set/player.rs` / `hostilepile.rs`：固定公式改为委托 `*_with(&CoefficientConfig::default())`；
  `guo_yujin_huixiao_with` 改用 `yuhui_xishu`（55123.2，修复错接）、`guo_yujin_huixin_with` 改用 `yujin_xishu`
- `host/calc.rs`、`host/config.rs`：DTO→core 走 `CoefficientConfig::from`（旧档/缺字段安全回退）

### jpcg_combo
- `host.rs` conv 改用 `CoefficientConfig::from`；测试系数字面量补齐新字段
- `engine.rs` 无质期望回填：御劲会伤减免修正后 91768→90651、183536→181302（hostile yujin=5047）

### core 金标准
- `atkcal.rs` golden 7 组（gong/wei/zheng × default/buff + gong_wuzhi）H/Q/导数按新引擎回填
  （变化方向 = 御劲会伤减免增强，仅 yujin>0 目标生效）

### 前端（examples/jpcg_app）
- `types/index.ts`、`utils/constants.ts`（COEFFICIENT_FIELDS + DEFAULT_COEFFICIENT）、
  `utils/normalize.ts` 同步新增字段；其余经 `...DEFAULT_COEFFICIENT` 展开自动携带

## 验证
- `cargo fmt --all -- --check` 干净
- `cargo clippy -p jpcg_const -p jpcg_api -p jpcg_core -p jpcg_combo --all-targets` 零警告
- `cargo test -p jpcg_const`（4）/ `-p jpcg_core` / `-p jpcg_combo` 全部通过
- 前端 `tsc --noEmit` 通过
- python demo / FFI JSON 无需改动：缺字段零值由 `From` 回退真源默认

## 注意/待办
- 逆向进展后可扩展：预设增多级快照 → `[LevelConstant; N]` 表；关系已知后替换为
  `const fn from_level(level) -> LevelConstant` 推导（预设退化为系数）。
- `yujin_xishu` 当前与 `huixin_xishu` 同值(197703)，是否同一游戏常量待逆向确认；
  已拆独立字段避免届时改 schema。
- core 现编译期依赖 jpcg_const：动态模块更新语义下 core dll 会内嵌 const 数据副本，
  若未来要求"只更 const dll 即更新引擎默认系数"需另做运行时取数设计（当前阶段无此需求）。

### 来自 changes/2026-09-04-200540.md

# 修复 release.sh：独立版本降级中断 + prep 分支命名冲突

日期：2026-09-04

## 背景
首次执行 `scripts/release.sh beta`（2.1.0-beta.1）暴露两处缺陷：

1. `cargo set-version "$VERSION"` 会把 workspace 内**所有**成员都尝试改成新版本，
   但 `jpcg_const` 使用独立版本号（`130.3.20260602`，等级.赛季.日期），
   直接被判定 "Cannot downgrade from 130.3.20260602 to 2.1.0-beta.1" 中断脚本。
2. prep 分支名 `release/prep-${VERSION}` 与已有分支 `release` 冲突
   （GitHub 禁止 `release/xxx` 与分支 `release` 共存），push 被拒。

## 变更
- `scripts/release.sh`：
  - 版本 bump 改为**仅改根 `[workspace.package]` version**（继承成员自动生效，
    独立版本 crate 不动），随后 `sync-version.sh` 同步前端/tauri/模拟串，
    再 `cargo metadata` 刷新 Cargo.lock
  - prep 分支改为 `prep/${VERSION}`，避开 `release/` 命名空间
- `scripts/README.md`：同步两处描述

## 验证
- beta.1 实际发布时手工等价步骤跑通（根版本改 + sync-version + lock 刷新 + prep/v2.1.0-beta.1 + tag）
- 本修复让 `release.sh beta` 可全自动复现

## 注意
- 更新器 `jpcg_updater` 同样为独立版本（2.1.0），不受 workspace bump 影响

### 来自 changes/2026-09-04-210000.md

# 修复 release.yml：Windows runner 两步缺 shell: bash

日期：2026-09-04

## 背景
`v2.1.0-beta.1` 触发 release.yml 时 windows-latest job 在"判断更新通道"步骤失败：
该步（及"提取版本号"）未指定 `shell: bash`，Windows runner 默认用 PowerShell，
`[[ "$GITHUB_REF_NAME" == *-beta* ]]` 为 bash 语法 → 报错，导致三平台只产出 darwin/linux 资产。

## 变更
- `.github/workflows/release.yml`：为"提取版本号（tag）"与"判断更新通道"两步显式加 `shell: bash`

## 验证
- 失败 run 335...（33870657978）三 job 结论：darwin/linux success、windows 唯一失败步 = 判断更新通道
- 修复后重建 tag 重跑，windows 应产出 app exe / dll / 清单

### 来自 changes/2026-09-04-220000.md

# feat: 以 GitHub Release 为载具的更新自动部署（deploy-gen + package job + 模块平台过滤）

日期：2026-09-04

## 背景
beta.1（v2.1.0-beta.1）发布与服务器落盘全程手工。目标：tag 构建成功后自动产出
服务器可直接应用的"通道 tgz"，随 Release 推送；服务器侧以 webhook 拉取落位
（服务器接收端为独立私有仓库实现，不随本仓库发布）。

## 变更（本仓库 jpcg）
- 新增 `server_tools/deploy-gen`（workspace 成员）：
  - 输入：GitHub Release 资产目录 + 仓库 data + version/channel
  - 输出：更新服务器 downloads 根布局 stage（beta 根式 / stable 版本目录式）
  - 生成 update.toml / manifest.toml（app 二进制 sha256）/ 三平台合并
    modules_manifest.toml（platform="multi"，core/update 版本随 app、
    const 反查自身 Cargo.toml 独立版本）/ data 布局（按 data_manifest 列出的文件拷贝）
  - beta 与 stable 双通道本地试跑通过，与 beta.1 手工部署布局一致
- `.github/workflows/release.yml`：`deploy`(SSH 直推) 方案废弃，改为新增
  `package` job（needs: build，仅 tag push）：下载资产 → deploy-gen → tar 为
  `jpcg-<channel>-<tag>.tgz` 并附 `.sha256` → `gh release upload` 到同一 Release
- `crates/jpcg_update/src/modules.rs`：模块更新按本机平台扩展名过滤
  （mac=.dylib / linux=.so / windows=.dll），适配多平台合并清单，避免下载无关 dll

## 验证
- `cargo build -p deploy-gen` / beta、stable 双通道试跑通过
- `cargo check -p jpcg_update` 通过

## 待办（服务器接收端，独立私有仓库，不在本仓库实现）
- 服务器 webhook 接收（验签）→ 拉取该 tgz → 校验 .sha256 →
  原子切换（beta 覆写根 / stable 合并保留 3 版）
- 下载服务目录列表处理（stable 客户端依赖 /updates/JPCG/ 的 HTML 目录列表）

### 来自 changes/2026-09-08-214000.md

# fix: 更新检查/执行无限等待 + Tauri v2 中 window.confirm 失效导致无法确认更新

日期：2026-09-08

## 背景
- beta.1 构建中点击"检查更新"卡在"正在检查更新..."不返回（Beta 开关开/关都卡）。
  服务器侧所有被请求 URL 均 200，定位为客户端代码层。
- 修复后 beta 仍"点了无反应"：Tauri v2 的 WebView **禁用 window.confirm**（恒返 false），
  检测到新版本时的确认框被当"取消"处理 → 界面直接复位、既不更新也不提示。

## 变更
- `crates/jpcg_update/src/download.rs`：新增 `http_client()`（统一 60s 超时），
  全部 5 处 `reqwest::Client::new()` 改用之
- `crates/jpcg_update/src/modules.rs`：模块清单请求同样走带超时客户端
- `examples/jpcg_app/src-tauri/src/commands/update.rs`：`check_update` / `perform_update` /
  `perform_app_update` / `perform_modules_update` 同步实现移入
  `tauri::async_runtime::spawn_blocking`，避免在 async 命令线程内嵌套新建 tokio runtime
- `examples/jpcg_app/src/components/ConfigPanel.tsx`：废弃 `window.confirm`，改为应用内
  确认区（发现应用/模块更新时显示"下载更新/取消"按钮），确认后走原下载→验证→重启流程。
  补充状态修正：命中确认时 `updating` 保持 true（禁用重复触发）、进度框与确认框互斥避免
  叠显；取消有 toast 反馈；确保点"下载更新"后进度条可见（否则又像"无反应"）

## 验证
- `cargo check -p jpcg_update` / `-p jpcg_app` 通过；前端 `tsc --noEmit` 通过
- 效果：任何网络卡点 60s 内转为明确报错；命令不再无界等待；更新确认不再被静默吞掉

### 来自 changes/2026-09-08-223000.md

# fix: macOS 产物架构错标 + 全面切到 Apple Silicon（aarch64）单架构

日期：2026-09-08

## 背景
- GitHub 标准 macOS runner 已全部迁移到 arm64（macOS 26，Intel 已被官方淘汰），
  `macos-latest` 上 `cargo build` 产出的就是原生 aarch64 二进制，但此前产物名与清单
  一直写死 `jpcg-app-darwin-x86_64` / `arch = "x86_64"`——**错标**。
- 更隐蔽的是 manifest `os` 字段写的是 `"darwin"`，而客户端按
  `std::env::consts::OS` 比对（= `"macos"`），导致 macOS 客户端永远匹配不到二进制，
  报 "未找到适用于 macos aarch64 的二进制文件"（Apple Silicon 上必现）。

## 变更
- `.github/workflows/release.yml`：matrix 增 `arch`；mac 资产命名为
  `jpcg-app-macos-aarch64`，linux/windows 保持 `x86_64`；命名不再硬编码
- `server_tools/deploy-gen/src/main.rs`：`APP_BINS` → macos/aarch64，清单 `os` 字段
  用客户端词汇（`macos` 而非 `darwin`）
- `server_tools/manifest-gen/src/main.rs`：`detect_platform` darwin → macos
- `server_manifest.md`：目录/清单示例修正为实际 `{os}-{arch}` 命名，去掉 Intel mac

## 决策
- **不保留 Intel macOS**：苹果已淘汰全部 Intel Mac，不为旧机型维护 x86_64 构建；
  每平台单架构，模块 dylib 亦无同名冲突
- 产物命名/清单 `os+arch` 与客户端 `std::env::consts::{OS,ARCH}` 严格对齐

## 验证
- 本机（arm64）dev 构建此前因清单缺 macos/aarch64 命中报错；beta.2 发布后此路径应打通

### 来自 changes/2026-09-08-223500.md

# chore: dev 补齐 release.sh 修复（#11）+ 清理已聚合 changes

日期：2026-09-08

## 背景
- dev 分支此前缺失 #11 的 `scripts/release.sh` 修复（仅 beta 有）：dev 版仍是
  `cargo set-version`（会因 jpcg_const 独立版本降级而中断）+ `release/prep-` 分支名
  （与已有 `release` 分支冲突）。dev 作为最上游分支必须与修复同步，避免后续提升把
  未修版脚本带回 beta/release。
- `changes/` 累积了 56 个已聚合进 beta 分支 `[2.1.0-beta.1]` CHANGELOG 的旧条目，
  若不清除，下次发布会把这些历史再聚合一遍（重复）。

## 变更
- `scripts/release.sh` / `scripts/README.md`：从 beta 同步 #11 修复
  （版本 bump 仅改根 `[workspace.package]` + `sync-version.sh` + `cargo metadata`
  刷 lock；prep 分支名改 `prep/<版本>`）
- `changes/`：删除 56 个已聚合条目，保留未发布条目
  （2026-09-04-200540/210000/220000、2026-09-08-214000 与本仓库内新条目）

## 验证
- 与 origin/beta 对比 `scripts/` 差异为空（完全对齐）

### 来自 changes/2026-09-08-231500.md

# fix: release.sh 在 set -u 下 `$VAR` 紧跟全角标点触发 unbound variable

日期：2026-09-08

## 背景
`scripts/release.sh` 第 7 步 `gh pr create` 的 `--body` 及末尾提示语中，
`$STAGE` / `$TARGET_BRANCH` 后面紧跟全角字符（`，`、`。`）。在 `set -u`
（`set -euo pipefail`）下，bash 会把多字节字符的首字节并入变量名，导致
`STAGE�: unbound variable` 报错，脚本在 beta.2 发布时于第 7/8 步中断
（commit + tag 已生成、prep 分支已推，但 PR 与 tag 推送需手工补做）。

## 变更
- `scripts/release.sh`：`$STAGE，` → `${STAGE}，`；`$TARGET_BRANCH。` →
  `${TARGET_BRANCH}。`（用花括号显式界定变量名，避免紧跟多字节字符被并入）

## 验证
- 复现：`bash -uc 'STAGE=beta; x="…（$STAGE，…"'` 报 `STAGE�: unbound variable`；
  加花括号后通过
- 已用于 beta.2 发布的手工补做（PR + tag 推送）

### 来自 changes/2026-09-08-233000.md

# fix: 更新 URL 拼接缺分隔符（beta/stable/all_updates 三处 404）

日期：2026-09-08

## 背景
beta 通道点"下载更新"报 `下载失败: 下载文件失败，HTTP 状态码: 404 Not Found`。

根因：`fetch_app_update_info` / `all_updates` 里用
`format!("{}{}", base_url.trim_end_matches('/'), path)` 直接拼接，
而 base trim 后无结尾 `/`、path 也无前导 `/`，导致：
- beta：`.../updates/JPCG_beta` + `jpcg-app-macos-aarch64`
  → `.../JPCG_betajpcg-app-...`
- stable：`.../updates/JPCG` + `v2.1.0` + `/...`
  → `.../JPCGv2.1.0/jpcg-app-...`
- `all_updates` 的 `version_url`：`.../JPCG` + `v2.1.0/`
  → `.../JPCGv2.1.0/`

3 处均为同一类漏分隔符问题（stable/CLI 通道此前未被实际使用，故只有 beta 显现）。

## 变更
- `crates/jpcg_update/src/lib.rs`：
  - 新增 `join_url(base, segments)` 统一拼接（base 去尾 `/`，各段去首尾 `/` 后以 `/` 连接）
  - 三处（beta 二进制、stable 版本目录、all_updates version_url）改用 `join_url`
  - 补 3 个单测覆盖 beta/stable/缺斜杠容错

## 验证
- `cargo test -p jpcg_update --lib`：3 passed
- 修复后 URL = `https://nefinita-ai.com/updates/JPCG_beta/jpcg-app-macos-aarch64`（服务器实际 200）

### 来自 changes/2026-09-08-234500.md

# fix: Release 产物用 tauri build 嵌入前端（裸 cargo build 产物白屏）

日期：2026-09-08

## 背景
beta.2 更新到新版后**白屏**。排查：
- 浏览器 mock / `tauri dev` 前端正常 → 排除 JS 回归
- 本地 `cargo build -p jpcg_app --release` 产物直接运行也白屏
- 对比二进制内容：裸 cargo build 产物搜不到前端资源（如 `index-*.js`）；
  同一份源码用 `npx tauri build --no-bundle` 产物含 `index-*.js`

结论：Tauri v2 前端资源由 `tauri` CLI 构建流程嵌入二进制；
只跑 `cargo build` 不嵌入 frontendDist → 产物无 UI，被 updater 替换后白屏。

## 变更
- `.github/workflows/release.yml`：构建 app 改为
  `npx tauri build --no-bundle`（在 examples/jpcg_app 下，产物仍落在
  target/release/jpcg_app），updater 仍用 `cargo build -p jpcg_updater`

## 验证
- 本机对照：cargo build 产物无 `index-*.js`；tauri build 产物有（UI 正常）

## [2.1.0-beta.4] - 2026-10-11

### 来自 changes/2026-09-04-200540.md

# 修复 release.sh：独立版本降级中断 + prep 分支命名冲突

日期：2026-09-04

## 背景
首次执行 `scripts/release.sh beta`（2.1.0-beta.1）暴露两处缺陷：

1. `cargo set-version "$VERSION"` 会把 workspace 内**所有**成员都尝试改成新版本，
   但 `jpcg_const` 使用独立版本号（`130.3.20260602`，等级.赛季.日期），
   直接被判定 "Cannot downgrade from 130.3.20260602 to 2.1.0-beta.1" 中断脚本。
2. prep 分支名 `release/prep-${VERSION}` 与已有分支 `release` 冲突
   （GitHub 禁止 `release/xxx` 与分支 `release` 共存），push 被拒。

## 变更
- `scripts/release.sh`：
  - 版本 bump 改为**仅改根 `[workspace.package]` version**（继承成员自动生效，
    独立版本 crate 不动），随后 `sync-version.sh` 同步前端/tauri/模拟串，
    再 `cargo metadata` 刷新 Cargo.lock
  - prep 分支改为 `prep/${VERSION}`，避开 `release/` 命名空间
- `scripts/README.md`：同步两处描述

## 验证
- beta.1 实际发布时手工等价步骤跑通（根版本改 + sync-version + lock 刷新 + prep/v2.1.0-beta.1 + tag）
- 本修复让 `release.sh beta` 可全自动复现

## 注意
- 更新器 `jpcg_updater` 同样为独立版本（2.1.0），不受 workspace bump 影响

### 来自 changes/2026-09-04-210000.md

# 修复 release.yml：Windows runner 两步缺 shell: bash

日期：2026-09-04

## 背景
`v2.1.0-beta.1` 触发 release.yml 时 windows-latest job 在"判断更新通道"步骤失败：
该步（及"提取版本号"）未指定 `shell: bash`，Windows runner 默认用 PowerShell，
`[[ "$GITHUB_REF_NAME" == *-beta* ]]` 为 bash 语法 → 报错，导致三平台只产出 darwin/linux 资产。

## 变更
- `.github/workflows/release.yml`：为"提取版本号（tag）"与"判断更新通道"两步显式加 `shell: bash`

## 验证
- 失败 run 335...（33870657978）三 job 结论：darwin/linux success、windows 唯一失败步 = 判断更新通道
- 修复后重建 tag 重跑，windows 应产出 app exe / dll / 清单

### 来自 changes/2026-09-04-220000.md

# feat: 以 GitHub Release 为载具的更新自动部署（deploy-gen + package job + 模块平台过滤）

日期：2026-09-04

## 背景
beta.1（v2.1.0-beta.1）发布与服务器落盘全程手工。目标：tag 构建成功后自动产出
服务器可直接应用的"通道 tgz"，随 Release 推送；服务器侧以 webhook 拉取落位
（服务器接收端为独立私有仓库实现，不随本仓库发布）。

## 变更（本仓库 jpcg）
- 新增 `server_tools/deploy-gen`（workspace 成员）：
  - 输入：GitHub Release 资产目录 + 仓库 data + version/channel
  - 输出：更新服务器 downloads 根布局 stage（beta 根式 / stable 版本目录式）
  - 生成 update.toml / manifest.toml（app 二进制 sha256）/ 三平台合并
    modules_manifest.toml（platform="multi"，core/update 版本随 app、
    const 反查自身 Cargo.toml 独立版本）/ data 布局（按 data_manifest 列出的文件拷贝）
  - beta 与 stable 双通道本地试跑通过，与 beta.1 手工部署布局一致
- `.github/workflows/release.yml`：`deploy`(SSH 直推) 方案废弃，改为新增
  `package` job（needs: build，仅 tag push）：下载资产 → deploy-gen → tar 为
  `jpcg-<channel>-<tag>.tgz` 并附 `.sha256` → `gh release upload` 到同一 Release
- `crates/jpcg_update/src/modules.rs`：模块更新按本机平台扩展名过滤
  （mac=.dylib / linux=.so / windows=.dll），适配多平台合并清单，避免下载无关 dll

## 验证
- `cargo build -p deploy-gen` / beta、stable 双通道试跑通过
- `cargo check -p jpcg_update` 通过

## 待办（服务器接收端，独立私有仓库，不在本仓库实现）
- 服务器 webhook 接收（验签）→ 拉取该 tgz → 校验 .sha256 →
  原子切换（beta 覆写根 / stable 合并保留 3 版）
- 下载服务目录列表处理（stable 客户端依赖 /updates/JPCG/ 的 HTML 目录列表）

### 来自 changes/2026-09-08-214000.md

# fix: 更新检查/执行无限等待 + Tauri v2 中 window.confirm 失效导致无法确认更新

日期：2026-09-08

## 背景
- beta.1 构建中点击"检查更新"卡在"正在检查更新..."不返回（Beta 开关开/关都卡）。
  服务器侧所有被请求 URL 均 200，定位为客户端代码层。
- 修复后 beta 仍"点了无反应"：Tauri v2 的 WebView **禁用 window.confirm**（恒返 false），
  检测到新版本时的确认框被当"取消"处理 → 界面直接复位、既不更新也不提示。

## 变更
- `crates/jpcg_update/src/download.rs`：新增 `http_client()`（统一 60s 超时），
  全部 5 处 `reqwest::Client::new()` 改用之
- `crates/jpcg_update/src/modules.rs`：模块清单请求同样走带超时客户端
- `examples/jpcg_app/src-tauri/src/commands/update.rs`：`check_update` / `perform_update` /
  `perform_app_update` / `perform_modules_update` 同步实现移入
  `tauri::async_runtime::spawn_blocking`，避免在 async 命令线程内嵌套新建 tokio runtime
- `examples/jpcg_app/src/components/ConfigPanel.tsx`：废弃 `window.confirm`，改为应用内
  确认区（发现应用/模块更新时显示"下载更新/取消"按钮），确认后走原下载→验证→重启流程。
  补充状态修正：命中确认时 `updating` 保持 true（禁用重复触发）、进度框与确认框互斥避免
  叠显；取消有 toast 反馈；确保点"下载更新"后进度条可见（否则又像"无反应"）

## 验证
- `cargo check -p jpcg_update` / `-p jpcg_app` 通过；前端 `tsc --noEmit` 通过
- 效果：任何网络卡点 60s 内转为明确报错；命令不再无界等待；更新确认不再被静默吞掉

### 来自 changes/2026-09-08-223000.md

# fix: macOS 产物架构错标 + 全面切到 Apple Silicon（aarch64）单架构

日期：2026-09-08

## 背景
- GitHub 标准 macOS runner 已全部迁移到 arm64（macOS 26，Intel 已被官方淘汰），
  `macos-latest` 上 `cargo build` 产出的就是原生 aarch64 二进制，但此前产物名与清单
  一直写死 `jpcg-app-darwin-x86_64` / `arch = "x86_64"`——**错标**。
- 更隐蔽的是 manifest `os` 字段写的是 `"darwin"`，而客户端按
  `std::env::consts::OS` 比对（= `"macos"`），导致 macOS 客户端永远匹配不到二进制，
  报 "未找到适用于 macos aarch64 的二进制文件"（Apple Silicon 上必现）。

## 变更
- `.github/workflows/release.yml`：matrix 增 `arch`；mac 资产命名为
  `jpcg-app-macos-aarch64`，linux/windows 保持 `x86_64`；命名不再硬编码
- `server_tools/deploy-gen/src/main.rs`：`APP_BINS` → macos/aarch64，清单 `os` 字段
  用客户端词汇（`macos` 而非 `darwin`）
- `server_tools/manifest-gen/src/main.rs`：`detect_platform` darwin → macos
- `server_manifest.md`：目录/清单示例修正为实际 `{os}-{arch}` 命名，去掉 Intel mac

## 决策
- **不保留 Intel macOS**：苹果已淘汰全部 Intel Mac，不为旧机型维护 x86_64 构建；
  每平台单架构，模块 dylib 亦无同名冲突
- 产物命名/清单 `os+arch` 与客户端 `std::env::consts::{OS,ARCH}` 严格对齐

## 验证
- 本机（arm64）dev 构建此前因清单缺 macos/aarch64 命中报错；beta.2 发布后此路径应打通

### 来自 changes/2026-09-08-223500.md

# chore: dev 补齐 release.sh 修复（#11）+ 清理已聚合 changes

日期：2026-09-08

## 背景
- dev 分支此前缺失 #11 的 `scripts/release.sh` 修复（仅 beta 有）：dev 版仍是
  `cargo set-version`（会因 jpcg_const 独立版本降级而中断）+ `release/prep-` 分支名
  （与已有 `release` 分支冲突）。dev 作为最上游分支必须与修复同步，避免后续提升把
  未修版脚本带回 beta/release。
- `changes/` 累积了 56 个已聚合进 beta 分支 `[2.1.0-beta.1]` CHANGELOG 的旧条目，
  若不清除，下次发布会把这些历史再聚合一遍（重复）。

## 变更
- `scripts/release.sh` / `scripts/README.md`：从 beta 同步 #11 修复
  （版本 bump 仅改根 `[workspace.package]` + `sync-version.sh` + `cargo metadata`
  刷 lock；prep 分支名改 `prep/<版本>`）
- `changes/`：删除 56 个已聚合条目，保留未发布条目
  （2026-09-04-200540/210000/220000、2026-09-08-214000 与本仓库内新条目）

## 验证
- 与 origin/beta 对比 `scripts/` 差异为空（完全对齐）

### 来自 changes/2026-09-08-231500.md

# fix: release.sh 在 set -u 下 `$VAR` 紧跟全角标点触发 unbound variable

日期：2026-09-08

## 背景
`scripts/release.sh` 第 7 步 `gh pr create` 的 `--body` 及末尾提示语中，
`$STAGE` / `$TARGET_BRANCH` 后面紧跟全角字符（`，`、`。`）。在 `set -u`
（`set -euo pipefail`）下，bash 会把多字节字符的首字节并入变量名，导致
`STAGE�: unbound variable` 报错，脚本在 beta.2 发布时于第 7/8 步中断
（commit + tag 已生成、prep 分支已推，但 PR 与 tag 推送需手工补做）。

## 变更
- `scripts/release.sh`：`$STAGE，` → `${STAGE}，`；`$TARGET_BRANCH。` →
  `${TARGET_BRANCH}。`（用花括号显式界定变量名，避免紧跟多字节字符被并入）

## 验证
- 复现：`bash -uc 'STAGE=beta; x="…（$STAGE，…"'` 报 `STAGE�: unbound variable`；
  加花括号后通过
- 已用于 beta.2 发布的手工补做（PR + tag 推送）

### 来自 changes/2026-09-08-233000.md

# fix: 更新 URL 拼接缺分隔符（beta/stable/all_updates 三处 404）

日期：2026-09-08

## 背景
beta 通道点"下载更新"报 `下载失败: 下载文件失败，HTTP 状态码: 404 Not Found`。

根因：`fetch_app_update_info` / `all_updates` 里用
`format!("{}{}", base_url.trim_end_matches('/'), path)` 直接拼接，
而 base trim 后无结尾 `/`、path 也无前导 `/`，导致：
- beta：`.../updates/JPCG_beta` + `jpcg-app-macos-aarch64`
  → `.../JPCG_betajpcg-app-...`
- stable：`.../updates/JPCG` + `v2.1.0` + `/...`
  → `.../JPCGv2.1.0/jpcg-app-...`
- `all_updates` 的 `version_url`：`.../JPCG` + `v2.1.0/`
  → `.../JPCGv2.1.0/`

3 处均为同一类漏分隔符问题（stable/CLI 通道此前未被实际使用，故只有 beta 显现）。

## 变更
- `crates/jpcg_update/src/lib.rs`：
  - 新增 `join_url(base, segments)` 统一拼接（base 去尾 `/`，各段去首尾 `/` 后以 `/` 连接）
  - 三处（beta 二进制、stable 版本目录、all_updates version_url）改用 `join_url`
  - 补 3 个单测覆盖 beta/stable/缺斜杠容错

## 验证
- `cargo test -p jpcg_update --lib`：3 passed
- 修复后 URL = `https://nefinita-ai.com/updates/JPCG_beta/jpcg-app-macos-aarch64`（服务器实际 200）

### 来自 changes/2026-09-08-234500.md

# fix: Release 产物用 tauri build 嵌入前端（裸 cargo build 产物白屏）

日期：2026-09-08

## 背景
beta.2 更新到新版后**白屏**。排查：
- 浏览器 mock / `tauri dev` 前端正常 → 排除 JS 回归
- 本地 `cargo build -p jpcg_app --release` 产物直接运行也白屏
- 对比二进制内容：裸 cargo build 产物搜不到前端资源（如 `index-*.js`）；
  同一份源码用 `npx tauri build --no-bundle` 产物含 `index-*.js`

结论：Tauri v2 前端资源由 `tauri` CLI 构建流程嵌入二进制；
只跑 `cargo build` 不嵌入 frontendDist → 产物无 UI，被 updater 替换后白屏。

## 变更
- `.github/workflows/release.yml`：构建 app 改为
  `npx tauri build --no-bundle`（在 examples/jpcg_app 下，产物仍落在
  target/release/jpcg_app），updater 仍用 `cargo build -p jpcg_updater`

## 验证
- 本机对照：cargo build 产物无 `index-*.js`；tauri build 产物有（UI 正常）

### 来自 changes/2026-09-12-215500.md

# fix: 应用更新判定改用二进制版本（不再依赖 local_update_info.toml）

日期：2026-09-12

## 背景
更新到 beta.3 成功后，界面仍提示"发现新版本 v2.1.0-beta.3，是否下载并重启应用？"。
根因：`check_updates` 用 `local_update_info.toml`（CWD）里的 `version` 与服务器比对，
而 **GUI 更新路径（`jpcg_core::host::update::perform_app_update`）从不写回该文件**
（仅 CLI 路径会写），于是更新后本地版本仍是旧值/缺失（None）→ 恒报同一版本可更新。
另有一个连带问题：CLI 写回时 `data_version: None` 直接覆盖文件，会丢掉已记录的数据版本。

## 变更
- `crates/jpcg_update/src/lib.rs`：
  - **以当前运行二进制版本为唯一真相**：`check_updates` / `fetch_app_update_info`
    新增 `current_version: Option<&str>` 参数（宿主注入；None 回退本 crate 编译版本），
    判定统一走 `needs_app_update(force, current, latest)`
  - **版本号归一后再比**：`CARGO_PKG_VERSION` 为 `2.1.0-beta.3`（无 v），服务器
    `update.toml` 为 `v2.1.0-beta.3`（deploy-gen 带 v）——`normalize_version`
    统一去空白与 `v` 前缀，否则同版本仍会被判为有更新
  - `all_updates`（CLI）同样改用本 crate 编译版本；新增 4 个单测
    （相同/不同/force、v 前缀归一）
- `crates/jpcg_core/src/host/update.rs`：`check_update` / `perform_app_update`
  透传 `current_version`
- `crates/jpcg_core/src/ffi.rs`：`update_check` / `update_app` 请求新增可选
  `current_version`
- `examples/jpcg_app/src-tauri/src/commands/update.rs`：static 直调传
  `env!("CARGO_PKG_VERSION")`；dynamic 在 JSON 中带 `current_version`
- `crates/jpcg_update/src/ffi.rs`：update dll 的 check / fetch_app_info 请求同步支持
- `crates/jpcg_update/src/download.rs`：CLI 写回改读-改-写，**保留 data_version**

## 决策
- `local_update_info.toml` 退化为"更新渠道 + 数据版本记忆"，不再参与应用版本判定；
  文件缺失/过时/首装/手动替换二进制都不会误报更新
- 数据版本比较仍按记录值（数据文件无编译期版本，语义不同）

## 验证
- `cargo check -p jpcg_update -p jpcg_core`、`-p jpcg_app`（static 与 dynamic）通过
- `cargo test -p jpcg_update --lib`：6 passed（含版本比较三态、v 前缀归一、回退）

### 来自 changes/2026-09-18-133000.md

# feat(values): 数值集外置到 data 通道 + 运行期选择（P1，默认体验服一测）

日期：2026-09-18

## 背景
2.1.0 目标：一个包同时兼容体验服/正式服数值。原实现把等级常数/换算分母
**编译期固化**（`crates/jpcg_const/preset/*.toml` + `include_str!`），换数值必须重打包。
按决策改为**架构 C**：数值真源外置到 data 通道，运行期按"值集"加载；
体验服·苍生铸世一测作为**默认值集**，可切换正式服。技能/装备/血量等留待后续 data 更新。

## 变更
- **新增 `data/values/`**（随 data 通道下发，manifest-gen 递归收集已覆盖）
  - `index.toml`：`default = "cszj-exp-260908"` + 两个 `[[value_sets]]`（体验服一测 / 正式服130）
  - `cszj-exp-260908.toml`：体验服一测数值（原 exp 分支 preset 迁入）
  - `live-130.toml`：正式服 130 数值（原 `crates/jpcg_const/preset/level_constant.toml` 迁入）
- **`crates/jpcg_const`**：删除 `preset/`；`include_str!` 改指 `data/values/*`；
  新增 `LIVE_130`（正式服基线）；`CURRENT`/`LEVEL` 语义改为**内置兜底默认（体验服一测）**
- **`crates/jpcg_core/src/store/values.rs`（新增）**：运行期加载
  - `list_value_sets()` / `load_value_set(id)`；白名单严格解析（`deny_unknown_fields`）
  - 校验：换算分母须为有限正数、`pvp_global_jianshang ∈ [0,1]`、`level > 0`
  - 任一失败（目录/文件/解析/校验）→ 回退内置兜底并告警；`source` 字段区分 `data`/`builtin`
- **`store/paths.rs`**：新增 `values_dir()`（JPCG_DATA_DIR → exe 同目录/data/values → .app Resources）
- **`type_set/coefficient.rs`**：`Default` 固定为 `LIVE_130`（历史/测试基线，保持金标准不变）；
  新增 `from_dto(dto, base)` 与 `from_level_constant`；`From<&DTO>` 保留为兼容入口
- **`host/calc.rs`**：按 `req.value_set` 加载值集，作为 0/缺失字段的回退基准
- **`host/values.rs`（新增）+ `ffi.rs` + Tauri `list_value_sets_cmd`**：值集查询入口（双模式）
- **`jpcg_api`**：`CalculateRequest`/`ConfigDataDTO` 增 `value_set`；新增 `ValueSetDTO`
- **`store/config.rs`**：`saved_config.toml` 增 `value_set`（保存时保留旧值，避免丢失）
- **`tauri.conf.json`**：bundle.resources 增 `data/values/*.toml`（打包进 .app）

## 决策
- 数值真源外置（架构 C），**不改技能/装备/血量**（留后续 data 更新）
- `CoefficientConfig::Default` 保持正式服 130 基线 → 正式服金标准预期值**完全不变**
- 产品默认值集由 `data/values/index.toml` 的 `default` 决定（当前=体验服一测）

## 验证
- `cargo check`：const/api/core/app（static+dynamic）/combo/update 全过
- `cargo test -p jpcg_const -p jpcg_core`：const 5 passed、core 19 passed（金标准未变）
- `store::values` 单测：无 data → 内置兜底；`JPCG_DATA_DIR=./data` → 按 id 加载 live-130 与默认体验服

## 后续（P2/P3）
- 前端：值集下拉 + 持久化 + 状态栏展示；`CalculateRequest` 携带 `value_set`；去 `?raw`
- 双平台打包与 beta.4 公测 → 2.1.0 正式 → stable 通道首次落位

### 来自 changes/2026-09-18-150000.md

# fix(values): 修复 review 三项（连招值集一致性 / DATA_DIR 同级 values / 可用性与回退可观察）

日期：2026-09-18

针对 PigeonMuyz 对 PR #28 的审核意见（1×P1 + 2×P2）逐项修复。

## 1. [P1] 单技能与连招使用同一运行期值集解析
- 新增 `jpcg_core::host::values::value_set_constant(id)` 作为**唯一取值入口**
- `jpcg_combo::host::calculate_combo` / `into_core` 新增 `value_set` 参数，
  改用 `CoefficientConfig::from_dto(&dto, &value_set_constant(id))`（原为固定 `LIVE_130`）
- 打通调用链：combo FFI `ComboRequest` 增 `value_set`；Tauri `calculate_combo_cmd`（static/dynamic）
  增参并透传；前端 `calculateCombo` 透传 `value_set`
- 效果：省略/置 0 分母时，单技能、求导、连招回退到**同一值集**（默认=体验服一测）

## 2. [P2] `JPCG_DATA_DIR` 直接指向 `data/shuxing` 时 values 解析错误
- 新增 `paths::values_dir_from_env()`：兼容指向数据根与指向 `<root>/shuxing`（取同级 values）
- `values_dir()` 走该函数；`data_dir()` 语义保持不变

## 3. [P2] 单快照损坏时无法反映实际使用的值集
- `ValueSetDTO` 增 `available`：`list_value_sets()` **逐项校验快照**（读取+解析+校验）
- 新增 `host::values::resolve_value_set(id)` + core FFI `resolve_value_set` +
  Tauri `resolve_value_set_cmd`：返回本次**实际使用**的值集信息（含 `source=builtin` 回退标记）

## 4. 测试（确定性，不依赖真实 data / 环境变量）
- `store::values`：改用**隔离临时目录**注入 `*_from` / `load_with_fallback`，覆盖
  - 列表可用性与默认标记；显式/默认/未知 id 加载
  - 快照**缺失**、**分母置 0**、**未知字段** → `available=false` 且加载回退 `builtin`
  - `values_dir_from_env` 两种写法（数据根 / `<root>/shuxing`）均命中 `<root>/values`
- `jpcg_combo`：新增跨入口一致性回归（省略分母时连招与单技能取同一值集）
- `jpcg_core::ffi_tests`：加串行锁（全局 `last_error` 并行覆盖导致偶发失败）

## 验证
- `cargo check`：core/combo/app（static+dynamic）通过
- `cargo test --workspace` 连跑 3 次全绿（combo 12 / const 5 / core 21 …）
- 前端 `tsc --noEmit` 通过

### 来自 changes/2026-10-09-080730.md

# fix(values): 快照修复后独立动态库自动恢复值集

日期：2026-10-09

## 改动与原因

PR #28 在快照缺失、TOML 损坏或系数非法时缓存内置兜底，却没有保存快照依赖。
当数据更新只修复快照、不改 `index.toml` 时，combo 独立动态库会继续命中失败缓存。

- `load_cached` 不再缓存失败结果，并移除同键的旧条目；合法快照仍使用原有 mtime 缓存。
- `CacheEntry.snapshot_path` 改为必需路径，消除无快照依赖却被判为有效的缓存状态。
- 保留内置回退和告警；不增加跨 cdylib 广播接口，不改变计算公式或数据版本。
- 取舍：仅故障状态下重试读盘/告警，优先保证修复立即可见；正常计算保留缓存性能。
- 增加缺失、语法损坏、非法分母三种恢复回归：预热失败读取，只修快照，确认 index mtime 不变，
  随后同 ID 自动恢复正式服常数，不调用显式缓存失效。

## 动态库回归

新增 `scripts/test-values-dynamic.py`（Python 3.11+，仅标准库）：真实加载 core 与 combo 两个 cdylib，
在缺失、TOML 损坏、非法分母三种场景预热两份缓存；修复时只写快照，断言 index 内容、inode、
大小、mtime 都不变。先检查 combo，再检查 core，均与显式正式服系数的完整计算响应比较。
目标 HP 设为 0，避开随机蒙特卡洛；缺少模块直接失败，不悄悄跳过。CI 已接入。

运行方式：

```sh
cargo build -p jpcg_core -p jpcg_combo
python3 scripts/test-values-dynamic.py
```

## 验证

- `cargo test -p jpcg_core -- golden`：9/9 通过。
- `cargo test -p jpcg_core -p jpcg_combo`：core 36/36、combo 12/12 通过。
- `cargo build -p jpcg_core -p jpcg_combo` + Python 双 cdylib 回归：缺失/语法损坏/非法分母三例全部通过。
- `cargo fmt --all -- --check`：通过。
- `make check-all`：通过。static/dynamic app、四个 cdylib 构建成功；workspace 59 个测试通过，
  dynamic app FFI roundtrip 1 个测试通过。
- 实际 CI 命令 `cargo clippy --workspace --all-targets`：通过，仅保留未改动的
  `crates/jpcg_combo/src/host.rs:77` 八参数既有告警。
- 同时执行 CONTRIBUTING 中的严格命令 `cargo clippy --workspace --all-targets -- -D warnings`：
  因上述既有 `too_many_arguments` 告警失败（rc 101），未将其记为通过，也未扩大修复范围。
  AGENTS/当前 CI 明确不将既有告警作为门禁。
- dynamic 构建仍有未改动的 `ffi_bridge.rs:315` 冗余 unsafe 告警。
- 完整矩阵后再次运行金标准、双 cdylib 回归、rustfmt 与 `git diff --check`，全部通过。

验证在 Linux 云环境完成，工具链和 Tauri 依赖安装在工作区用户目录，没有修改用户电脑。
本次未发布、合并或打 tag；远端新提交的 CI 仍需在提交后确认。

### 来自 changes/2026-10-09-080731.md

# fix(app): 存档加载后保持当前数值版本一致

日期：2026-10-09

## 改动与原因

PR #28 加载正式服存档时只改变表单，清空和数据更新仍从旧 localStorage 读取体验服选择，
导致当前请求、下拉选择和“实际使用”提示不一致。

以当前选择为运行期真源，持久化只用于启动恢复；加载、切换、清空和热更新沿用同一选择。
异步解析需要确认请求仍有效，避免旧响应覆盖更新选择；热更新继续保留用户修改或存档中的系数，
明确清空/切换时才重新套用值集常数。

- 分开跟踪选择操作与解析请求，防止切换、加载、热更新响应乱序覆盖新状态。
- 等待当前值集解析期间禁用计算/保存，避免新 ID 配旧系数提交；解析失败需重试成功后再提交。
- 旧存档无 `value_set` 时使用默认语义（null），清除旧持久化选集；默认选择不误报“不可用”。
- 新增 Vitest + jsdom（真实 React DOM、仅 mock 命令边界），`npm test` 接入前端 CI；
  lockfile 同步，既有依赖版本保持不变。

## 验证

在 `examples/jpcg_app/` 执行：

- `npm ci`：通过。
- `npm test`：16/16 通过，覆盖体验服→加载正式服→清空/更新、计算请求和“实际使用”提示，
  存档/手动系数保留、默认选择、失败加载恢复、初始化/加载/切换/热更新响应乱序、待解析提交阻止。
- 负向回归验证：临时替换为原 HEAD 组件时 16 例中 13 例失败；恢复修复版后重新全绿。
- `npm run build`（tsc + Vite）：通过；仍有既有产物体积提示（chunk > 500 kB）。
- `git diff --check`：通过。

本次为普通修复，不 bump 产品版本、不提前聚合发布 CHANGELOG。锁文件根包版本由过期的
`2.1.0-alpha.1` 同步到原 `package.json` 已有的 `2.1.0-alpha.2`，不是新版本升级。

### 来自 changes/2026-10-09-083600.md

# fix(app): 阻止新值集基线待解析时混入旧系数

日期：2026-10-09

## 问题与复现

提交 `5e11d3b` 的提交后复核发现：切换值集或清空后、解析完成前，表单仍保留旧系数。
此时修改一个系数会把全局自定义标记设为 true，令整个新值集的 seed 被跳过，形成
“新 ID + 一个手改系数 + 其余旧系数”的请求。计算/保存等待解析的门禁不能阻止这种混合状态。

先新增真实组件回归，在已发布实现上确认“切换待解析”和“清空待解析”两例都失败，
然后再修改实现。

## 修复与取舍

单独跟踪新值集基线是否待解析，在初始加载、切换和清空的基线未就绪期间禁用系数编辑，
同时在编辑处理函数中拒绝待解析期间的旧事件。成功解析后恢复编辑，避免将旧系数误作新集自定义值。

同一选集的数据热更新仍允许手动编辑并保留自定义系数；存档系数已构成完整基线，也保持可编辑。
不引入逐字段覆盖模型，不改数值计算或后端缓存，不扩大修复范围。

## 验证

- `npm ci`、`npm test`、`npm run build`：通过，20 个真实组件回归全部通过。
- 新增四例：切换/清空待解析时不混入单字段编辑；存档基线在来源解析期间仍可编辑；
  迟到的旧解析不能解锁，解析失败保持锁定直至重试成功。
- 原同选集热更新测试增加输入仍可编辑的断言，验证手工覆盖策略未退化。
- 补丁前的已发布实现明确失败前述切换与清空两例；修复后全部通过。
- 仍有既有 Vite chunk > 500 kB 提示。
- `make check-all`：完整通过，static/dynamic app、四模块构建成功；workspace 59 个测试、
  dynamic FFI roundtrip 1 个测试通过。
- 金标准 9/9、真实双 cdylib 三类快照恢复、rustfmt、普通 workspace/all-targets clippy、
  `git diff --check` 全部通过。
- 独立提交后复核再次运行 20 个前端测试并检查差异，原发现关闭。
- 全量重跑曾遭云执行连接中断，恢复后重新完成全部检查，没有把中断运行记为通过。
- 本轮没有 Rust 改动；既有 combo 参数过多及 dynamic 冗余 unsafe 告警仍在，
  严格 `-D warnings` 的既有失败情况见 `changes/2026-10-09-080730.md`。

验证仅在云工作区进行；未合并、打 tag 或发布。

### 来自 changes/2026-10-09-124157.md

# fix(update/data): 统一数据根（写读一致）+ 值集内存缓存 + 保存不再重置 buff/系数 + 快照路径校验

日期：2026-10-09

PR #28 自检发现的问题（先经临时探针**实测复现**再修，避免误判）。共修 4 项。

## 1. [高·既有 bug] 数据更新落盘位置 ≠ 运行期读取位置
**现象（实测）**：`download_and_install_data` / `check_data_updates` 用 `base_path.join("data")`，
而 `base_path = Path::new(".").canonicalize()`（**CWD**）。读取侧走 `exe_dir/data`、bundle
`Contents/Resources/data`、`JPCG_DATA_DIR`。探针：往 `CWD/data` 写好后，读者**完全忽略**
（`list_professions=0`、`value_sets` 仍为 `builtin`）。
安装版（Finder 启动 CWD=`/`）会把数据写到 `/data`（失败/写错位置），`values/` 永远回退内置，
`check_data_updates` 反复判定需更新。此 bug 在 P1 之前即存在。

**修复**：引入「数据根」单一真源，读写共用同一解析。
- `store::paths`：新增 `data_root_writable()`（更新落盘目标）与纯函数 `writable_root` / `read_roots`，
  并新增 `bundle_user_data(home)`（`~/Library/Application Support/com.qinthirteen.jpcg/data`，
  与连招预设目录同源）。`data_dir()` / `values_dir()` 改为基于 `read_root_candidates()`。
  - 写入：`JPCG_DATA_DIR` → bundle 用户目录 → `exe_dir/data`
  - 读取：`JPCG_DATA_DIR` → bundle 用户目录（更新落盘处，**优先**）→ bundle `Resources/data` → `exe_dir/data`
- `jpcg_update`：`check_data_updates` / `download_and_install_data` 改为显式 `data_root` 参数
  （去掉内部 `.join("data")`）；`check_updates` / `download_updates` 的参数语义改为 `data_root`
  （二者原本就只用于 data）。`all_updates`（CLI）与 update FFI 传 `<应用根>/data` 保持既有行为。
- `host::update`：`base_path()`（CWD）→ `data_root()`（`store::paths::data_root_writable()`）；
  `perform_app_update` 不再依赖 CWD（`fetch_app_update_info` 本就不用该参数）。

## 2. [中·既有 bug] 保存配置会重置 buff 与系数
**现象（实测）**：`store::save_config` 硬编码 `buff: default()` / `coefficient: default()`，
而前端 `saveConfig` 只传 player/hostile/xinfa → 一保存就把阵眼/奇穴增益与系数清零。
（`value_set` 实际**保留**：早前探针因输入缺必填段导致“解析失败”，属误判，已更正。）

**修复**：`save_config` 改为**保留**已存的 `buff` / `coefficient` / `value_set`（不再写默认值）。
真正的“保存用户改过的 buff/系数”需前端传参，属 P2 范畴。

## 3. [低] `index.toml` 的 `file` 可目录穿越
**现象（实测）**：`file = "../outside.toml"` 能读到 values 目录外（`huixin=9999` 加载成功）。
**修复**：`read_snapshot` 前置 `is_safe_relative_filename`（拒绝 `/`、`\`、`.`、`..`、空名），
非法即视为不可用并回退内置。

## 4. [优化] 每次计算读盘 → 内存缓存
**现象（实测）**：每次 `value_set_constant` 都读 index + 快照 + 校验，1000 次 ≈ 69ms。
**修复**：`store::values` 增进程内缓存（`Mutex<ValueCache>`，键含解析目录与 id）；
新增 `invalidate_cache()`，在 `host::update::perform_update` 数据落盘后调用，使新数据即时生效。

## 测试（确定性，不依赖真实 data / 环境变量）
- `store::paths`：`writable_root`（dev / bundle / env）、`env_data_root`、
  `read_roots`（bundle 下用户目录优先于 Resources）、以及「可写根 == 读取首选」的写读一致不变式
- `store::values`：`rejects_path_traversal_in_index`
- `jpcg_core::ffi_tests`：`save_config_preserves_buff_coefficient_and_value_set`

## 验证
- `cargo test -p jpcg_core -p jpcg_update -p jpcg_const -p jpcg_combo` 全绿（core 29 / update 6 / const 5 / combo 12）
- `cargo build-app-static` / `cargo build-app-dynamic` 通过
- `cargo fmt --check` / `cargo clippy -p jpcg_core -p jpcg_update` 无告警

### 来自 changes/2026-10-09-125428.md

# feat(values): P2 前端接线——数值版本下拉 + 系数 seed + 持久化（架构 C 落地）

日期：2026-10-09

架构 C 的「可更换数值」需前端真正生效，否则值集选择形同虚设。本笔补齐 P2，并顺带修复 2 个相关 bug。

## 1. 前端「数值版本」下拉（核心）
- `ConfigPanel` 顶部新增「数值版本」下拉：列出 `list_value_sets_cmd` 结果（不可用项禁用）
- 选中即调 `resolve_value_set_cmd`，用返回的**实际使用**值集（含回退）的常数**填充「系数设置」**
  （用户仍可再改），并写入 `form.value_set`
- 启动时：读取 `localStorage[jpcg_value_set]` → 命中且可用则用；否则用 `is_default`（体验服一测）；
  再否则第一个可用项。**App 默认数值因此从硬编码正式服 130 改为体验服一测**（架构 C 决策）
- 选择持久化用 localStorage（与 `lastXinfa`/`betaChannel` 同款）；`清空` 后重新套用当前值集

## 2. DTO：值集携带换算常数
- `jpcg_api::ValueSetDTO` 增 `coefficient: Option<CoefficientConfigDTO>`（快照可用时给出 8 个常数）
- `store::values`：`entry_to_dto` 接收已解析常数；`list_value_sets`/`resolve_value_set` 一次返回即可 seed

## 3. 全链路透传 `value_set`
- 前端 `CalculateRequest`/`FormData` 增 `value_set`；`toCalculateRequest` 携带
- `ComboPage` 调 `calculateCombo` 时透传 `req.value_set`；`OptimizePage` 经 `compute_derivatives`
  随 `CalculateRequest` 自动携带
- `api`：新增 `listValueSets()` / `resolveValueSet(id)`

## 4. 顺带修复的 bug
- **`calculateCombo` 参数名错误**：Tauri v2 命令参数为 **camelCase**（JS 侧 `forumUrl`→Rust `forum_url`），
  原代码发 `value_set`（snake_case）→ 被当作未知键忽略 → 连招的值集透传**一直是死代码**。
  改为 `valueSet`，连招现在真正随值集计算。
- **技能编辑保存写只读 bundle**：`store::save_skill_toml` 原写 `data_dir()`（安装版指向只读
  `Resources/data/shuxing`）→ 改为写 `data_root_writable()/shuxing`，与读取侧统一。

## 测试 / 验证
- `store::values`：`dto_carries_coefficient_when_available`（可用带常数 / 缺失为 None）
- `cargo test --workspace` 全绿（core 30 / combo 12 / update 6 / const 5）
- 前端 `tsc --noEmit` + `npm run build` 通过
- `cargo build-app-static` / `cargo build-app-dynamic` 通过

### 来自 changes/2026-10-09-132651.md

# fix(values): 修复 PigeonMuyz 复审（1×P1 + 4×P2）——多目录合并 / 保存一致性 / 初始化回退 / 热更新 / 双库缓存

日期：2026-10-09

针对 `76b6891` 的复审逐项修复。所有问题均**对照代码核实为真**后再修，并补确定性回归。

## 1. [P1] 安装版保存单个心法后，用户目录遮蔽其他随包心法
**问题**：`save_skill_toml` 在用户目录创建 `shuxing/A.toml` 后，`data_dir()` 按目录存在性选中
**整个**用户 `shuxing`，不再读 `Resources/data/shuxing` → 其他心法加载为空、门派列表只剩已保存者。

**修复**：改为**多目录合并读取**（用户覆盖优先，其后随包资源）：
- `paths::data_dirs()`：返回 shuxing 目录候选（存在者，按优先级）；`store::data_dirs` 导出
- `store::toml::load_config_from(dirs, profession)`：取**首个**存在该文件的目录
- `store::profession::list_from_dirs(dirs)`：先按文件名去重（先到先得）再分组 → 跨目录并集
- `engine::start_calculation_with_config`：在 `data_dirs()` 中定位文件
- 测试：`load_config_merges_across_dirs_user_first`、`list_merges_across_dirs_user_first`
  （资源有 A/B、用户仅 A → A 取用户、B 仍可加载、列表含 A/B）

## 2. [P2] 默认体验服的“保存→加载”变成体验服标签、正式服系数
**问题**：`handleSave` 不传 `coefficient`/`value_set`，后端按缺失文件取默认（LIVE130 + None）；
加载后 value_set 回用 prev（体验服）→ id 与系数不一致。

**修复**：保存时携带完整配置。
- `store::save_config(player, hostile, xinfa, buff, coefficient, value_set)`（不再从文件“保留”）
- `host::config::save_config` 增三参；core FFI `save_config` 请求增 `buff/coefficient/value_set`；
  Tauri `save_config_cmd` 增参（static/dynamic）
- 前端 `saveConfig` 发送 `buff`（`mode_is_point` 保布尔）、`coefficient`、`valueSet`
- `handleSave` 传 `form.buff` / `norm.coefficient` / `form.value_set`
- 旧存档兼容：缺 `value_set` → `None`（`#[serde(default)]`）；系数按存档恢复
- 测试：`save_config_persists_buff_coefficient_value_set`

## 3. [P2] 默认快照损坏时初始化未应用回退，仍发送正式服系数
**修复**：初始化与用户切换**统一走 `resolveValueSet`**：
- `resolveAndSeed(id)`：`resolveValueSet` 返回**实际使用**的值集（含 builtin 回退）与其常数 → seed
- 挂载时 `pickValueSetId`（localStorage→默认→首个可用→…）后 `resolveAndSeed`
- 下拉下方显示「实际使用：…（内置兜底 / 已回退）」

## 4. [P2] 数据热更新后前端仍把旧快照常数作为显式覆盖发送
**修复**：
- 数据更新成功后重新 `listValueSets()` + `resolveValueSet(当前)`：刷新列表、更新“实际使用”显示，
  且**未手动改过系数时**重新 seed（`coefficientCustomRef` 跟踪用户是否改过系数）
- `handleClear` 改为重新 `resolveAndSeed(当前)`（不再依赖可能过时的 `valueSets` 快照）

## 5. [P2] dynamic 模式仅清 core 缓存，combo 动态库继续用旧值集
**修复**：缓存改为 **mtime 校验**（`CacheEntry` 记 index/快照 mtime）——文件被数据更新改写后，
**每个 cdylib 各自**在下次读取时自动失效，无需跨库广播。
- 键仍为 (目录, id)；`cache_valid` 比对 index 与快照 mtime
- `list_value_sets` 去掉缓存（非热点）；新增 `load_cached(dir, id)` 便于测试
- 测试：`cache_reloads_after_snapshot_change`（同 id 快照改写后重载）

## 验证
- `cargo test --workspace` 全绿（core 33 / combo 12 / const 5 / update 6）
- `cargo build-app-static` / `cargo build-app-dynamic` 通过
- 前端 `tsc --noEmit` + `npm run build` 通过
- `cargo fmt --check` 通过；clippy 仅余既有告警（combo 8 参 / app unused_unsafe）

## [2.1.0] - 2026-10-13

### 来自 changes/2026-09-04-200540.md

# 修复 release.sh：独立版本降级中断 + prep 分支命名冲突

日期：2026-09-04

## 背景
首次执行 `scripts/release.sh beta`（2.1.0-beta.1）暴露两处缺陷：

1. `cargo set-version "$VERSION"` 会把 workspace 内**所有**成员都尝试改成新版本，
   但 `jpcg_const` 使用独立版本号（`130.3.20260602`，等级.赛季.日期），
   直接被判定 "Cannot downgrade from 130.3.20260602 to 2.1.0-beta.1" 中断脚本。
2. prep 分支名 `release/prep-${VERSION}` 与已有分支 `release` 冲突
   （GitHub 禁止 `release/xxx` 与分支 `release` 共存），push 被拒。

## 变更
- `scripts/release.sh`：
  - 版本 bump 改为**仅改根 `[workspace.package]` version**（继承成员自动生效，
    独立版本 crate 不动），随后 `sync-version.sh` 同步前端/tauri/模拟串，
    再 `cargo metadata` 刷新 Cargo.lock
  - prep 分支改为 `prep/${VERSION}`，避开 `release/` 命名空间
- `scripts/README.md`：同步两处描述

## 验证
- beta.1 实际发布时手工等价步骤跑通（根版本改 + sync-version + lock 刷新 + prep/v2.1.0-beta.1 + tag）
- 本修复让 `release.sh beta` 可全自动复现

## 注意
- 更新器 `jpcg_updater` 同样为独立版本（2.1.0），不受 workspace bump 影响

### 来自 changes/2026-09-04-210000.md

# 修复 release.yml：Windows runner 两步缺 shell: bash

日期：2026-09-04

## 背景
`v2.1.0-beta.1` 触发 release.yml 时 windows-latest job 在"判断更新通道"步骤失败：
该步（及"提取版本号"）未指定 `shell: bash`，Windows runner 默认用 PowerShell，
`[[ "$GITHUB_REF_NAME" == *-beta* ]]` 为 bash 语法 → 报错，导致三平台只产出 darwin/linux 资产。

## 变更
- `.github/workflows/release.yml`：为"提取版本号（tag）"与"判断更新通道"两步显式加 `shell: bash`

## 验证
- 失败 run 335...（33870657978）三 job 结论：darwin/linux success、windows 唯一失败步 = 判断更新通道
- 修复后重建 tag 重跑，windows 应产出 app exe / dll / 清单

### 来自 changes/2026-09-04-220000.md

# feat: 以 GitHub Release 为载具的更新自动部署（deploy-gen + package job + 模块平台过滤）

日期：2026-09-04

## 背景
beta.1（v2.1.0-beta.1）发布与服务器落盘全程手工。目标：tag 构建成功后自动产出
服务器可直接应用的"通道 tgz"，随 Release 推送；服务器侧以 webhook 拉取落位
（服务器接收端为独立私有仓库实现，不随本仓库发布）。

## 变更（本仓库 jpcg）
- 新增 `server_tools/deploy-gen`（workspace 成员）：
  - 输入：GitHub Release 资产目录 + 仓库 data + version/channel
  - 输出：更新服务器 downloads 根布局 stage（beta 根式 / stable 版本目录式）
  - 生成 update.toml / manifest.toml（app 二进制 sha256）/ 三平台合并
    modules_manifest.toml（platform="multi"，core/update 版本随 app、
    const 反查自身 Cargo.toml 独立版本）/ data 布局（按 data_manifest 列出的文件拷贝）
  - beta 与 stable 双通道本地试跑通过，与 beta.1 手工部署布局一致
- `.github/workflows/release.yml`：`deploy`(SSH 直推) 方案废弃，改为新增
  `package` job（needs: build，仅 tag push）：下载资产 → deploy-gen → tar 为
  `jpcg-<channel>-<tag>.tgz` 并附 `.sha256` → `gh release upload` 到同一 Release
- `crates/jpcg_update/src/modules.rs`：模块更新按本机平台扩展名过滤
  （mac=.dylib / linux=.so / windows=.dll），适配多平台合并清单，避免下载无关 dll

## 验证
- `cargo build -p deploy-gen` / beta、stable 双通道试跑通过
- `cargo check -p jpcg_update` 通过

## 待办（服务器接收端，独立私有仓库，不在本仓库实现）
- 服务器 webhook 接收（验签）→ 拉取该 tgz → 校验 .sha256 →
  原子切换（beta 覆写根 / stable 合并保留 3 版）
- 下载服务目录列表处理（stable 客户端依赖 /updates/JPCG/ 的 HTML 目录列表）

### 来自 changes/2026-09-08-214000.md

# fix: 更新检查/执行无限等待 + Tauri v2 中 window.confirm 失效导致无法确认更新

日期：2026-09-08

## 背景
- beta.1 构建中点击"检查更新"卡在"正在检查更新..."不返回（Beta 开关开/关都卡）。
  服务器侧所有被请求 URL 均 200，定位为客户端代码层。
- 修复后 beta 仍"点了无反应"：Tauri v2 的 WebView **禁用 window.confirm**（恒返 false），
  检测到新版本时的确认框被当"取消"处理 → 界面直接复位、既不更新也不提示。

## 变更
- `crates/jpcg_update/src/download.rs`：新增 `http_client()`（统一 60s 超时），
  全部 5 处 `reqwest::Client::new()` 改用之
- `crates/jpcg_update/src/modules.rs`：模块清单请求同样走带超时客户端
- `examples/jpcg_app/src-tauri/src/commands/update.rs`：`check_update` / `perform_update` /
  `perform_app_update` / `perform_modules_update` 同步实现移入
  `tauri::async_runtime::spawn_blocking`，避免在 async 命令线程内嵌套新建 tokio runtime
- `examples/jpcg_app/src/components/ConfigPanel.tsx`：废弃 `window.confirm`，改为应用内
  确认区（发现应用/模块更新时显示"下载更新/取消"按钮），确认后走原下载→验证→重启流程。
  补充状态修正：命中确认时 `updating` 保持 true（禁用重复触发）、进度框与确认框互斥避免
  叠显；取消有 toast 反馈；确保点"下载更新"后进度条可见（否则又像"无反应"）

## 验证
- `cargo check -p jpcg_update` / `-p jpcg_app` 通过；前端 `tsc --noEmit` 通过
- 效果：任何网络卡点 60s 内转为明确报错；命令不再无界等待；更新确认不再被静默吞掉

### 来自 changes/2026-09-08-223000.md

# fix: macOS 产物架构错标 + 全面切到 Apple Silicon（aarch64）单架构

日期：2026-09-08

## 背景
- GitHub 标准 macOS runner 已全部迁移到 arm64（macOS 26，Intel 已被官方淘汰），
  `macos-latest` 上 `cargo build` 产出的就是原生 aarch64 二进制，但此前产物名与清单
  一直写死 `jpcg-app-darwin-x86_64` / `arch = "x86_64"`——**错标**。
- 更隐蔽的是 manifest `os` 字段写的是 `"darwin"`，而客户端按
  `std::env::consts::OS` 比对（= `"macos"`），导致 macOS 客户端永远匹配不到二进制，
  报 "未找到适用于 macos aarch64 的二进制文件"（Apple Silicon 上必现）。

## 变更
- `.github/workflows/release.yml`：matrix 增 `arch`；mac 资产命名为
  `jpcg-app-macos-aarch64`，linux/windows 保持 `x86_64`；命名不再硬编码
- `server_tools/deploy-gen/src/main.rs`：`APP_BINS` → macos/aarch64，清单 `os` 字段
  用客户端词汇（`macos` 而非 `darwin`）
- `server_tools/manifest-gen/src/main.rs`：`detect_platform` darwin → macos
- `server_manifest.md`：目录/清单示例修正为实际 `{os}-{arch}` 命名，去掉 Intel mac

## 决策
- **不保留 Intel macOS**：苹果已淘汰全部 Intel Mac，不为旧机型维护 x86_64 构建；
  每平台单架构，模块 dylib 亦无同名冲突
- 产物命名/清单 `os+arch` 与客户端 `std::env::consts::{OS,ARCH}` 严格对齐

## 验证
- 本机（arm64）dev 构建此前因清单缺 macos/aarch64 命中报错；beta.2 发布后此路径应打通

### 来自 changes/2026-09-08-223500.md

# chore: dev 补齐 release.sh 修复（#11）+ 清理已聚合 changes

日期：2026-09-08

## 背景
- dev 分支此前缺失 #11 的 `scripts/release.sh` 修复（仅 beta 有）：dev 版仍是
  `cargo set-version`（会因 jpcg_const 独立版本降级而中断）+ `release/prep-` 分支名
  （与已有 `release` 分支冲突）。dev 作为最上游分支必须与修复同步，避免后续提升把
  未修版脚本带回 beta/release。
- `changes/` 累积了 56 个已聚合进 beta 分支 `[2.1.0-beta.1]` CHANGELOG 的旧条目，
  若不清除，下次发布会把这些历史再聚合一遍（重复）。

## 变更
- `scripts/release.sh` / `scripts/README.md`：从 beta 同步 #11 修复
  （版本 bump 仅改根 `[workspace.package]` + `sync-version.sh` + `cargo metadata`
  刷 lock；prep 分支名改 `prep/<版本>`）
- `changes/`：删除 56 个已聚合条目，保留未发布条目
  （2026-09-04-200540/210000/220000、2026-09-08-214000 与本仓库内新条目）

## 验证
- 与 origin/beta 对比 `scripts/` 差异为空（完全对齐）

### 来自 changes/2026-09-08-231500.md

# fix: release.sh 在 set -u 下 `$VAR` 紧跟全角标点触发 unbound variable

日期：2026-09-08

## 背景
`scripts/release.sh` 第 7 步 `gh pr create` 的 `--body` 及末尾提示语中，
`$STAGE` / `$TARGET_BRANCH` 后面紧跟全角字符（`，`、`。`）。在 `set -u`
（`set -euo pipefail`）下，bash 会把多字节字符的首字节并入变量名，导致
`STAGE�: unbound variable` 报错，脚本在 beta.2 发布时于第 7/8 步中断
（commit + tag 已生成、prep 分支已推，但 PR 与 tag 推送需手工补做）。

## 变更
- `scripts/release.sh`：`$STAGE，` → `${STAGE}，`；`$TARGET_BRANCH。` →
  `${TARGET_BRANCH}。`（用花括号显式界定变量名，避免紧跟多字节字符被并入）

## 验证
- 复现：`bash -uc 'STAGE=beta; x="…（$STAGE，…"'` 报 `STAGE�: unbound variable`；
  加花括号后通过
- 已用于 beta.2 发布的手工补做（PR + tag 推送）

### 来自 changes/2026-09-08-233000.md

# fix: 更新 URL 拼接缺分隔符（beta/stable/all_updates 三处 404）

日期：2026-09-08

## 背景
beta 通道点"下载更新"报 `下载失败: 下载文件失败，HTTP 状态码: 404 Not Found`。

根因：`fetch_app_update_info` / `all_updates` 里用
`format!("{}{}", base_url.trim_end_matches('/'), path)` 直接拼接，
而 base trim 后无结尾 `/`、path 也无前导 `/`，导致：
- beta：`.../updates/JPCG_beta` + `jpcg-app-macos-aarch64`
  → `.../JPCG_betajpcg-app-...`
- stable：`.../updates/JPCG` + `v2.1.0` + `/...`
  → `.../JPCGv2.1.0/jpcg-app-...`
- `all_updates` 的 `version_url`：`.../JPCG` + `v2.1.0/`
  → `.../JPCGv2.1.0/`

3 处均为同一类漏分隔符问题（stable/CLI 通道此前未被实际使用，故只有 beta 显现）。

## 变更
- `crates/jpcg_update/src/lib.rs`：
  - 新增 `join_url(base, segments)` 统一拼接（base 去尾 `/`，各段去首尾 `/` 后以 `/` 连接）
  - 三处（beta 二进制、stable 版本目录、all_updates version_url）改用 `join_url`
  - 补 3 个单测覆盖 beta/stable/缺斜杠容错

## 验证
- `cargo test -p jpcg_update --lib`：3 passed
- 修复后 URL = `https://nefinita-ai.com/updates/JPCG_beta/jpcg-app-macos-aarch64`（服务器实际 200）

### 来自 changes/2026-09-08-234500.md

# fix: Release 产物用 tauri build 嵌入前端（裸 cargo build 产物白屏）

日期：2026-09-08

## 背景
beta.2 更新到新版后**白屏**。排查：
- 浏览器 mock / `tauri dev` 前端正常 → 排除 JS 回归
- 本地 `cargo build -p jpcg_app --release` 产物直接运行也白屏
- 对比二进制内容：裸 cargo build 产物搜不到前端资源（如 `index-*.js`）；
  同一份源码用 `npx tauri build --no-bundle` 产物含 `index-*.js`

结论：Tauri v2 前端资源由 `tauri` CLI 构建流程嵌入二进制；
只跑 `cargo build` 不嵌入 frontendDist → 产物无 UI，被 updater 替换后白屏。

## 变更
- `.github/workflows/release.yml`：构建 app 改为
  `npx tauri build --no-bundle`（在 examples/jpcg_app 下，产物仍落在
  target/release/jpcg_app），updater 仍用 `cargo build -p jpcg_updater`

## 验证
- 本机对照：cargo build 产物无 `index-*.js`；tauri build 产物有（UI 正常）

### 来自 changes/2026-09-12-215500.md

# fix: 应用更新判定改用二进制版本（不再依赖 local_update_info.toml）

日期：2026-09-12

## 背景
更新到 beta.3 成功后，界面仍提示"发现新版本 v2.1.0-beta.3，是否下载并重启应用？"。
根因：`check_updates` 用 `local_update_info.toml`（CWD）里的 `version` 与服务器比对，
而 **GUI 更新路径（`jpcg_core::host::update::perform_app_update`）从不写回该文件**
（仅 CLI 路径会写），于是更新后本地版本仍是旧值/缺失（None）→ 恒报同一版本可更新。
另有一个连带问题：CLI 写回时 `data_version: None` 直接覆盖文件，会丢掉已记录的数据版本。

## 变更
- `crates/jpcg_update/src/lib.rs`：
  - **以当前运行二进制版本为唯一真相**：`check_updates` / `fetch_app_update_info`
    新增 `current_version: Option<&str>` 参数（宿主注入；None 回退本 crate 编译版本），
    判定统一走 `needs_app_update(force, current, latest)`
  - **版本号归一后再比**：`CARGO_PKG_VERSION` 为 `2.1.0-beta.3`（无 v），服务器
    `update.toml` 为 `v2.1.0-beta.3`（deploy-gen 带 v）——`normalize_version`
    统一去空白与 `v` 前缀，否则同版本仍会被判为有更新
  - `all_updates`（CLI）同样改用本 crate 编译版本；新增 4 个单测
    （相同/不同/force、v 前缀归一）
- `crates/jpcg_core/src/host/update.rs`：`check_update` / `perform_app_update`
  透传 `current_version`
- `crates/jpcg_core/src/ffi.rs`：`update_check` / `update_app` 请求新增可选
  `current_version`
- `examples/jpcg_app/src-tauri/src/commands/update.rs`：static 直调传
  `env!("CARGO_PKG_VERSION")`；dynamic 在 JSON 中带 `current_version`
- `crates/jpcg_update/src/ffi.rs`：update dll 的 check / fetch_app_info 请求同步支持
- `crates/jpcg_update/src/download.rs`：CLI 写回改读-改-写，**保留 data_version**

## 决策
- `local_update_info.toml` 退化为"更新渠道 + 数据版本记忆"，不再参与应用版本判定；
  文件缺失/过时/首装/手动替换二进制都不会误报更新
- 数据版本比较仍按记录值（数据文件无编译期版本，语义不同）

## 验证
- `cargo check -p jpcg_update -p jpcg_core`、`-p jpcg_app`（static 与 dynamic）通过
- `cargo test -p jpcg_update --lib`：6 passed（含版本比较三态、v 前缀归一、回退）

### 来自 changes/2026-09-18-133000.md

# feat(values): 数值集外置到 data 通道 + 运行期选择（P1，默认体验服一测）

日期：2026-09-18

## 背景
2.1.0 目标：一个包同时兼容体验服/正式服数值。原实现把等级常数/换算分母
**编译期固化**（`crates/jpcg_const/preset/*.toml` + `include_str!`），换数值必须重打包。
按决策改为**架构 C**：数值真源外置到 data 通道，运行期按"值集"加载；
体验服·苍生铸世一测作为**默认值集**，可切换正式服。技能/装备/血量等留待后续 data 更新。

## 变更
- **新增 `data/values/`**（随 data 通道下发，manifest-gen 递归收集已覆盖）
  - `index.toml`：`default = "cszj-exp-260908"` + 两个 `[[value_sets]]`（体验服一测 / 正式服130）
  - `cszj-exp-260908.toml`：体验服一测数值（原 exp 分支 preset 迁入）
  - `live-130.toml`：正式服 130 数值（原 `crates/jpcg_const/preset/level_constant.toml` 迁入）
- **`crates/jpcg_const`**：删除 `preset/`；`include_str!` 改指 `data/values/*`；
  新增 `LIVE_130`（正式服基线）；`CURRENT`/`LEVEL` 语义改为**内置兜底默认（体验服一测）**
- **`crates/jpcg_core/src/store/values.rs`（新增）**：运行期加载
  - `list_value_sets()` / `load_value_set(id)`；白名单严格解析（`deny_unknown_fields`）
  - 校验：换算分母须为有限正数、`pvp_global_jianshang ∈ [0,1]`、`level > 0`
  - 任一失败（目录/文件/解析/校验）→ 回退内置兜底并告警；`source` 字段区分 `data`/`builtin`
- **`store/paths.rs`**：新增 `values_dir()`（JPCG_DATA_DIR → exe 同目录/data/values → .app Resources）
- **`type_set/coefficient.rs`**：`Default` 固定为 `LIVE_130`（历史/测试基线，保持金标准不变）；
  新增 `from_dto(dto, base)` 与 `from_level_constant`；`From<&DTO>` 保留为兼容入口
- **`host/calc.rs`**：按 `req.value_set` 加载值集，作为 0/缺失字段的回退基准
- **`host/values.rs`（新增）+ `ffi.rs` + Tauri `list_value_sets_cmd`**：值集查询入口（双模式）
- **`jpcg_api`**：`CalculateRequest`/`ConfigDataDTO` 增 `value_set`；新增 `ValueSetDTO`
- **`store/config.rs`**：`saved_config.toml` 增 `value_set`（保存时保留旧值，避免丢失）
- **`tauri.conf.json`**：bundle.resources 增 `data/values/*.toml`（打包进 .app）

## 决策
- 数值真源外置（架构 C），**不改技能/装备/血量**（留后续 data 更新）
- `CoefficientConfig::Default` 保持正式服 130 基线 → 正式服金标准预期值**完全不变**
- 产品默认值集由 `data/values/index.toml` 的 `default` 决定（当前=体验服一测）

## 验证
- `cargo check`：const/api/core/app（static+dynamic）/combo/update 全过
- `cargo test -p jpcg_const -p jpcg_core`：const 5 passed、core 19 passed（金标准未变）
- `store::values` 单测：无 data → 内置兜底；`JPCG_DATA_DIR=./data` → 按 id 加载 live-130 与默认体验服

## 后续（P2/P3）
- 前端：值集下拉 + 持久化 + 状态栏展示；`CalculateRequest` 携带 `value_set`；去 `?raw`
- 双平台打包与 beta.4 公测 → 2.1.0 正式 → stable 通道首次落位

### 来自 changes/2026-09-18-150000.md

# fix(values): 修复 review 三项（连招值集一致性 / DATA_DIR 同级 values / 可用性与回退可观察）

日期：2026-09-18

针对 PigeonMuyz 对 PR #28 的审核意见（1×P1 + 2×P2）逐项修复。

## 1. [P1] 单技能与连招使用同一运行期值集解析
- 新增 `jpcg_core::host::values::value_set_constant(id)` 作为**唯一取值入口**
- `jpcg_combo::host::calculate_combo` / `into_core` 新增 `value_set` 参数，
  改用 `CoefficientConfig::from_dto(&dto, &value_set_constant(id))`（原为固定 `LIVE_130`）
- 打通调用链：combo FFI `ComboRequest` 增 `value_set`；Tauri `calculate_combo_cmd`（static/dynamic）
  增参并透传；前端 `calculateCombo` 透传 `value_set`
- 效果：省略/置 0 分母时，单技能、求导、连招回退到**同一值集**（默认=体验服一测）

## 2. [P2] `JPCG_DATA_DIR` 直接指向 `data/shuxing` 时 values 解析错误
- 新增 `paths::values_dir_from_env()`：兼容指向数据根与指向 `<root>/shuxing`（取同级 values）
- `values_dir()` 走该函数；`data_dir()` 语义保持不变

## 3. [P2] 单快照损坏时无法反映实际使用的值集
- `ValueSetDTO` 增 `available`：`list_value_sets()` **逐项校验快照**（读取+解析+校验）
- 新增 `host::values::resolve_value_set(id)` + core FFI `resolve_value_set` +
  Tauri `resolve_value_set_cmd`：返回本次**实际使用**的值集信息（含 `source=builtin` 回退标记）

## 4. 测试（确定性，不依赖真实 data / 环境变量）
- `store::values`：改用**隔离临时目录**注入 `*_from` / `load_with_fallback`，覆盖
  - 列表可用性与默认标记；显式/默认/未知 id 加载
  - 快照**缺失**、**分母置 0**、**未知字段** → `available=false` 且加载回退 `builtin`
  - `values_dir_from_env` 两种写法（数据根 / `<root>/shuxing`）均命中 `<root>/values`
- `jpcg_combo`：新增跨入口一致性回归（省略分母时连招与单技能取同一值集）
- `jpcg_core::ffi_tests`：加串行锁（全局 `last_error` 并行覆盖导致偶发失败）

## 验证
- `cargo check`：core/combo/app（static+dynamic）通过
- `cargo test --workspace` 连跑 3 次全绿（combo 12 / const 5 / core 21 …）
- 前端 `tsc --noEmit` 通过

### 来自 changes/2026-10-09-080730.md

# fix(values): 快照修复后独立动态库自动恢复值集

日期：2026-10-09

## 改动与原因

PR #28 在快照缺失、TOML 损坏或系数非法时缓存内置兜底，却没有保存快照依赖。
当数据更新只修复快照、不改 `index.toml` 时，combo 独立动态库会继续命中失败缓存。

- `load_cached` 不再缓存失败结果，并移除同键的旧条目；合法快照仍使用原有 mtime 缓存。
- `CacheEntry.snapshot_path` 改为必需路径，消除无快照依赖却被判为有效的缓存状态。
- 保留内置回退和告警；不增加跨 cdylib 广播接口，不改变计算公式或数据版本。
- 取舍：仅故障状态下重试读盘/告警，优先保证修复立即可见；正常计算保留缓存性能。
- 增加缺失、语法损坏、非法分母三种恢复回归：预热失败读取，只修快照，确认 index mtime 不变，
  随后同 ID 自动恢复正式服常数，不调用显式缓存失效。

## 动态库回归

新增 `scripts/test-values-dynamic.py`（Python 3.11+，仅标准库）：真实加载 core 与 combo 两个 cdylib，
在缺失、TOML 损坏、非法分母三种场景预热两份缓存；修复时只写快照，断言 index 内容、inode、
大小、mtime 都不变。先检查 combo，再检查 core，均与显式正式服系数的完整计算响应比较。
目标 HP 设为 0，避开随机蒙特卡洛；缺少模块直接失败，不悄悄跳过。CI 已接入。

运行方式：

```sh
cargo build -p jpcg_core -p jpcg_combo
python3 scripts/test-values-dynamic.py
```

## 验证

- `cargo test -p jpcg_core -- golden`：9/9 通过。
- `cargo test -p jpcg_core -p jpcg_combo`：core 36/36、combo 12/12 通过。
- `cargo build -p jpcg_core -p jpcg_combo` + Python 双 cdylib 回归：缺失/语法损坏/非法分母三例全部通过。
- `cargo fmt --all -- --check`：通过。
- `make check-all`：通过。static/dynamic app、四个 cdylib 构建成功；workspace 59 个测试通过，
  dynamic app FFI roundtrip 1 个测试通过。
- 实际 CI 命令 `cargo clippy --workspace --all-targets`：通过，仅保留未改动的
  `crates/jpcg_combo/src/host.rs:77` 八参数既有告警。
- 同时执行 CONTRIBUTING 中的严格命令 `cargo clippy --workspace --all-targets -- -D warnings`：
  因上述既有 `too_many_arguments` 告警失败（rc 101），未将其记为通过，也未扩大修复范围。
  AGENTS/当前 CI 明确不将既有告警作为门禁。
- dynamic 构建仍有未改动的 `ffi_bridge.rs:315` 冗余 unsafe 告警。
- 完整矩阵后再次运行金标准、双 cdylib 回归、rustfmt 与 `git diff --check`，全部通过。

验证在 Linux 云环境完成，工具链和 Tauri 依赖安装在工作区用户目录，没有修改用户电脑。
本次未发布、合并或打 tag；远端新提交的 CI 仍需在提交后确认。

### 来自 changes/2026-10-09-080731.md

# fix(app): 存档加载后保持当前数值版本一致

日期：2026-10-09

## 改动与原因

PR #28 加载正式服存档时只改变表单，清空和数据更新仍从旧 localStorage 读取体验服选择，
导致当前请求、下拉选择和“实际使用”提示不一致。

以当前选择为运行期真源，持久化只用于启动恢复；加载、切换、清空和热更新沿用同一选择。
异步解析需要确认请求仍有效，避免旧响应覆盖更新选择；热更新继续保留用户修改或存档中的系数，
明确清空/切换时才重新套用值集常数。

- 分开跟踪选择操作与解析请求，防止切换、加载、热更新响应乱序覆盖新状态。
- 等待当前值集解析期间禁用计算/保存，避免新 ID 配旧系数提交；解析失败需重试成功后再提交。
- 旧存档无 `value_set` 时使用默认语义（null），清除旧持久化选集；默认选择不误报“不可用”。
- 新增 Vitest + jsdom（真实 React DOM、仅 mock 命令边界），`npm test` 接入前端 CI；
  lockfile 同步，既有依赖版本保持不变。

## 验证

在 `examples/jpcg_app/` 执行：

- `npm ci`：通过。
- `npm test`：16/16 通过，覆盖体验服→加载正式服→清空/更新、计算请求和“实际使用”提示，
  存档/手动系数保留、默认选择、失败加载恢复、初始化/加载/切换/热更新响应乱序、待解析提交阻止。
- 负向回归验证：临时替换为原 HEAD 组件时 16 例中 13 例失败；恢复修复版后重新全绿。
- `npm run build`（tsc + Vite）：通过；仍有既有产物体积提示（chunk > 500 kB）。
- `git diff --check`：通过。

本次为普通修复，不 bump 产品版本、不提前聚合发布 CHANGELOG。锁文件根包版本由过期的
`2.1.0-alpha.1` 同步到原 `package.json` 已有的 `2.1.0-alpha.2`，不是新版本升级。

### 来自 changes/2026-10-09-083600.md

# fix(app): 阻止新值集基线待解析时混入旧系数

日期：2026-10-09

## 问题与复现

提交 `5e11d3b` 的提交后复核发现：切换值集或清空后、解析完成前，表单仍保留旧系数。
此时修改一个系数会把全局自定义标记设为 true，令整个新值集的 seed 被跳过，形成
“新 ID + 一个手改系数 + 其余旧系数”的请求。计算/保存等待解析的门禁不能阻止这种混合状态。

先新增真实组件回归，在已发布实现上确认“切换待解析”和“清空待解析”两例都失败，
然后再修改实现。

## 修复与取舍

单独跟踪新值集基线是否待解析，在初始加载、切换和清空的基线未就绪期间禁用系数编辑，
同时在编辑处理函数中拒绝待解析期间的旧事件。成功解析后恢复编辑，避免将旧系数误作新集自定义值。

同一选集的数据热更新仍允许手动编辑并保留自定义系数；存档系数已构成完整基线，也保持可编辑。
不引入逐字段覆盖模型，不改数值计算或后端缓存，不扩大修复范围。

## 验证

- `npm ci`、`npm test`、`npm run build`：通过，20 个真实组件回归全部通过。
- 新增四例：切换/清空待解析时不混入单字段编辑；存档基线在来源解析期间仍可编辑；
  迟到的旧解析不能解锁，解析失败保持锁定直至重试成功。
- 原同选集热更新测试增加输入仍可编辑的断言，验证手工覆盖策略未退化。
- 补丁前的已发布实现明确失败前述切换与清空两例；修复后全部通过。
- 仍有既有 Vite chunk > 500 kB 提示。
- `make check-all`：完整通过，static/dynamic app、四模块构建成功；workspace 59 个测试、
  dynamic FFI roundtrip 1 个测试通过。
- 金标准 9/9、真实双 cdylib 三类快照恢复、rustfmt、普通 workspace/all-targets clippy、
  `git diff --check` 全部通过。
- 独立提交后复核再次运行 20 个前端测试并检查差异，原发现关闭。
- 全量重跑曾遭云执行连接中断，恢复后重新完成全部检查，没有把中断运行记为通过。
- 本轮没有 Rust 改动；既有 combo 参数过多及 dynamic 冗余 unsafe 告警仍在，
  严格 `-D warnings` 的既有失败情况见 `changes/2026-10-09-080730.md`。

验证仅在云工作区进行；未合并、打 tag 或发布。

### 来自 changes/2026-10-09-124157.md

# fix(update/data): 统一数据根（写读一致）+ 值集内存缓存 + 保存不再重置 buff/系数 + 快照路径校验

日期：2026-10-09

PR #28 自检发现的问题（先经临时探针**实测复现**再修，避免误判）。共修 4 项。

## 1. [高·既有 bug] 数据更新落盘位置 ≠ 运行期读取位置
**现象（实测）**：`download_and_install_data` / `check_data_updates` 用 `base_path.join("data")`，
而 `base_path = Path::new(".").canonicalize()`（**CWD**）。读取侧走 `exe_dir/data`、bundle
`Contents/Resources/data`、`JPCG_DATA_DIR`。探针：往 `CWD/data` 写好后，读者**完全忽略**
（`list_professions=0`、`value_sets` 仍为 `builtin`）。
安装版（Finder 启动 CWD=`/`）会把数据写到 `/data`（失败/写错位置），`values/` 永远回退内置，
`check_data_updates` 反复判定需更新。此 bug 在 P1 之前即存在。

**修复**：引入「数据根」单一真源，读写共用同一解析。
- `store::paths`：新增 `data_root_writable()`（更新落盘目标）与纯函数 `writable_root` / `read_roots`，
  并新增 `bundle_user_data(home)`（`~/Library/Application Support/com.qinthirteen.jpcg/data`，
  与连招预设目录同源）。`data_dir()` / `values_dir()` 改为基于 `read_root_candidates()`。
  - 写入：`JPCG_DATA_DIR` → bundle 用户目录 → `exe_dir/data`
  - 读取：`JPCG_DATA_DIR` → bundle 用户目录（更新落盘处，**优先**）→ bundle `Resources/data` → `exe_dir/data`
- `jpcg_update`：`check_data_updates` / `download_and_install_data` 改为显式 `data_root` 参数
  （去掉内部 `.join("data")`）；`check_updates` / `download_updates` 的参数语义改为 `data_root`
  （二者原本就只用于 data）。`all_updates`（CLI）与 update FFI 传 `<应用根>/data` 保持既有行为。
- `host::update`：`base_path()`（CWD）→ `data_root()`（`store::paths::data_root_writable()`）；
  `perform_app_update` 不再依赖 CWD（`fetch_app_update_info` 本就不用该参数）。

## 2. [中·既有 bug] 保存配置会重置 buff 与系数
**现象（实测）**：`store::save_config` 硬编码 `buff: default()` / `coefficient: default()`，
而前端 `saveConfig` 只传 player/hostile/xinfa → 一保存就把阵眼/奇穴增益与系数清零。
（`value_set` 实际**保留**：早前探针因输入缺必填段导致“解析失败”，属误判，已更正。）

**修复**：`save_config` 改为**保留**已存的 `buff` / `coefficient` / `value_set`（不再写默认值）。
真正的“保存用户改过的 buff/系数”需前端传参，属 P2 范畴。

## 3. [低] `index.toml` 的 `file` 可目录穿越
**现象（实测）**：`file = "../outside.toml"` 能读到 values 目录外（`huixin=9999` 加载成功）。
**修复**：`read_snapshot` 前置 `is_safe_relative_filename`（拒绝 `/`、`\`、`.`、`..`、空名），
非法即视为不可用并回退内置。

## 4. [优化] 每次计算读盘 → 内存缓存
**现象（实测）**：每次 `value_set_constant` 都读 index + 快照 + 校验，1000 次 ≈ 69ms。
**修复**：`store::values` 增进程内缓存（`Mutex<ValueCache>`，键含解析目录与 id）；
新增 `invalidate_cache()`，在 `host::update::perform_update` 数据落盘后调用，使新数据即时生效。

## 测试（确定性，不依赖真实 data / 环境变量）
- `store::paths`：`writable_root`（dev / bundle / env）、`env_data_root`、
  `read_roots`（bundle 下用户目录优先于 Resources）、以及「可写根 == 读取首选」的写读一致不变式
- `store::values`：`rejects_path_traversal_in_index`
- `jpcg_core::ffi_tests`：`save_config_preserves_buff_coefficient_and_value_set`

## 验证
- `cargo test -p jpcg_core -p jpcg_update -p jpcg_const -p jpcg_combo` 全绿（core 29 / update 6 / const 5 / combo 12）
- `cargo build-app-static` / `cargo build-app-dynamic` 通过
- `cargo fmt --check` / `cargo clippy -p jpcg_core -p jpcg_update` 无告警

### 来自 changes/2026-10-09-125428.md

# feat(values): P2 前端接线——数值版本下拉 + 系数 seed + 持久化（架构 C 落地）

日期：2026-10-09

架构 C 的「可更换数值」需前端真正生效，否则值集选择形同虚设。本笔补齐 P2，并顺带修复 2 个相关 bug。

## 1. 前端「数值版本」下拉（核心）
- `ConfigPanel` 顶部新增「数值版本」下拉：列出 `list_value_sets_cmd` 结果（不可用项禁用）
- 选中即调 `resolve_value_set_cmd`，用返回的**实际使用**值集（含回退）的常数**填充「系数设置」**
  （用户仍可再改），并写入 `form.value_set`
- 启动时：读取 `localStorage[jpcg_value_set]` → 命中且可用则用；否则用 `is_default`（体验服一测）；
  再否则第一个可用项。**App 默认数值因此从硬编码正式服 130 改为体验服一测**（架构 C 决策）
- 选择持久化用 localStorage（与 `lastXinfa`/`betaChannel` 同款）；`清空` 后重新套用当前值集

## 2. DTO：值集携带换算常数
- `jpcg_api::ValueSetDTO` 增 `coefficient: Option<CoefficientConfigDTO>`（快照可用时给出 8 个常数）
- `store::values`：`entry_to_dto` 接收已解析常数；`list_value_sets`/`resolve_value_set` 一次返回即可 seed

## 3. 全链路透传 `value_set`
- 前端 `CalculateRequest`/`FormData` 增 `value_set`；`toCalculateRequest` 携带
- `ComboPage` 调 `calculateCombo` 时透传 `req.value_set`；`OptimizePage` 经 `compute_derivatives`
  随 `CalculateRequest` 自动携带
- `api`：新增 `listValueSets()` / `resolveValueSet(id)`

## 4. 顺带修复的 bug
- **`calculateCombo` 参数名错误**：Tauri v2 命令参数为 **camelCase**（JS 侧 `forumUrl`→Rust `forum_url`），
  原代码发 `value_set`（snake_case）→ 被当作未知键忽略 → 连招的值集透传**一直是死代码**。
  改为 `valueSet`，连招现在真正随值集计算。
- **技能编辑保存写只读 bundle**：`store::save_skill_toml` 原写 `data_dir()`（安装版指向只读
  `Resources/data/shuxing`）→ 改为写 `data_root_writable()/shuxing`，与读取侧统一。

## 测试 / 验证
- `store::values`：`dto_carries_coefficient_when_available`（可用带常数 / 缺失为 None）
- `cargo test --workspace` 全绿（core 30 / combo 12 / update 6 / const 5）
- 前端 `tsc --noEmit` + `npm run build` 通过
- `cargo build-app-static` / `cargo build-app-dynamic` 通过

### 来自 changes/2026-10-09-132651.md

# fix(values): 修复 PigeonMuyz 复审（1×P1 + 4×P2）——多目录合并 / 保存一致性 / 初始化回退 / 热更新 / 双库缓存

日期：2026-10-09

针对 `76b6891` 的复审逐项修复。所有问题均**对照代码核实为真**后再修，并补确定性回归。

## 1. [P1] 安装版保存单个心法后，用户目录遮蔽其他随包心法
**问题**：`save_skill_toml` 在用户目录创建 `shuxing/A.toml` 后，`data_dir()` 按目录存在性选中
**整个**用户 `shuxing`，不再读 `Resources/data/shuxing` → 其他心法加载为空、门派列表只剩已保存者。

**修复**：改为**多目录合并读取**（用户覆盖优先，其后随包资源）：
- `paths::data_dirs()`：返回 shuxing 目录候选（存在者，按优先级）；`store::data_dirs` 导出
- `store::toml::load_config_from(dirs, profession)`：取**首个**存在该文件的目录
- `store::profession::list_from_dirs(dirs)`：先按文件名去重（先到先得）再分组 → 跨目录并集
- `engine::start_calculation_with_config`：在 `data_dirs()` 中定位文件
- 测试：`load_config_merges_across_dirs_user_first`、`list_merges_across_dirs_user_first`
  （资源有 A/B、用户仅 A → A 取用户、B 仍可加载、列表含 A/B）

## 2. [P2] 默认体验服的“保存→加载”变成体验服标签、正式服系数
**问题**：`handleSave` 不传 `coefficient`/`value_set`，后端按缺失文件取默认（LIVE130 + None）；
加载后 value_set 回用 prev（体验服）→ id 与系数不一致。

**修复**：保存时携带完整配置。
- `store::save_config(player, hostile, xinfa, buff, coefficient, value_set)`（不再从文件“保留”）
- `host::config::save_config` 增三参；core FFI `save_config` 请求增 `buff/coefficient/value_set`；
  Tauri `save_config_cmd` 增参（static/dynamic）
- 前端 `saveConfig` 发送 `buff`（`mode_is_point` 保布尔）、`coefficient`、`valueSet`
- `handleSave` 传 `form.buff` / `norm.coefficient` / `form.value_set`
- 旧存档兼容：缺 `value_set` → `None`（`#[serde(default)]`）；系数按存档恢复
- 测试：`save_config_persists_buff_coefficient_value_set`

## 3. [P2] 默认快照损坏时初始化未应用回退，仍发送正式服系数
**修复**：初始化与用户切换**统一走 `resolveValueSet`**：
- `resolveAndSeed(id)`：`resolveValueSet` 返回**实际使用**的值集（含 builtin 回退）与其常数 → seed
- 挂载时 `pickValueSetId`（localStorage→默认→首个可用→…）后 `resolveAndSeed`
- 下拉下方显示「实际使用：…（内置兜底 / 已回退）」

## 4. [P2] 数据热更新后前端仍把旧快照常数作为显式覆盖发送
**修复**：
- 数据更新成功后重新 `listValueSets()` + `resolveValueSet(当前)`：刷新列表、更新“实际使用”显示，
  且**未手动改过系数时**重新 seed（`coefficientCustomRef` 跟踪用户是否改过系数）
- `handleClear` 改为重新 `resolveAndSeed(当前)`（不再依赖可能过时的 `valueSets` 快照）

## 5. [P2] dynamic 模式仅清 core 缓存，combo 动态库继续用旧值集
**修复**：缓存改为 **mtime 校验**（`CacheEntry` 记 index/快照 mtime）——文件被数据更新改写后，
**每个 cdylib 各自**在下次读取时自动失效，无需跨库广播。
- 键仍为 (目录, id)；`cache_valid` 比对 index 与快照 mtime
- `list_value_sets` 去掉缓存（非热点）；新增 `load_cached(dir, id)` 便于测试
- 测试：`cache_reloads_after_snapshot_change`（同 id 快照改写后重载）

## 验证
- `cargo test --workspace` 全绿（core 33 / combo 12 / const 5 / update 6）
- `cargo build-app-static` / `cargo build-app-dynamic` 通过
- 前端 `tsc --noEmit` + `npm run build` 通过
- `cargo fmt --check` 通过；clippy 仅余既有告警（combo 8 参 / app unused_unsafe）

## [2.2.0-beta.1] - 2026-11-10

### 来自 changes/2026-10-16-064650.md

# fix(app): 加载配置时保留目标最大与当前血量

## 问题与触发

保存或导入含 `hostile.max_hp` / `hostile.current_hp` 的配置后点击「加载」，
`ConfigPanel.handleLoad` 手工重建目标属性时漏掉了这两个字段，导致输入框变空。
实际前端回归已复现：原 20 项测试通过，新加 4 项全部失败；补齐字段后 24 项通过。

调用链分析表明，计算请求归一化会把缺失值变成 0，连招引擎因此改走
`target_hp` 满血回退（若也为 0，则为未指定血量模式）。再次保存时，Rust DTO
同样会将缺失字段默认为 0，覆盖原存档的两项血量。此后端影响为源码核查，
本机没有运行 Rust 或真实 Tauri IPC，不将其描述为本机实测。

## 修复与范围

- `handleLoad` 恢复 `max_hp` 和 `current_hp`；旧响应缺字段时使用 `?? 0`，与既有 DTO 默认值一致。
- 回归覆盖受伤目标、显式 0、旧响应缺字段，以及 UI、计算 DTO 和再次保存的参数。
- 不修改伤害、血量或击杀率算法，不改依赖、数值数据、版本和更新流程。

## 验证

- 本机 `npm ci --ignore-scripts --no-audit --no-fund --cache /tmp/jpcg-npm-cache-20261016` 成功，锁文件未修改。
- 本机 `NODE_OPTIONS=--max-old-space-size=256 npm test -- --maxWorkers=1 --minWorkers=1`：修复前 4 失败 / 20 通过，修复后 24 通过。
- 本机 `NODE_OPTIONS=--max-old-space-size=256 npm run build`：TypeScript 与 Vite 通过；保留大于 500 kB 的 bundle 提示。
- `git diff --check` 通过。
- 未在本机运行 Rust workspace、金标准、clippy、双模式构建或桌面交互测试：该云环境没有 cargo/rustc。
- 前端组件测试 mock 了 API；使用真实 `toCalculateRequest` 校验 DTO。真实文件持久化、IPC 和 Rust 计算不在本机验证范围。
- 远端 CI 结果以本 PR 对应 head 的 GitHub Actions 为准。

### 来自 changes/2026-10-19-062940.md

# 修复迟到心法配置覆盖当前表单

## 什么与为什么

`ConfigPanel.handleXinfaChange` 异步读取心法配置时未判断请求是否过期。快速选择 A 再选择 B，若 A 较晚返回，界面仍显示 B，但 `xinfa_config` 被改回 A，计算 DTO 和保存参数随之使用 A。请求尚未返回时加载存档或清空也会被同一旧响应覆盖。

## 最小修复与关键决策

- 增加独立心法请求版本号，仅应用当前版本的响应。
- 心法切换、成功加载替换表单、清空及组件清理时使旧请求失效；不能只比较心法 ID，否则 A→B→A 的第一次响应仍会覆盖最新配置。
- 不改变正常配置内容、游戏数值算法、数值集规则、依赖、版本或 IPC 协议。
- 6 个确定性 deferred 回归覆盖乱序切换、切回同一 ID、加载及清空，以及加载期间切换和空加载保留有效请求；同时核对下拉、表单、真实 `toCalculateRequest` 转换及保存调用参数。

## 实际验证

- 基于 dev `b05680c3a230caf5a0b7761f503916229724a6dd`，本轮触发后取得隔离源码。
- `npm ci --ignore-scripts --no-audit --no-fund`：通过，锁文件未变。
- 实现未改时原 24 项及新增空加载对照通过、新增 5 项缺陷回归失败；修复后 30/30 通过。
- `NODE_OPTIONS=--max-old-space-size=256 npm run build`：TypeScript + Vite 通过，保留大于 500 kB 的构建 chunk 提示。
- `git diff --check`：通过。
- 测试命令：`NODE_OPTIONS=--max-old-space-size=256 npm test -- --maxWorkers=1 --minWorkers=1`。
- 组件测试 mock API，计算 DTO 转换是真实实现。Rust 按 `xinfa_config.profession` 选择技能的影响为源码调用链核查，不声称已实跑伤害计算或真实文件写入。
- 当前云环境没有 cargo/rustc，本机 Rust workspace、金标准、rustfmt、clippy、make check-all、双动态库及真实桌面/Tauri IPC 未运行。远端 CI 应按本次 head 单独核验。

### 来自 changes/2026-10-20-013600.md

# 计算结果增加技能搜索和排序（#75）

- 为计算结果增加技能名称子串搜索（忽略大小写和首尾空白），支持原始顺序、技能名称和技能期望 Q 的升降序，以及一键重置。
- 只在展示层对筛选后的新数组排序，不修改计算响应、DTO、引擎或数值。相同排序值保留原顺序。
- 先排序技能，再展开 DOT，每个技能的各跳保持原有连续顺序；明确说明使用技能 Q 值而非单跳 Q 排序。
- 显示筛选技能数/全部技能数、独立的无匹配提示；底部统计和最高期望标记仍基于全部技能，界面明确标注此范围。
- 重新计算和清空过程中保留筛选偏好，新响应使用当前筛选；可随时重置。不引入持久化或依赖。
- 补充组件测试覆盖搜索、组合/重置、数值/名称稳定排序、DOT、特殊伤害、空结果以及重算切换。

## 复审后小修（2026-10-20）

按 code review 与无头 Chromium 截图复核，做以下纯展示层调整，不改筛选/排序逻辑与数据契约：

- 排序语义文案改为「技能期望 Q（合计）」，hint 明确 DOT 行 Q 为单跳值、各跳连续、统计仍为全部技能；避免 q-desc 时首组 DOT 单跳值看起来「排序错」。
- 搜索框/排序下拉/重置按钮补 `:focus-visible`（teal 描边 + 2px 光晕，对齐既有 `.input:focus`）；按钮补 `cursor: pointer`、disabled 用 `not-allowed`；控件设最小宽度防「从高到低」被截断。
- 无匹配提示行使用 `emptyRow`（居中 + muted + 允许换行），不再沿用表体左对齐正文色。
- 行 key 由下标改为「技能名 + 跳序 + 下标」，降低行复用错位风险。
- 验证：`npm test` 32/32、`npm run build`（tsc + vite）通过；截图复跑见 `.scratch/pr77-shots/`。

### 来自 changes/2026-10-20-220000.md

# Makefile 一键流水线：npm 安装 → 前端构建 → Rust 双模式 + 4 模块库 + updater

- 日期：2026-10-20 22:00
- 分支：`dev`（工作区改动，未提交）
- 范围：`Makefile`、`.gitignore`、`.cargo/config.toml`（+ 本变更日志）

## 做什么

把原先只能逐个手工执行的构建矩阵，整理成「干净检出后一条命令」的 Makefile 流水线。

### 新增/重命名目标

| 目标 | 作用 |
|------|------|
| `setup` | 前端依赖安装：`npm ci`，缓存写入工作区 `$(CURDIR)/.cache/npm` |
| `frontend` | 前端构建：`npm run build`（tsc + vite → `examples/jpcg_app/dist/`） |
| `frontend-test` | 前端测试：`npm test`（vitest + jsdom） |
| `updater` | 编译 `jpcg_updater` 并复制到 `examples/jpcg_app/src-tauri/binaries/jpcg_updater-<host-triple>[.exe]` |
| `all` | 一键：`setup → frontend → modules-dir → updater → build-static`（结束：`jpcg_app`=static、`jpcg_app-dynamic`=dynamic 变体） |
| `fmt-check` / `clippy` | 分别对应 `cargo fmt --all -- --check` / `cargo clippy --workspace --all-targets` |
| `help` | 列出目标与可用变量 |

### 保留并增强的既有目标

- `build-static` / `build-dynamic` / `build-modules` / `modules-dir` / `test` 全部保留；
- `build-static` / `build-dynamic` 现在依赖 `frontend`（Tauri 的 `frontendDist` 在编译期嵌入，必须先有 `dist/`）；
- `frontend` / `frontend-test` 依赖 `setup`，干净检出可直接 `make build-static`；
- `modules-dir` 修复了原来 `cp` 源/目标同为 `target/$(BUILD)` 的无效复制：改为把 4 个模块库复制到
  `target/$(BUILD)/modules/`（`ffi_bridge` 动态加载顺序首选项 `exe同目录/modules/`），缺失任一库则报错退出；
  同时把构建出的 dynamic app 另存为 `target/$(BUILD)/jpcg_app-dynamic[.exe]`；
- `check-all` 扩展为超集：`fmt-check + clippy + build-modules + test（workspace 测试 + dynamic 库测试）
  + frontend-test + frontend + build-dynamic + build-static`，失败即停（make 默认语义），static 放最后。

### 环境适配

- `NPM_CACHE ?= $(CURDIR)/.cache/npm`，所有 npm 命令经 `npm_config_cache` 使用该缓存（本机 `$HOME` 只读，
  默认缓存目录不可写）；可用 `make NPM_CACHE=/path` 覆盖；
- `setup` 带增量跳过：`node_modules/.jpcg-setup-stamp` 新于 `package.json`/`package-lock.json` 时跳过
  `npm ci`（`FORCE_SETUP=1` 强制重装），干净检出时自动安装；
- `BUILD ?= debug` 语义保留，`BUILD=release` 时所有 cargo 目标带 `--release`、产物落 `target/release/`；
- 跨平台：按 `uname`/`OS` 识别 `.dylib`/`.so`/`.dll` 与 `.exe`，`<host-triple>` 由 `rustc -vV` 取；
- 新增 `CARGO_JOBS ?=`（透传 `-jN`），便于内存紧张时限制并行度；
- `.NOTPARALLEL: all check-all`：一键/全绿目标内部有产物顺序要求，即使外部传 `-j` 也强制其前置目标
  串行执行（避免 dynamic/static 同名产物交错覆盖）；
- Makefile 顶部改为完整的「一键流程 + 目标一览 + 依赖顺序 + 产物布局」说明；
- `.DEFAULT_GOAL := build-static`，保持原有裸 `make` 行为不变。

### 同步配置

- `.gitignore` 增加 `.cache/`、`.scratch/`（工作区本地 npm 缓存与临时目录，HOME 只读环境必需）；
- `.cargo/config.toml` 的 `build-modules` 别名补上 `jpcg_combo`，与 Makefile/README 的「四个模块库」一致。

## 为什么

1. **干净检出无法一条命令构建**：缺 `node_modules` 时前端无法构建，`tauri-build` 因 `frontendDist`
   不存在直接报错；缺 `jpcg_updater-<host-triple>` 时 `tauri` 打包因 `externalBin` 失败。原先这些前置步骤
   只散落在 README/CI 的手工命令里，容易漏。
2. **本机 `$HOME` 只读**：npm 默认把缓存/日志写到 `~/.npm`，会直接失败。必须显式把缓存放到工作区。
3. **模块库数量不一致**：Makefile 已含 `jpcg_combo`，但 `.cargo/config.toml` 别名仍缺它；dynamic 模式下
   combo 方法由独立 dll 提供，漏构建会表现为运行期功能缺失而非编译错误。
4. **对齐 CI**：`.github/workflows/ci.yml` 的顺序就是「npm ci/build → updater → fmt → clippy → build/test →
   前端 test/build」，Makefile 目标化后本地可一条命令复现同一条链路。

## 关键决策

- **前置依赖显式化**：`frontend: setup`、`build-static`/`build-dynamic`: `frontend`。这样
  `make build-static` 单独执行也能在干净检出跑通；代价是每次 app 构建会重跑前端构建（vite ≈3s）与
  按需 `npm ci`（stamp 命中时跳过）。
- **`setup` 增量跳过而非无条件 `npm ci`**：既保证干净检出自动安装，又避免每次构建都把
  `node_modules` 删掉重装（166MB / 10s 级）。`FORCE_SETUP=1` 保留「永远 npm ci」语义。
- **`modules-dir` 产出到 `target/$(BUILD)/modules/`**：原实现把 `target/$(BUILD)/libjpcg_*.so`
  复制到同一目录（实际是 `cp` 同文件，空操作）。改放 `modules/` 子目录既真实完成复制，又匹配
  `ffi_bridge` 的首选加载路径；4 个 dll 本身也仍在 `target/$(BUILD)/`（cargo 产物）。
- **static/dynamic 同名产物的「一键体验」**：两者都写 `target/$(BUILD)/jpcg_app`，后构建者覆盖前者。
  验收反馈指出 `make all` 结束在 dynamic 变体与「默认 static」直觉不符，故调整为：
  `modules-dir` 构建 dynamic 后先另存为 `jpcg_app-dynamic[.exe]`，`all` 把 `build-static` 放到最后，
  一键结束后 `jpcg_app`=默认 static、`jpcg_app-dynamic`=dynamic 变体，两种产物同时可用；
  单独 `make build-dynamic` 仍是「构建这一种模式」的语义（会覆盖 `jpcg_app`）。
- **`check-all` 保留构建语义**：旧 `check-all` 包含 static/dynamic/modules 构建，新目标列表是扩展而非替换，
  因此作超集保留；`build-modules` 放在 `test` 之前，让 dynamic 冒烟测试能真实加载 dll（而不是静默跳过）；
  `build-static` 放最后，结束时 `jpcg_app` 为默认 static。
- **不纳入 `npx tauri build`**：Makefile 继续只做 `cargo build`（与既有语义/CI 一致）；安装包打包仍由
  `release.yml` 的 Tauri CLI 负责，避免把前端嵌入语义在 Makefile 里再造一份。
- **static/dynamic 输出同名**：两者都写 `target/$(BUILD)/jpcg_app`，后构建者覆盖前者。`make all` 按任务
  约定顺序（先 static 后 modules-dir 内的 dynamic）结束，因此最终二进制是 **dynamic 变体**；需要 static
  产物时再跑 `make build-static`，或为两种模式使用不同的 `CARGO_TARGET_DIR`（已在 Makefile 顶部注明）。

## 实际验证与耗时

环境：Linux x86_64，cargo 1.99 / rustc 1.99 / node 22.22 / npm 10.9.4，`$HOME` 只读，64 核；
为控制内存并行度使用 `CARGO_JOBS=16`。日志在 `.scratch/logs/`。

### 1) `make setup frontend build-static`（从无 node_modules 状态起）

```
make setup frontend build-static     # EXIT=0，总耗时 79s
```

- `npm ci`（首次，缓存目录新建）：`added 188 packages in 10s`，工作区缓存 `.cache/npm` 41MB；
- `npm run build`：`tsc -b && vite build` 成功，dist ≈ 778KB（js 740.52KB / css 36.09KB），2.77s；
- `cargo build -p jpcg_app`（static，debug）：`Finished dev in 1m04s`。

### 2) `make all`（一键，增量复跑）

```
make all                             # EXIT=0，总耗时 30s（验收调整后的新顺序）
```

各步：`setup` 因 `package-lock.json` 刚被其他任务更新（stamp 过期）自动重跑 `npm ci`（1s，命中缓存）；
`frontend` 2.48s；`build-dynamic` 8.87s；`build-modules` 2.58s；`modules-dir` 复制 4 库 + 另存
`jpcg_app-dynamic`；`updater` 0.06s；`build-static` 最后 11.54s。首次按旧顺序运行时为 44s
（含 static 13.23s / dynamic 20.28s）。

产物核对（`BUILD=debug`）：

| 产物 | 路径 | 大小/说明 |
|------|------|-----------|
| app 本体（默认 **static**，最后落地） | `target/debug/jpcg_app` | 331,324,088 B；含 2408 个 `jpcg_core::` 符号 |
| dynamic 变体（modules-dir 另存） | `target/debug/jpcg_app-dynamic` | 315,315,128 B；0 个 `jpcg_core::` 符号，含 `libjpcg_core.so` 字符串 |
| 四个模块库 | `target/debug/libjpcg_{core,combo,update,const}.so` | core 121MB / combo 123MB / update 111MB / const 4.6MB |
| 模块目录副本 | `target/debug/modules/` 下同名 4 个 `.so` | 动态模式加载首选路径 |
| updater | `examples/jpcg_app/src-tauri/binaries/jpcg_updater-x86_64-unknown-linux-gnu` | 5.0MB（`externalBin` 命名） |
| 前端资源 | `examples/jpcg_app/dist/{index.html,assets/}` | 编译期嵌入 app |

static/dynamic 身份检查方法（比大小更硬）：

```sh
nm -C target/debug/jpcg_app         | grep -c 'jpcg_core::'   # 2408（静态链接了 core）
nm -C target/debug/jpcg_app-dynamic | grep -c 'jpcg_core::'   # 0（运行时 dlopen）
grep -c 'libjpcg_core.so' target/debug/jpcg_app-dynamic        # 1（dll 名）
```

### 3) `make check-all`（fmt + clippy + 模块库 + 双模式构建 + 全量测试 + 前端）

```
make check-all                       # EXIT=0，总耗时 43s
```

- `cargo fmt --all -- --check`：通过（无输出）；
- `cargo clippy --workspace --all-targets`：通过，仅 2 处既有 warning（`jpcg_combo/src/host.rs`
  参数过多、`jpcg_app` dynamic 分支冗余 unsafe），与 CI 的「不 -D warnings」策略一致；
- `cargo build -p jpcg_core -p jpcg_combo -p jpcg_update -p jpcg_const`：0.17s；
- `cargo test --workspace`：全部通过（12 + 5 + 36 + 6 个用例，其余 crate 0 用例；无失败/忽略）；
- `cargo test -p jpcg_app --lib --no-default-features --features dynamic`：1 passed
  （真实加载到 `target/debug/libjpcg_core.so`，未跳过）；
- `npm test`（vitest）：`Test Files 1 passed / Tests 24 passed`；
- `npm run build`：`✓ built in 2.18s`；
- `cargo build -p jpcg_app`（static）8.91s、dynamic 6.53s；末尾输出 `==> 全部通过`。
- 调整 `all` / `check-all` 顺序后复跑（`build-dynamic` 在前、`build-static` 最后）：EXIT=0，34s，
  结果同上；结束时 `jpcg_app` 仍为默认 static（2408 个 `jpcg_core::` 符号），`jpcg_app-dynamic` 保留。

### 4) 干净检出补充模拟（缺 dist / 缺 node_modules）

- node_modules：步骤 1 的 `npm ci` 是从「无 `node_modules`」状态安装（npm ci 本身先删除再安装），
  188 个包 10s，证明 lockfile 完整且工作区缓存可用；
- dist：把 `examples/jpcg_app/dist/` 整个移走，模拟干净检出缺前端产物后执行 `make build-static`：

```
mv examples/jpcg_app/dist .scratch/dist-backup
make build-static                    # EXIT=0，总耗时 11s
```

  `frontend` 依赖先自动执行 `npm run build` 重建 `dist/`（index.html + assets，js 740KB / css 36KB），
  随后 static app 编译 6.50s——证明「app 必须在 frontend 之后」的顺序依赖生效，未出现 tauri
  `frontendDist` 缺失错误。
- cargo 侧从零编译由 CI 干净 runner 覆盖（`cargo build --workspace`），本地为既有 target 增量验证。

### 5) `BUILD=release` 支持

`make -n BUILD=release all` 检查：所有 cargo 命令带 `--release`（`cargo build -p jpcg_app --release`、
`--no-default-features --features dynamic --release` 等），产物路径切换为 `target/release/`。

实跑（后台，`CARGO_JOBS=8`）：

```
make BUILD=release build-static modules-dir updater   # EXIT=0，总耗时 106s
```

- static app release：1m05s（release 依赖从旧缓存重建）；dynamic app release：27.44s；
- 四个模块 release：7.75s，已复制到 `target/release/modules/`；
- updater release：0.30s，已复制到
  `examples/jpcg_app/src-tauri/binaries/jpcg_updater-x86_64-unknown-linux-gnu`（522KB）；
- 产物：`target/release/jpcg_app` 22.7MB、`libjpcg_{core,combo,update,const}.so` 13.9/13.8/11.7/0.4MB。

### 6) `-j` 并行下的顺序保护

```
CARGO_JOBS=16 make -j8 all           # EXIT=0，总耗时 26s
```

输出顺序仍为 `frontend → build-dynamic → build-modules → modules-dir（另存 dynamic）→ updater →
build-static`，未出现 static/dynamic 交错；结束后 `jpcg_app` 331,324,088 B / 2408 个 `jpcg_core::`
符号（static），`jpcg_app-dynamic` 315,315,128 B / 0 个符号（dynamic）。`.NOTPARALLEL` 生效。

## 遗留/已知事项

- `make all` / `make check-all` 都会把同一 crate 以 static、dynamic 两种 feature 各编译一次（约多花
  ~20s），换来 `jpcg_app`（static）与 `jpcg_app-dynamic`（dynamic）两种产物并存。
- 单独 `make build-dynamic` 的语义仍是「只构建 dynamic 模式」，会覆盖 `target/$(BUILD)/jpcg_app`；
  要同时保留两种产物请用 `make all`，或先 `build-dynamic` 再 `build-static`。
- `all` / `check-all` 已加 `.NOTPARALLEL`，外部传 `-j` 时其前置目标仍按序串行，不会出现
  dynamic/static 同名产物交错覆盖。
- 首次 `npm ci` 需要网络；之后复用 `.cache/npm`。缓存目录已加入 `.gitignore`。
- 安装包（AppImage/deb 等）与 `modules_manifest.toml` 生成不在本次 Makefile 范围：前者由
  `release.yml` 的 `npx tauri build` + `deploy-gen` 负责，后者由 `manifest-gen` 负责。
- 其他文档（README/AGENTS/CONTRIBUTING 等）中的 make 目标说明由文档任务同步更新，本变更未触碰。

### 来自 changes/2026-10-20-220100.md

# docs: 全量文档更新到 v2.1.0 / 2026-10-20 真实状态

## 背景与范围

仓库文档长期停留在 v2.1.0-beta.1 / 私有仓库基线：README 版本与「本次更新」过时、
PLAN 的 M1 仍写「待执行」、AGENTS workspace/命令缺 `deploy-gen` / `json-to-toml` /
make 一键目标、CONTRIBUTING 与 PR 模板仍按旧 CI/分支保护描述、`server_manifest.md`
的服务器布局与 `data_version`/模块清单示例过期。本次把 7 个文档 + `.github/CODEOWNERS` 过时注释对齐到 2026-10-20 的实际状态。

事实来源（均为本轮重新核实，不依赖任务描述转述）：

- `gh release view v2.1.0`：publishedAt `2026-10-13`，assets 含三平台 app、
  `jpcg-stable-v2.1.0.tgz` + `.sha256`、9 个 dll、`data_manifest.toml`、`modules_manifest.toml`；
- `gh repo view`：`visibility=PUBLIC`、`isPrivate=false`、defaultBranch `dev`；
- `gh api repos/.../branches/{dev,beta,release}/protection`：必需检查 3 个
  （`Rust 构建 + 测试 + lint` / `金标准回归（计算数值）` / `前端构建 (tsc + vite)`）、
  1 review + code owner reviews、linear history、conversation resolution、禁强推/禁删除；
  `beta`/`release` strict，`release` require last push approval；
- `git log`：dev 头 `b05680c`（2026-10-18）；本地仅有 `v2.1.0-beta.1` tag（稳定 tag 由 Release API 核实）；
- `Cargo.toml`：workspace `2.1.0`、`jpcg_const=130.3.20260602`、`jpcg_updater=2.1.0`；
- 最新 `Makefile`（另一任务在本次执行期间更新）：目标 `setup/frontend/frontend-test/updater/all/check-all`、
  `NPM_CACHE ?= $(CURDIR)/.cache/npm`；`.cargo/config.toml` 的 `build-modules` 已含 `jpcg_combo`；
- `server_tools/` 实际目录：`manifest-gen`、`forum`、`deploy-gen`、`json-to-toml`；
- `crates/jpcg_update` URL 常量与 `server_tools/deploy-gen` 编排逻辑；
- GitHub Release 资产 `data_manifest.toml` / `modules_manifest.toml` 实际格式；
- CHANGELOG 第 5869 行起 `[2.1.0] - 2026-10-13` 段（18 条 changes 记录）。

## 各文件改动

- **README.md**：版本/日期改 `v2.1.0 / 2026年10月`；删除无依据的「Android 适配中」；
  「本次更新」按 CHANGELOG `[2.1.0]` 提炼为 5 条真实亮点（值集外置、更新链路自动化、
  macOS aarch64、数据读写一致性、发布工程质量）；原 alpha.2 内容降级为「往期版本记录」；
  未来计划标注稳定版已完成；补「make 一键构建」流程并与 Makefile 实际目标/依赖顺序对齐；
  核心功能补值集切换/热更新说明；Workspace 速览补 `data/shuxing`、`data/values`、server_tools 四工具；
  往期版本记录标题由开发线版本 `v2.1.0-alpha.2` 改为 CHANGELOG 实际发布段
  `v2.1.0-beta.1`（2026-09-04，日期取自 CHANGELOG），并移除 diff 块中不可核实的「alpha.1 起已包含」表述；
  `make all` 顺序/产物与 Makefile 最终版对齐（setup → frontend → modules-dir → updater → build-static；
  结束 `jpcg_app`=static、`jpcg_app-dynamic`=dynamic），`modules-dir` 产物路径由「应用同目录」更正为
  `target/<build>/modules/`，并补 dynamic app 另存说明。
- **PLAN.md**：M1 标记完成并写入实际保护配置（1 review+CODEOWNERS、3 checks、linear、
  conversation resolution、beta/release strict、release last-push-approval）；
  状态表改为 Public + v2.1.0 + 保护已配置；M2 注明「暂无新授权进展」（不虚构）；
  M3 改为「已转公有，进行中」；收尾文案标注部分完成（README 徽章/SECURITY.md 未补）；
  决策记录追加 2026-10-20 核实条目；M0 引用的已聚合 `changes/2026-08-11-231500.md` 改为指向 CHANGELOG 条目。
- **AGENTS.md**：workspace 表补 `server_tools/deploy-gen/`、`server_tools/json-to-toml/`；
  修正 `jpcg_const` 描述（它是 `jpcg_core` 等级常数真源，被 core/app 依赖，并非 unused；
  依据：`crates/jpcg_core/Cargo.toml:20-21`（注释 + `jpcg_const.workspace = true` 依赖）、
  `examples/jpcg_app/src-tauri/Cargo.toml:28`（app 直接依赖）、
  代码使用 `crates/jpcg_core/src/type_set/coefficient.rs:31/39/54/71` 的 `jpcg_const::level_constant::LIVE_130`）；
  Commands 段补 make 一键目标、frontend-test、check-all 实际组成，补 deploy-gen/json-to-toml 用法；
  修正 `.cargo/config.toml` 别名说明（`build-modules` 含 combo，与 Makefile 一致）；
  版本 bump 改为 `release.sh` / 手改根版本 + sync-version，并注明 `cargo set-version` 会因
  `jpcg_const` 独立版本降级失败；开源状态改为 Public + 三分支保护；新增 `data/values/` 值集 Key fact；
  版本模型补 `data_manifest.data_version` 语义；模块更新补 deploy-gen 合并 `platform="multi"`；
  移除已过时的「更新服务器清单手工 3 步」（现由 release.yml 的 package job 自动调用 deploy-gen）；
  `make all` 顺序/产物与 `make modules-dir` 产物（`target/<build>/modules/` + `jpcg_app-dynamic`）
  按 make-dev 更新后的最终 Makefile 对齐。
- **CONTRIBUTING.md**：新增「分支保护现状」小节（检查名/CODEOWNERS/strict/last push approval）；
  验证命令对齐 CI（clippy 不 `-D warnings`；前端 `npm test` + build）；版本表修正
  `data_version` 语义（= `vX.Y.Z` 发布 tag，由 manifest-gen 写入，shuxing 内 `[version]` 另为
  `{level,season,modified}`）；bump 改为 release.sh/手改根版本，明确不要用 `cargo set-version`；
  发布流程补 prep 分支 + PR + tag + deploy-gen tgz 链路。
- **server_manifest.md**：新增「客户端 URL 约定」表（对照 `jpcg_update` 实现）；
  部署 stage 改为 `deploy-gen` 实际输出（stable 版本目录含 `modules/`、`data/`；beta 根式含
  update.toml/manifest/modules/data）；`update.toml` 的 `data_version` 由 `v2.0.2026050201`
  更正为实际 `v2.1.0`；模块清单补真实字段示例（`modules_version`/`platform="multi"`/version/sha256/size）
  与三平台 9 个 dll；更新器改为 host-triple 后缀 + `make updater`；数据清单补实际覆盖
  （readme.md、shuxing/*、values/*）；macOS 错标历史说明由具体 `beta.1` 改为「早期 Beta 产物」（保留事实、去掉过期版本串）。
- **scripts/README.md**：受保护分支处理补实际保护规则；sync-version 补 `package-lock.json`
  同步与 `NPM_CACHE`；修正 release.sh 文件表与 alpha 阶段「push dev」表述（实为推 prep 分支 + PR）；
  阶段示例版本号由 `2.1.0-alpha.n`/`2.1.0-beta.n` 泛化为 `X.Y.Z-alpha.n`/`X.Y.Z-beta.n`（X.Y.Z 以根 Cargo.toml 为准）。
- **.github/PULL_REQUEST_TEMPLATE.md**：目标分支 `develop` → `dev`（受保护/squash）；
  组件版本项补 `jpcg_updater` 与 `data_manifest.data_version`；验证项对齐 `make check-all` 新语义、
  clippy 不 `-D warnings`、前端 `npm test` + build。
- **.github/CODEOWNERS**（按 lead 补充要求）：过时注释「master/develop 分支保护」→「dev/beta/release 分支保护」；
  owner 规则 `* @QinThirteen` 未改动。
- **changes/2026-10-20-220100.md**：本文件。

## 决策与取舍

- 只改 7 个文档 + `.github/CODEOWNERS` 注释 + 本 changes 文件；未动 Makefile / `.cargo` / `.gitignore` /
  `package-lock` / `sync-version.sh` / CHANGELOG / 源码（这些由 task-1、task-3 负责），未提交 git。
- 不重排 CHANGELOG 历史；README 「本次更新」只从 `[2.1.0]` 段提炼，不引入未记录功能。
- make 命令段在读取**最新** Makefile 后写成实际目标（`setup/frontend/frontend-test/updater/all/check-all`），
  未使用任务描述中的占位命名。
- M2 数据授权未取得进展，PLAN 中不虚构授权状态。
- `data_version` 语义以 `manifest-gen` 实现为准更正，避免继续传播「等级.赛季.日期」的旧说法。
- 版本串处理原则：已发布/CHANGELOG 可查的版本按 CHANGELOG 段落引用（含日期）；
  未单独发布、只在开发线存在的 `2.1.0-alpha.2` 不再作为 README 历史标题；
  仅表达阶段约定处保留 `alpha.n` / `X.Y.Z-alpha.n` 这类通用写法。
- 构建命令/产物描述以 make-dev **最终** Makefile 为准复核（`all:`/`modules-dir:` 实际依赖与命令），
  不沿用调整前的旧顺序/路径。

## 验证

- `git diff --stat`：本轮 8 个文件（7 文档 + `.github/CODEOWNERS` 注释）+ 新增 changes 文件；
  `git diff --check` 通过（无空白错误）。
- `make -n all` / `make -n check-all` 已逐项对照 make-dev **task-1 完成后**的最终 Makefile
  （make-dev 已 inactive，md5 `e932c64da37955cb7b7db47020d2251e`；`all:` / `modules-dir:` / `check-all:` 依赖行），
  `make -n all` 实测顺序为：npm run build → dynamic app → 4 模块库 → 复制 `target/debug/modules/` 并另存
  `jpcg_app-dynamic` → updater → static app（最终 `jpcg_app`）：
  `all` = setup → frontend → modules-dir（build-dynamic + build-modules，dll 复制到 `target/<build>/modules/`，
  并把 dynamic app 另存 `jpcg_app-dynamic`）→ updater → build-static（最终 `jpcg_app` 为 static）；
  `check-all` 展开 fmt-check → clippy → build-modules → test → frontend-test → frontend → build-dynamic → build-static；
  与 README/AGENTS/CONTRIBUTING/PR 模板描述一致（CONTRIBUTING 无 `make all` 产物描述，check-all 描述为顺序无关表述，无需改）；
  未运行真实构建/测试。
- grep 版本串（范围：README / PLAN / AGENTS / CONTRIBUTING / server_manifest / scripts/README / PR 模板；
  模式 `alpha.`、`beta.1`、`v2.0.`、`v2.1.0-beta`、`[0-9]+\.[0-9]+\.[0-9]+-(alpha|beta)`）逐一核实：
  过期具体版本串已全部清除；剩余匹配仅为阶段约定写法（AGENTS 分支说明、CONTRIBUTING 分支图/版本阶段、
  scripts/README `X.Y.Z-alpha.n` 示意）与 README 中带 CHANGELOG 日期的历史段 `v2.1.0-beta.1`，均非过期声明；
  无 `Android`、无「私有仓库/待执行」旧表述、无 `cargo set-version` 正向用法（仅「不要用」警示）、
  无 `-D warnings` 正向命令、无 `v2.0.2026050201` / `v2.0.X` 旧示例。
- 未在本机跑完整构建/测试（本轮为纯文档改动；Makefile 行为由 task-1 实跑验证）。

## 不确定 / 待确认

- `server_manifest.md` 中「服务端把 `modules/`、`data/` 以 `/files/` 暴露」的落位映射，
  依据是 `jpcg_update` 客户端 URL 常量与 `deploy-gen` 输出结构；服务器端 nginx/部署脚本
  不在本仓库，未直接验证。
- PLAN M1 收尾项（README 徽章、`SECURITY.md`）仍缺失，本次只记录状态、未新增文件。
- 本地 git 未 fetch 稳定 tag `v2.1.0`（仅 `v2.1.0-beta.1`）；发布事实以 GitHub Release API 为准。

### 来自 changes/2026-10-20-220200.md

# chore(meta): 元数据对齐 —— package-lock 版本回同步 + commands.ts mock 修复 + sync-version.sh 扩展

日期：2026-10-20

`examples/jpcg_app/package-lock.json` 的根 `version` 仍停留在 `2.1.0-alpha.2`，
与 `package.json`/根 `Cargo.toml`/`tauri.conf.json` 的 `2.1.0` 不一致；且
`scripts/sync-version.sh` 不处理 package-lock，版本 bump 后 lock 会持续漂移。本次只做元数据对齐，
**不改任何构建文件，不升级任何依赖**。

lead 复核后追加二轮补充：`commands.ts` 的 `get_module_versions` mock 陈旧且形状缺字段，
一并修复并纳入 `sync-version.sh` 同步范围（详见 §2.1 与 §3）。

## 1. package-lock.json 版本同步（A1）

- 现象：lock 顶层 `"version": "2.1.0-alpha.2"` 与 `packages[""].version` 同为旧值，而 `package.json` 已是 `2.1.0`。
- 修复：优先用 npm 官方方式重新生成 lock：
  ```sh
  cd examples/jpcg_app
  npm install --package-lock-only --ignore-scripts --no-audit --no-fund \
    --cache /compile/workspace/jpcg/.scratch/npm-cache
  ```
- 结果：`git diff` **只有 2 行**（顶层 version + `packages[""].version`），
  无任何依赖版本/resolved/integrity 变化；`package.json` 的 `dependencies`/`devDependencies`
  版本范围与 lock `packages[""]` 完全一致（校验脚本确认无 pkg-only / lock-only / range-diff）。
- 决策：`--package-lock-only` 不安装依赖、不动 `node_modules`、不跑生命周期脚本，是同步 lock 根版本的最小侵入方式；
  缓存指到工作区 `.scratch/npm-cache`，适配 `$HOME` 只读环境。

## 2. 版本一致性审计（A2）

版本源与实测值（截至本次修改）：

| 来源 | 值 | 结论 |
|------|----|------|
| 根 `Cargo.toml` `[workspace.package].version` | `2.1.0` | 版本源（release tag / 安装包命名） |
| `crates/jpcg_api` / `jpcg_core` / `jpcg_combo` / `jpcg_update` / app / forum / manifest-gen / deploy-gen / json-to-toml | `version.workspace = true` → `2.1.0` | 继承，一致 |
| `crates/jpcg_const` | `130.3.20260602` | **设计上独立**（等级.赛季.日期），不改 |
| `crates/jpcg_updater` | `2.1.0` | **设计上独立**，本次恰好等于 workspace 版本，不改 |
| `Cargo.lock`（11 个 workspace 成员） | 与 Cargo.toml 全部一致 | `cargo metadata --locked --no-deps` 退出码 0，**未手改** |
| `examples/jpcg_app/package.json` | `2.1.0` | 一致 |
| `examples/jpcg_app/src-tauri/tauri.conf.json` | `2.1.0` | 一致 |
| `examples/jpcg_app/src/api/commands.ts` `current_app_version` / `latest_app_version` | `v2.1.0` | 一致（sync-version.sh 维护） |
| `examples/jpcg_app/src/api/commands.ts` `get_module_versions` mock | `core/update/combo/app = 2.1.0`、`const = 130.3.20260602` | **本次修复**（原 core/update/app 为 `2.1.0-alpha.1`，且缺 `combo`） |
| `examples/jpcg_app/package-lock.json`（顶层 / `packages[""]`） | `2.1.0` | **本次修复**（原 `2.1.0-alpha.2`） |

### 2.1 commands.ts `get_module_versions` mock 修复（lead 复核追加）

- 问题：mock 返回 `core/update/app = "2.1.0-alpha.1"`（陈旧）；且 Rust
  `commands/version.rs::ModuleVersions` 与 TS `src/types/index.ts::ModuleVersions` 都是 **5 个字段**
  （`core/update/const/combo/app`），mock 只有 4 个、**缺 `combo`** —— 形状与真实命令不一致
  （mock 返回类型是 `Promise<unknown>`，TypeScript 不会报错，属静默漂移）。
- 修复：
  - `core/update/app` → `2.1.0`；补 `combo: "2.1.0"`；`const` 保持独立版本 `130.3.20260602`；
  - 字段顺序与 Rust struct 一致（core、update、const、combo、app），便于对照。
- 依据：`jpcg_core::CORE_VERSION`/`jpcg_update::UPDATE_VERSION`/`jpcg_combo::COMBO_VERSION` 与
  app 的 `CARGO_PKG_VERSION` 都继承 workspace 版本（2.1.0）；`jpcg_const::CONST_VERSION` 独立。
- 该修复为一次性源码修正；后续版本 bump 由 `sync-version.sh` 维护（见 §3）。

未修 / 仅报告项（均超出本次写作用域，或属产品元数据决策，未改动）：

1. **`README.md:123` 标题仍为「📦 版本记录 (v2.1.0-alpha.2)」**，与 `2.1.0` 不符；属 README 写作用域，未改。
2. `tauri.conf.json` 的 `identifier=com.qinthirteen.jpcg` 正常；`bundle` 未配置 `publisher`/`copyright`/`license`
   等发布元数据（仅影响安装包属性，不影响构建），本次不建议改。
3. `examples/jpcg_app/src-tauri/Cargo.toml` 仍保留模板值 `description = "A Tauri App"`、`authors = ["you"]`；
   建议后续统一（属 app 源码写作用域）。
4. `license = "GPL-3.0"` 不是严格 SPDX 标识（规范写法 `GPL-3.0-only` / `GPL-3.0-or-later`）；
   改它会影响 `cargo set-version`/crates.io 语义，需单独评估，本次不动。
5. 全仓无 `rust-version`、无 `rust-toolchain.toml`（AGENTS 明确依赖 rustup default）；edition 2024 要求
   rustc ≥ 1.85，建议将来在根 `[workspace.package]` 加 `rust-version = "1.85"`，属根 Cargo.toml 写作用域。
6. `.github/dependabot.yml` 三个生态（cargo/npm/github-actions）`target-branch: dev` 与
   CONTRIBUTING 的「dev 为最上游集成分支」模型一致，**仍合理**，不改。

## 3. scripts/sync-version.sh 扩展（A3）

- 新增第 4 步：同步 package-lock.json。
  - 优先 `npm install --package-lock-only --ignore-scripts --no-audit --no-fund`（`cd examples/jpcg_app` 执行）；
    若设置了 `NPM_CACHE` 则追加 `--cache "$NPM_CACHE"` 并 `mkdir -p`（适配只读 `$HOME`）。
  - npm 不存在/执行失败时回退到 **python 最小编辑**：只改 `version` 与 `packages[""].version` 两个字段，
    不重排 lock，不触碰依赖；回退方案实测输出与 npm 路径**字节一致**。
  - npm 跑完后用 python 校验两个 version 字段确实等于目标版本；不满足则走最小编辑兜底，最后不会静默留下旧版本。
- 扩展第 3 步（commands.ts）：
  - 除已有的 `current_app_version` / `latest_app_version`（`v<ver>`）外，**限定在
    `case "get_module_versions": return { ... };` mock 段内**同步 `core`/`update`/`combo`/`app`
    为 `<ver>`（无 v 前缀）；`const` **显式不动**（独立版本）。
  - 只匹配该 mock 段，避免误改文件其他位置同名键；mock 段形状被重构（正则不匹配）时
    **只向 stderr 告警、不中断脚本**（其余版本同步仍有效），作为结构漂移的 fallback。
- 幂等：脚本在已同步状态下重复执行，四个输出文件（package.json、tauri.conf.json、commands.ts、
  package-lock.json）sha256 完全不变；模拟「陈旧 mock」后执行可逐字节恢复期望内容。
- 头部注释同步更新（新增 package-lock 行、`NPM_CACHE` 用法、commands.ts 模块版本）。
  `scripts/release.sh` 已以 `scripts/sync-version.sh "$VERSION"` 调用，发布流程自动获得 lock 同步，
  无需改 release.sh。

## 4. 验证

1. 先把 lock 还原为 `2.1.0-alpha.2`，运行 `NPM_CACHE=.scratch/npm-cache scripts/sync-version.sh`
   （不带参数，从根 Cargo.toml 读 `2.1.0`）：
   - `package.json` / `tauri.conf.json` / `commands.ts` **无 diff**（JSON 重序列化字节稳定）；
   - `package-lock.json` 仅 2 行变化（顶层 + `packages[""]`）。
2. 幂等：紧接着再跑一次，上述四个文件 sha256 全部一致。
3. 回退路径：构造不含 npm 的 `PATH`（仅 python3/bash/dirname/mkdir）运行脚本：
   输出「(最小编辑)」，diff 与 npm 路径字节一致；再跑一次同样幂等。
4. `bash -n scripts/sync-version.sh` 语法检查通过。
5. `cargo metadata --locked --no-deps` 退出码 0 → `Cargo.lock` 与 `Cargo.toml` 一致（未手改）。
6. commands.ts mock 同步（二轮补充）：
   - 一次性修复后 `git diff` 为预期 4 行（core/update/app 三个版本串 + 新增 combo）；
   - 人为把 mock 四段改回 `2.1.0-alpha.1` 后运行脚本 → 与修复后文件**逐字节一致**（const 未变）；
   - 再跑一次 → 四文件 sha256 不变（幂等）；
   - 用改名 mock 段模拟重构 → 脚本 stderr 告警、exit 0、其余同步继续执行；
   - 隔离副本（symlink 主工作区 node_modules，未写主工作区）`tsc -b` + `vite build` 均 exit 0。

## 5. 最终改动范围

- `examples/jpcg_app/package-lock.json`：2 行（version 字段）
- `examples/jpcg_app/src/api/commands.ts`：`get_module_versions` mock 4 行（3 个版本串 + 补 `combo`）
- `scripts/sync-version.sh`：新增 package-lock 同步步骤与 `NPM_CACHE` 支持 + 扩展 commands.ts mock 同步

未提交 git、未切换分支；未触碰构建脚本 / README / tauri.conf / 其他 tracked 文件；
tauri.conf 发布元数据、Cargo.toml 模板值、SPDX 许可、rust-version 等仅报告、不修改（属产品元数据决策）。

### 来自 changes/2026-10-20-220300.md

# docs(meta): 新增 npm vs bun 对比与迁移评估报告

日期：2026-10-20

在 A 段元数据对齐完成后，对「是否应从 npm 迁移到 bun」做了一次有真实实测数据的调研，
产物为 `docs/npm-vs-bun.md`（新建 `docs/`）。本变更**只新增文档，不改默认工具链**。

## 背景

- 本机已安装 bun（`/home/qin13/.bun/bin/bun`，1.4.2），具备做真实对比实验的条件；
- JPCG 前端（Tauri v2 + React 19 + Vite + Vitest）目前用 npm（CI `npm ci`，本地 `make setup`）；
- 需要回答：bun 是否值得迁移，收益/风险如何，若试点需要改哪些地方。

## 实验设计（隔离、可复现）

- 复制 `examples/jpcg_app` 到工作区 `.scratch/meta-dev/bench/{npm-app,bun-app}`，**不触碰主工作区
  `node_modules`**；`$HOME` 只读，npm 用 `--cache`、bun 用 `BUN_INSTALL_CACHE_DIR` 指向工作区，
  每个样本独立空缓存。
- npm 侧用 CI 实际命令 `npm ci`；bun 侧先验证从 `package-lock.json` 自动迁移的行为，
  再用 `bun install --frozen-lockfile` / `bun ci` 计时。
- 依赖一致性：安装后逐一读取两侧 `node_modules/**/package.json` 的 `name→version` 全量对比。
- 兼容性：两侧分别执行 build 与 vitest，并验证 `npx tauri` / `bun x tauri`。

## 关键实测结论（详见 docs/npm-vs-bun.md）

| 指标 | npm | bun |
|------|-----|-----|
| 冷装（空缓存，2 次） | 8.8 s / 13.5 s | 12.3 s / 9.1 s |
| 暖装（删 node_modules，缓存命中） | 2.64 s | 0.14 s / 0.08 s（`bun ci` 0.15 s） |
| 构建 / 测试 | 5.84 s / 2.79 s（24 passed） | 5.45 s / 2.89 s（24 passed） |
| node_modules | 165 MB / 9,723 文件（137.4 MiB 表观） | 165 MB / 9,722 文件（137.3 MiB 表观） |
| 缓存 | 41 MB（压缩 tarball） | 165 MB（解包副本；同盘硬链接，额外占用 ~1 MB） |
| lockfile | `package-lock.json` 119,763 B | `bun.lock` 69,872 B |
| 安装结果 | 188 包 | 188 包，**版本完全一致（无升级）** |
| Tauri CLI | 2.11.2 | 2.11.2（`bun x tauri` 可用） |
| audit | 8 漏洞（2C/4H/2M），exit 1 | 12 漏洞（2C/6H/4M），**exit 0** |
| 生命周期脚本 | 默认执行（esbuild postinstall 跑） | 默认阻止（blocked 1: esbuild），build/test 仍通过 |

值得记录的发现：

1. 冷装两者**同量级**，小项目上未复现 bun 官方「快 ~7×」；暖装 bun 快一个数量级（缓存硬链接复用），
   但 CI 总时长由 Rust 构建主导，前端安装节省 2–3 s 无实质影响。
2. bun 1.4.2 在「只有 `package-lock.json`、无 `bun.lock`」时，`bun install --frozen-lockfile`
   **仍写入了 `bun.lock`**（与当前文档描述不一致），CI 若用它需加 `git diff --exit-code` 守卫。
3. `bun.lock` **不记录根 package version**，若迁移则 `scripts/sync-version.sh` 的 package-lock 同步步骤可移除。
4. `bun audit` 有漏洞时退出码仍为 0，不能直接替代现有 `.github/workflows/deps.yml` 的 `npm audit` 门禁。
5. Dependabot 的 bun **version updates 已 GA，但 security updates 尚未支持**（GitHub 2025-02-13 changelog），
   这是维持 npm 的关键理由之一。

## 建议（写入文档 §9）

- **短期维持 npm**：收益（本地暖装）小于成本（CI/发布/Makefile/Tauri 配置/文档/Dependabot 全链路改动 + 安全更新缺口）。
- **可选试点 bun**：独立分支/非必需 CI job，三平台跑 `bun ci + build + test` 收集数据，不阻塞主线。
- **不推荐**当前全量迁移。
- 文档给出 14 项「试点/迁移检查清单」（锁单一化、trustedDependencies、缓存 key 含 OS、tauri before 命令、
  Makefile/只读 HOME 缓存、Dependabot/Audit 门禁、三平台矩阵、回滚方案等）。

## 变更文件

- `docs/npm-vs-bun.md`（新增，唯一代码库变更；`docs/` 目录为新建）
- `changes/2026-10-20-220300.md`（本文件）

未改依赖、未改 CI/Makefile/tauri 配置、未提交 git。

### 来自 changes/2026-10-22-064900.md

# 技能编辑器隔离心法加载，防止跨心法保存

- 切换心法时同步清空旧数据和选中技能；加载期间禁止保存。
- 加载请求使用递增版本；切换与卸载失效旧请求，旧成功、失败和 finally 均不得覆盖当前状态。
- 根因：旧数据仍可编辑/保存，且迟到响应没有所属检查；保存使用新 profession 路径却携带旧数据，可能覆盖另一心法文件。
- 新增六个确定性组件回归：加载期间保存、失败后保存、乱序成功、旧失败、A→B→A、卸载迟到失败。原实现 6 fail / 38 pass；修复后 44/44 通过。
- npm 锁定安装、TypeScript 与 Vite 构建通过；无依赖、版本、计算数值或安全权限修改。
- 测试 mock API；未实跑 Tauri/文件写入。本机无 cargo/rustc，Rust/金标准/完整双模式测试未本机运行。新建配置既有行为不在本修复范围。

### 来自 changes/2026-10-23-130902.md

# make 一键流程在编译 app 前准备 updater sidecar

- 根因（review P2）：Tauri 的 build script（tauri-build 2.6.3 `src/lib.rs`）在**每次 cargo 编译 app 时**都会校验
  `frontendDist` 并复制 `bundle.externalBin` 指向的 sidecar（`copy_binaries` → `copy_file(&src, &dest)?`），
  缺任一项即在普通 `cargo build` 阶段失败，而非只在 `tauri build`/打包阶段。原 Makefile
  `all: setup frontend modules-dir updater build-static` 把 `updater` 排在 `modules-dir`（内部 `build-dynamic`
  会编译 app）之后；`check-all` 更是以 `clippy` 打头（clippy 同样编译 app build script）。干净检出后
  `make all` / `make check-all` / `make build-static` 都会因缺少 `binaries/jpcg_updater-<host-triple>` 失败。
- 修复：Makefile 新增变量 `APP_BUILD_DEPS := frontend updater`，并作为 **所有会编译 app 的目标** 的前置：
  `build-static`、`build-dynamic`、`clippy`、`test`。`all` 顺序改为
  `setup → frontend → updater → modules-dir → build-static`（updater 先于任何 app 编译）。
  `test` 另前置 `build-modules`，让 dynamic 冒烟用例在单跑 `make test` 时也能真正执行（原注释即要求先构建 dll）。
- 回归守卫：新增 `scripts/check-make-order.sh`，对 `make -n`（dry-run）计划做顺序断言——
  `npm run build` 与 `cargo build -p jpcg_updater` 必须出现在首条 app 编译命令
  （`cargo build -p jpcg_app` / `cargo clippy --workspace` / `cargo test --workspace`）之前；
  覆盖 `all` / `check-all` / `build-static` / `build-dynamic` / `test` / 默认 goal 六个入口。
  `.github/workflows/ci.yml` 的 rust-checks 中新增一步执行该守卫（dry-run，不产生构建产物、不依赖网络与 GUI 依赖）。
  验证：对修复前的 Makefile 运行该脚本，6/6 目标全部报错；修复后 6/6 通过。
- 同步更新文档中的顺序描述：Makefile 头注释与 `make help`、`README.md` 一键构建段、`AGENTS.md` Commands。
- 兼容性：默认 goal `build-static` 现在也会先 `npm ci`/`npm run build` 与编译 updater——干净检出即可成功；
  已有产物时 setup 走 stamp 跳过、cargo 走增量缓存，额外开销为秒级。`make all` 的最终产物布局不变
  （`jpcg_app`=static、`jpcg_app-dynamic`=dynamic、`modules/` 4 库）。
- 未修改 Rust 代码、计算数值、依赖版本、FFI/ABI、数据/manifest 与 CI 必需检查名。

### 来自 changes/2026-10-28-072329.md

# fix(core): 保留连招预设技能快照的伤害属性

## 问题与修复

- 连招预设保存通过 `ComboStep::from` 调用 core 的 `skill_dto_to_skilltype`，
  但该转换漏拷贝 `dot_interval`、`dot_duration`、`dot_up`、`wushijianshang`、
  `zhenshishanghai`，使这些字段在技能快照中变成默认零值。
- 加载预设时会还原快照中的零值。例如莫问「商（dot）」的间隔 2 秒、持续 18 秒、
  每跳递增 0.12 丢失后，连招引擎无法按原来的 9 跳计算，走单跳回退。
- 补齐五个字段的直接拷贝，与即时连招计算的转换保持一致；不调整公式、游戏数据、
  DTO 或存档格式，也不改变 overrides 的语义。
- 新增 DTO → ComboStep → TOML → DTO 回归，使用非零伤害属性并比较全部技能池字段，
  避免默认值掩盖丢失。测试不读写文件、不修改全局状态，不涉及蒙特卡洛。

## 兼容性与边界

- 旧版已保存为零的字段无法从快照自动恢复；需要重新从技能池添加相应技能并保存。
  本次不加入推断式迁移。
- `wushijianshang`、`zhenshishanghai` 的验证范围是属性保真，不宣称其当前公式数值效果。
- 回归覆盖保存/加载使用的类型转换与 TOML 序列化，不是完整 Tauri/FFI/磁盘端到端测试。

## 验证

- 先仅添加回归，执行 `cargo test -p jpcg_core --lib --locked -j1 preset_roundtrip_keeps_damage_fields`：
  断言失败，输出显示上述五字段全部被归零；补齐五行后同命令通过。
- `cargo test --workspace --exclude jpcg_app --locked -j1`：60 个单元测试全部通过
  （core 37、combo 12、const 5、update 6）。
- `cargo test -p jpcg_core --locked -j1 -- golden`：9/9 通过。
- `cargo fmt --all -- --check`、`bash scripts/check-make-order.sh` 通过。
- `cargo clippy --workspace --exclude jpcg_app --all-targets --locked -j1` 通过；
  仅报告既有的 combo `calculate_combo` 参数数量警告。
- `cargo build --workspace --exclude jpcg_app --locked -j1` 通过，包含四个模块库；
  `python3 scripts/test-values-dynamic.py` 的 missing/malformed/invalid 三种双 cdylib 回归均通过。
- 前端重新 `npm ci` 后，`npm test` 通过（3 个文件、44 个用例），`npm run build`
  的 TypeScript 与 Vite 构建通过；保留既有的大 chunk 提示。
- 本地完整 `make check-all`、含 app 的 workspace 构建/测试/clippy 与 app dynamic 测试未运行：
  环境缺少 GTK/WebKit 系统依赖；完整桌面验证由 PR CI 承担，结果以对应 head 的 CI 为准。

### 来自 changes/2026-10-28-121201.md

# chore(app): 升级 Vite 8 与 @vitejs/plugin-react 5

## 背景

- dependabot #84 单独把 `vite` 从 6.4.3 升到 8.3.5，CI 在 `npm ci` 阶段
  `ERESOLVE`：`@vitejs/plugin-react@4.7.0` 的 peer 仅支持 `vite ^4.2 || ^5 || ^6 || ^7`。
  该 PR 已关闭，改由本 PR 做协同升级。
- Vite 8 用 Rolldown / Oxc 替换 esbuild / Rollup，并改用 Lightning CSS 压缩样式，
  属工具链大版本迁移，不能当作普通 patch 依赖升级处理。

## 改动

- `vite` `^6.3.5` → `^8.3.5`
- `@vitejs/plugin-react` `^4.4.1` → `^5.2.0`
- `package.json` 增加 `engines.node: "^20.19.0 || >=22.12.0"`（Vite 8 与 plugin-react 5 的下限）
- 重新生成 `package-lock.json`：
  - 移除 `esbuild`、`rollup` 及全部平台二进制包
  - 新增 `rolldown`、`lightningcss`、`@oxc-project/types` 及平台二进制包
  - 传递依赖随 plugin-react 5 更新：`react-refresh 0.17→0.18`、`postcss`、`nanoid`、`source-map-js` 小版本
  - 新解析均走 `registry.npmmirror.com`，与 lockfile 现有主流来源一致
- 不修改 `vite.config.ts`：当前只使用 `plugins`、`server`、`clearScreen`，未涉及被移除/改名的
  `build.rollupOptions`、`esbuild`、`optimizeDeps.esbuildOptions` 等配置
- 不修改业务代码、计算逻辑、数值数据、版本号或更新流程

## 关键决策

- 选 `@vitejs/plugin-react@5.2` 而不是 6.x：5.2 的 peer 已声明支持 `vite ^8.0.0`，
  无需引入 `oxc-transform-react`、`@rolldown/plugin-babel` 等额外依赖，迁移面最小。
- 不额外安装 `esbuild`：项目没有使用 `transformWithEsbuild`，也没有配置
  `build.minify: 'esbuild'` / `build.cssMinify: 'esbuild'`，不需要兼容回退。
- 接受 >500 kB 的单 chunk 警告：升级前产物体积同级（主要来自 recharts 等），
  本次不改变分包策略；如需优化，后续单独引入 `build.rolldownOptions.output.codeSplitting`。
- Node 下限通过 `engines` 声明而非 CI 改动：CI 的 `node-version: 20` 会解析到 20.20.x，满足要求。

## 验证

- `rm -rf node_modules && npm ci` 成功（与 CI 相同的安装路径，锁文件一致）。
- `npm run build`：`tsc -b` + `vite 8.3.5` 通过（609 modules，748.86 kB / gzip 224.41 kB）。
- `npm test`：3 个测试文件、44 个用例全部通过。
- `git diff --check` 通过。
- 未在本机运行 `tauri dev` / `tauri build` 的桌面交互烟测；前端构建产物与 CI 路径一致，
  桌面端行为以 PR 的 CI 结果和后续手工验证为准。

### 来自 changes/2026-10-28-150416.md

# chore(release): 启动 2.2 开发，版本 bump 至 2.2.0-alpha.1

## 改动

- 根 `[workspace.package].version`：`2.1.0` → `2.2.0-alpha.1`；
  `version.workspace = true` 的成员（api/core/combo/update/app/forum/manifest-gen/
  deploy-gen/json-to-toml）随继承生效，`Cargo.lock` 已刷新
- `scripts/sync-version.sh` 同步 `package.json`、`src-tauri/tauri.conf.json`、
  `src/api/commands.ts`（`check_update` 与 `get_module_versions` 模拟串）以及 `package-lock.json`
- `jpcg_const`（`130.3.20260602`）与 `jpcg_updater` 为独立版本，本次不变
- 不改动业务代码、计算逻辑、数值数据与更新流程

## 关键决策

- 按三分支模型，`dev` 即 alpha 集成分支；本分支 `prep/2.2.0-alpha.1` 仅用于版本落地 PR，
  alpha 阶段不打 tag（与 `scripts/release.sh alpha` 一致）
- 本次不在 alpha.1 聚合 CHANGELOG：`changes/` 中仍保留 2.1.0 系列已发布条目，
  `release.sh` 的全量聚合会产生重复章节；留待 beta 阶段按 `v2.1.0` 之后的文件过滤后处理
- 不新增 `CHANGELOG.md` 章节，避免把 2.2 开发起点误标为一次内容发布

## 验证

- `cargo metadata --locked --format-version 1` 通过（锁文件与清单一致）
- `cargo fmt --all -- --check` 通过
- `cargo test -p jpcg_core -- golden`：9/9 通过
- 前端 `npm ci`、`npm run build`、`npm test`：3 个文件 / 44 个用例全部通过
- 未在本机运行 `make check-all` 与桌面 `tauri dev/build` 烟测，交由 PR CI 覆盖

### 来自 changes/2026-10-30-073241.md

# chore(ci): 升级 Node 22 与 jsdom 30（#79）

## 背景

- CI 仍使用 Node 20，不能满足已锁定的 Vitest 5 与待升级 jsdom 30 的运行要求。
- jsdom 30.1.3 声明 Node `^22.22.2 || ^24.15.0 || >=26.0.0`；不能把 Node 22.22.0 视为支持的最低版本。

## 改动与决策

- `ci.yml` 两处 Node 20 → 22；`release.yml` 与 `deps.yml` 同步为 22，统一依赖安装环境。
- jsdom `^26.1.0` → `^30.1.3`，更新其传递依赖与锁文件；其余直接依赖的锁定条目保持原样。
- `engines.node` 与 README 同步为 jsdom 的支持范围，CI 保留 Node 22 主版本选择，自动接收该分支补丁。
- 锁文件使用 npm 更新，并保留未升级平台包的既有 `libc` 元数据，避免 npm 10 重写产生无关变动。
- 基于 dev `3a410a19d5e610d93c61b8045562e533ebabc62d`；已核对 #90、#91、#92 与 PR #89，未修改业务组件、核心算法、时序模拟、数据或组件版本。
- 当前检出中不存在 `.agents/skills`；遵循根 `AGENTS.md` 与 `CONTRIBUTING.md`。

## 验证

- 升级前 Node 24.19.0：前端 3 文件 / 44 测试与 `tsc -b && vite build` 通过。
- 升级后 Node 22.23.4：`npm ci --engine-strict --no-audit --no-fund`、44 测试、生产构建、`npm ls --depth=0` 通过。
- 最低支持版本 Node 22.22.2：相同严格全新安装、44 测试与生产构建通过；`npm ci` 未改变锁文件（SHA-256 校验）。
- Node 22 两版本均使用 nodejs.org 官方 tarball，并核验官方 SHA-256。
- `cargo test --workspace --exclude jpcg_app --locked -j4`：59 单元测试通过（core 36 / combo 12 / const 5 / update 6）。
- `cargo test -p jpcg_core --locked -j4 -- golden`：9/9 通过。
- `cargo fmt --all -- --check`、`bash scripts/check-make-order.sh` 通过。
- `cargo clippy --workspace --exclude jpcg_app --all-targets --locked -j4` 通过，仅既有 combo 参数数量告警。
- `cargo build --workspace --exclude jpcg_app --locked -j4` 与 `python3 scripts/test-values-dynamic.py` 三种双 cdylib 回归通过。
- 所有 workflow YAML 可解析，4 个 setup-node 入口均使用 Node 22；`git diff --check` 通过。

## 验证限制

- 已实际运行 `make check-all CARGO_JOBS=4`；完整 workspace clippy 因环境缺少 GLib/GObject（另缺 GTK/WebKit）开发库失败，未覆盖完整桌面 static/dynamic 构建与 app 测试。完整 Linux 检查由 PR 的 CI 安装系统依赖后验证。
- 前端构建仍有既有 >500 kB chunk 提示，产物与升级前一致；本次不改变分包策略。
- 未触发发布或部署；release workflow 只核对配置与其前端安装/构建路径，未执行三平台打包。

### 来自 changes/2026-10-30-074207.md

# feat(gear): #90 配装模板、合成 API 与有界优化入口

- 新增独立 `jpcg_gear` crate（workspace 通配自动纳入）和 `gear-opt` CLI，提供四文件严格 TOML schema、12 槽位合成、JSON API、纯 player DTO 转换、约束优化、基线差值和属性生存 Pareto。
- 规则涵盖三来源、两档、固定绿字与饰品两套混搭、唯一性、御劲攻击显式录入、裤子不参与防具套、跨来源2+2、2/4与饰品2/3奖励同时生效。覆盖文件做完整属性替换，升级不做公式推导。
- 所有装备/套装数值为明确标记的合成样例（synthetic-v1），不宣称正式游戏数据；目录版本与值集必须匹配。已有 manifest-gen 递归收集数据，不改数据发布逻辑。
- 期望目标复用 core 的同一 q_cal，击杀目标复用 combo host 并固定种子；不改时序、加速、共享 DTO、UI、workflow 或 PR #89 的预设修复。
- 明确范围：有界穷举仅在 complete=true 时证明全局最优；大空间预算截断不得宣称全局最优或满足1秒验收。支持组合间进度与取消；加速不参与重排，未应用4件CD时击杀目标默认拒绝，显式允许才做属性侧计算并警告。
- 混档采用“每阈值按升级件数选择升级/基础奖励，单阈值不重复叠档”的可测试策略，待维护者确认；不是已核实的游戏规则。
- 正式数据、真实人工基准、全规模高效算法、CD时序、UI/动态桥接仍为后续依赖；该提交不关闭 #90。契约与运行方式见 `crates/jpcg_gear/README.md`。
- 新增回归验证模板/套装/约束/溢出/错误信封、TOML往返、CLI退出码、Pareto，以及与现有core/combo数值一致、固定种子确定性。

## 验证证据

- `cargo test --workspace --exclude jpcg_app --locked -j2`：77项（core36/combo12/const5/update6/gear18）通过；`cargo test -p jpcg_core --locked -- golden`：9/9。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --exclude jpcg_app --all-targets --locked`、`bash scripts/check-make-order.sh` 通过；clippy仅既有combo参数数量告警。
- 前端 `npm test -- --run` 44/44、`npm run build`通过，无前端源码修改。
- release构建和可复现 `scripts/bench-gear.py`通过：16组合，期望40506→40544，中位2.283ms；50k采样/1合成步击杀率0.09724→0.09724，中位3.862ms。此为样例微基准，不是全规模性能或真实场景验收。
- 本机 `pkg-config --modversion gtk+-3.0 webkit2gtk-4.1` 确认缺GTK/WebKit开发包，未执行完整make check-all与app静态/动态构建，交由同head的远端CI验证。

### 来自 changes/2026-10-30-074722.md

# #91 第一阶段：手动驾驶台与全局持久化

## 改动与原因

- 将手动表单、心法/敌人/Buff/系数/数值集、最近计算及其输入上下文、钉住基准集中到 App 的本地状态。计算结果与当前编辑稿分开，编辑后提示重算，避免把旧结果钉到新属性上。
- 新增 `usePersistentState`，沿用 `STORAGE_KEYS` + localStorage，使用 schema 版本封套、v0 无封套迁移、结构校验与异常回退；存储不可用时保持内存编辑，换 key 时防止不同心法串写。
- 复用 ConfigPanel 手动表单并支持受控输入；重启/恢复时保留保存的自定义系数与心法修正，避免初始化异步请求覆盖。归一化表单时保留 `mode_is_point` 布尔类型，提交 core 的现有归一化语义不变。
- 对缺失的心法名称/主属性元数据使用现有有效值回退，避免基准栏展示字面 `undefined`；收紧窄屏结果区宽度，表格保持内部横向滚动。
- 连招草稿按心法持久化：完整序列与每步 overrides、名称、所选预设、搜索与折叠状态。只有用户明确点保存时才写已保存预设；切页、重算与切心法不会自动覆盖预设。
- 页面首次访问后保持挂载；切心法时重建连招视图并读取对应草稿。加载/清空配置的心法直接来自全局表单，避免原独立 `currentXinfa` 状态不同步。
- 新增钉住/取消/恢复操作，基准记录实际计算输入与结果；恢复同时撤销后续属性编辑，丢弃在途计算的迟到响应。结果表增量增加每技能 Q（DOT 为逐跳 Q）的绝对/百分比差值，绿加红减；零分母显示破折号。

## 边界与决策

- 本次为 #91 的独立基础阶段，**不关闭 #91**。当前仅手动模式；12 部位配装卡片、双模式切换、配装合成/自动重算、数值过渡/占比动画、配装 TOML 导入导出与性能预算，等待 #90 稳定契约再接入。
- 不新增共享 DTO，不修改 `src/types/index.ts`、API 命令、Rust core/combo、数据、workflow 或依赖锁文件；不重做 PR #89 与 #79（现 PR #93）。
- 开工与提交前均检查远端开放 PR、分支及 #92 评论，尚无可识别的 #92 提交/分支供逐文件对比，不能证明其未推送工作无冲突。本次潜在前端交叠是 `App.tsx`、`ComboPage.tsx`、`ConfigPanel.tsx`；未触碰其时序核心/共享 DTO，后续需按 #92 的实际 patch 对齐。
- 旧结果 DTO 无技能 id，基准按「技能名 + 原始出现次数」匹配，排序不影响对应；基准缺项显示破折号。上游若调整同名技能顺序，应重新钉住，待稳定契约提供标识后再强化匹配。
- 前端本地状态类型放在 `state/cockpit.ts`，不是 Rust/Tauri/FFI 契约。旧收藏 key 保持不变。

## 验证

- Node 24.19.0：前端 Vitest **57/57**（原 44 + 新增 13），`npm run build`（tsc + Vite）通过；保留既有 >500 kB chunk 提示。
- 安装最初受锁文件 npmmirror 地址限制；临时缓存副本仅将下载主机换到官方 registry，版本/integrity 不变，随后仓库原锁文件的 `make setup` / `npm ci` 成功；仓库 lockfile 无改动。
- Playwright + 系统 Chromium：计算→钉住→差值→刷新恢复→恢复基准、连招步调整/草稿名刷新恢复、跨心法草稿隔离；1440px/390px 截图。使用现有浏览器 mock，未验证 Tauri 原生桥接。控制台保留既有 mock 重复技能 key 警告与 favicon.ico 404。
- `cargo test --workspace --exclude jpcg_app --locked -j2`：59 项通过；`cargo test -p jpcg_core --locked -- golden`：9/9。
- 非 app workspace clippy/build（locked）通过，仅既有 combo 参数数量告警；`python3 scripts/test-values-dynamic.py` 三种双动态库回归通过。
- `cargo fmt --all -- --check`、`bash scripts/check-make-order.sh`、`git diff --check` 通过。
- 完整 `make check-all CARGO_JOBS=2` 已运行，在 workspace clippy 因缺 `glib-2.0` 开发库失败，环境亦缺 GTK/WebKit；完整桌面构建/测试及 app dynamic 测试需 CI 验证。#79 的 Node/jsdom 工作由 PR #93 提供，本分支不混入。

### 来自 changes/2026-10-30-075408.md

# fix(gear): 拒绝缺失套装奖励数据

- 复核 #90 规则发现：原schema允许防具省略4件CD或把2件奖励填0，饰品也允许零奖励。这会把未录入的数值当成完整套装数据，击杀目标可能漏报未应用CD。
- 收紧为：防具必须有非零2件属性与显式4件CD；饰品2件化劲、3件御劲均必须为正。缺数据时报错，继续保留所有样例的synthetic标记，不填造正式游戏数值。
- 在既有非法目录测试中添加四种缺奖励用例；先运行该测试红，再修复为绿。优化公开入口在读取运行期数据前先校验Catalog。
- 不改引擎/共享DTO/UI/工作流。更新契约说明；配装18项、workspace非app测试、fmt/clippy与release微基准重跑验证。

- 修复后样例目标值不变；16组合的5次CLI测量中位数：期望2.430ms，50k采样击杀率4.948ms。

### 来自 changes/2026-10-30-083247.md

# #91 第二阶段：配装目录、桌面双模式桥接与驾驶台集成

## 基线与合入前置

- 上游 #94 固定为 `84afe7a9e4476a05fe6e9009786838fc520067d5`，#95 为 `d409fa91b102924754b914bc3bd540696123e7c5`。
- `integration/91-gear-base` 仅组合上述原提交，`feature/91-gear-cockpit` 在此之上开发。后续草稿 PR 指向集成基线，差异只包含第二阶段，不重复评审上游更改；没有合入 dev/beta/release。
- **不可直接合入该集成基线或 dev**：#94/#95 必须先完成评审合入 dev，再将本分支自集成基线之后的提交 rebase 到最新 dev，核对差异、移除 CI 的临时集成 PR 基线与配装分支 push 过滤并重跑检查，再改 PR 目标。
- #92 仍未发现可识别的远端 patch，无法证明其未推送工作无冲突。潜在交叠在 App/ComboPage/ConfigPanel/API 命令；没有改 combo 时序核心、共享 jpcg_api 或数据规则。#79 的 Node/jsdom 改动和 PR #89 均不混入。

## 实现与决策

- `jpcg_gear` 在 #94 实际 Loadout/Catalog/Composed 契约上增加宿主入口，前端模板目录返回应用 overrides 后的完整目录；只接受配装值或 TOML 内容，不接受文件路径。数据根复用 core 的读取规则，支持根/shuxing 环境路径，损坏或版本不匹配明确报错。
- 合成响应 `Composition {composed,player}` 的伤害属性转换复用 #94 `player()`，不在 JS 重写公式，不把手动总属性当裸装点数。导入导出均经 Rust 校验，可往返。
- 新增 stateless gear cdylib ABI 1、panic 防护和配对字符串释放。gear 的 `engine` feature 默认开启；关闭时只编译模型，dynamic 桌面壳经 libloading 调用，引擎不被静态链接。静态/动态 Tauri 命令调用同一宿主逻辑，阻塞文件操作移到后台任务。
- 第五个模块加入 Makefile/cargo aliases、发布资产与 deploy-gen 白名单；CI 增加真实 ABI 冒烟及临时叠放基线与 feature 分支 push 触发入口（PR 创建审核超时时仍可验证同一提交），没有触发发布。Tauri bundle 资源显式列入 gear 四个模板文件，确保首次安装不依赖先下载更新。
- 前端 12 卡片平铺：来源/档位/饰品属性套只取实际模板，可展开全属性与参考装分。新安装默认配装并采用匹配值集，旧 v1 数据迁移仍保持手动模式。手动属性与按心法的配装选择分开保存，切换/刷新保留。
- 125ms 防抖 → Rust 合成 → 既有伤害计算；版本/值集不匹配不提交，迟到响应不覆盖新输入，重复点击当前模式不会取消在途请求。结果表更新期间保留旧数据，增加技能池占比、变化行提示、摘要数字过渡与 reduced-motion 支持。
- 连招草稿在全局共享；排轴器配置草稿，驾驶台调用原 calculateCombo 计算伤害/击杀率。钉住基准包含配装、上下文、手动输入、连招草稿/结果；恢复会撤销后续改变并防止自动重算覆盖快照。
- 修复实际技能池同 id/sub_id 多条展示时的 React key 重复警告，仅使用列表出现序区分渲染，不改变收藏身份或技能/时序语义。
- 样例、加速与套装 CD 的限制明确展示：synthetic-v1 不是游戏数值；加速/CD 不宣称已接入 #92。

## 验证与限制

- 前端 Node 24.19.0：63/63 测试、tsc + Vite 构建通过。新增覆盖 12 卡片、模式保留、防抖、迟到合成、基准/刷新恢复、错误保留、TOML 宿主调用、驾驶台连招与草稿/击杀率恢复。fixture 由真实 gear cdylib 读取 #94 合成样例生成，仅测试使用。
- 非 app workspace：80 项（原 59 + gear 18 + host 1 + 实际 desktop bridge 2）通过；金标准包含在其中。clippy 仅既有 combo 参数数量告警。
- `cargo test -p jpcg_gear --no-default-features --test dynamic_bridge` 通过，编译的是桌面实际 bridge 文件，调用真实 gear DLL；`cargo tree` 证明 dynamic 壳不含 jpcg_core/jpcg_combo 引擎依赖。
- C ABI 脚本验证根/shuxing 路径、缺目录、空指针、未知路径字段、版本不匹配与 TOML 往返；既有双动态库回归保持不变。
- Playwright：生产前端 + 本机 HTTP 验证适配器 + **真实 core/gear/combo cdylib**，通过新安装默认模式、12 槽、自动计算、基准、双模式/刷新、真实 TOML 往返、连招配置与计算位置、1600px/390px 截图。非原生 WebView；桌面 Rust bridge 另有直接测试。控制台仅 favicon.ico 404。
- 样例计时：12 次饰品切换，从页面 click handler 到新计算快照持久化，P95 约 200.1ms（125ms 防抖在内，生产前端/debug Rust/HTTP 适配器）。Playwright 控制端的完整 click/wait 墙钟 P95 约 356ms；调试 React + 串行适配器初测约720ms。此处不声称已完成原生 Tauri、多心法大目录或 release 性能/60fps 验收。
- 本地完整桌面检查仍受 GLib/GTK/WebKit 开发库缺失限制；需本 PR head 的完整 CI 补齐。保留既有 Vite 大 chunk 提示。正式数据、混档规则确认、CD/加速时序与真实场景性能仍依赖上游/维护者。

### 来自 changes/2026-10-30-195338.md

# 时序引擎（timeline）首版：形态 / GCD 通道 / 充能 / 读条 / dot / 定时 buff

## 做什么

`jpcg_combo` 新增 `src/timeline.rs`（#92 引擎切片），在不动旧 `engine.rs`（按步累加，兼容层）的前提下，
提供「带时间轴的连招模拟」：

- 公共类型：`FormDef`（形态：id / 名字 / 基础 GCD / 初始形态 / 可用技能）、
  `OnCast`（施放副作用：切形态、额外占用的 GCD 通道、锁技能）、
  `TimingSkill`（技能 + 所属形态 + GCD 通道 + 副作用）、
  `TimelineAction::{Cast, SwitchForm}`、`TimelineInput` / `TimelineEvent` / `TimelineResult`。
- `simulate(&TimelineInput)`：按动作序列贪心排时间轴，输出事件流（施放 / dot 跳 / 形态切换 / buff 生效）、
  总伤害、窗口 DPS、被跳过的动作数。

## 关键实现

- **加速**：`jiasu_at(t)` 汇总玩家加速等级 + 当时活跃 buff；受上限来源与可突破上限来源分开，
  按 `final = min(capped, 25%) + uncapped` 合并（`timing::combine_jiasu`）。
  GCD 用 `timing::quantize_secs`（**不再二次夹 25%**，否则突破上限的加速会被截断两次）。
- **GCD 通道**：`lane_ready: HashMap<String, f32>`；技能默认落在 `default` 通道，
  `FormDef.gcd_base` 让不同形态走不同通道（双持 1.0s / 单持 1.5s 互不阻塞）。
- **读条 / 引导**：起手时刻为施放开始，`hit_time = start + cast_time_with_haste(rate)`（16 帧量化）。
- **CD / 充能**：单层走 `skill_ready`；多层走 `charges{剩余层, 下次恢复, 间隔}`。
  受加速影响（`cd_haste`）时按 `cooldown/(1+rate)` 连续缩短（是否帧量化待游戏数据确认）。
- **dot**：施加时快照进攻侧（玩家 / 心法 / buff / 系数 / 技能），并按当时加速率把总时长量化后均匀排成逐跳时间点；
  每跳结算用**当前**目标减伤重算（`DotInstance::per_tick_damage` → `JpcgConfig::q_cal()` 的对应跳）。
- **定时 buff**：技能 `buff_id` 引用 `TimedBuffDef`，施放后进入 `active_buffs`；
  刷新 / 叠层按定义；提供 `BuffEffect`（攻击 / 会心 / 会效 / 破防 / 无视防御 / 伤害 / 加速率）。
- **形态切换**：`OnCast.switch_form` 或手动 `TimelineAction::SwitchForm`；引擎不写任何门派分支。

## 单测（8 个，全部通过）

单 GCD 链 0/1.5s、双形态独立通道、断云额外占用双持通道 + 锁孤锋破浪到 1.5s、
读条 20.02% 加速 → 19 帧 = 1.1875s、dot 9 跳 2..18s 且总伤 = `q_cal` 的 dot 之和、
充能 0/1.5/8s、25% 封顶 + 突破上限 buff → 1.125s、手动 SwitchForm。

## 验证

- `cargo test -p jpcg_core`：45 passed
- `cargo test -p jpcg_combo`：20 passed（含新增 8 个 timeline 用例）
- 旧 `engine.rs` 与金标准未改动，兼容层保持原行为。

### 来自 changes/2026-10-30-202145.md

# 时序引擎 (B)：数据层 combat overlay + DTO/FFI/Tauri 打通 + 前端时间轴预览

## 做什么

在 #92 引擎切片 A（`timeline.rs` 纯时序模拟）之上，把「手工填写的战斗数据」接成可分发、可前后端调用的一条链路：

- **数据层**（`data/combat/{门派}.toml`，json-to-toml 不覆盖）：
  - `crates/jpcg_core/src/type_set/combat.rs`：`CombatData`（`meta` / `[[form]]` / `[[skill_combat]]` / `[[buff]]` / `[[miji]]` / `[[qixue]]`）、
    `FormDef`、`OnCast`、`SkillCombatDef`（可选覆盖 `cast_time` / `gcd_base` / `gcd_kind` / `cooldown` / `charges` / `recharge` / `cd_haste` / `channel` / `buff_id`）、
    `validate()` / `validate_against_skills()` / `apply_overlay()`。
  - `crates/jpcg_core/src/store/combat.rs`：`load_combat` / `load_combat_from`（首个存在文件生效）/ `load_skills_with_combat`（套 overlay + 交叉校验）。
  - `store/paths.rs::combat_dirs()`：`JPCG_DATA_DIR`（数据根或 `shuxing/`）→ `<root>/combat`，否则 `read_root_candidates()` 各目录 + `/combat`。
- **DTO 单源**（`crates/jpcg_api`）：`FormDefDTO` / `OnCastDTO` / `SkillCombatDTO` / `BuffEffectDTO` / `TimedBuffDefDTO` /
  `SkillModifierDTO` / `MijiDefDTO` / `QixueDefDTO` / `CombatMetaDTO` / `CombatDataDTO` / `CombatBundleDTO` /
  `TimelineActionDTO` / `TimelineRequestDTO` / `TimelineEventDTO` / `TimelineResultDTO`。
- **host 层**：`crates/jpcg_core/src/host/combat.rs`（core↔DTO 双向转换、`load_combat_data` / `load_combat_with_skills`）；
  `crates/jpcg_combo/src/timeline_host.rs`（`load_combat` / `load_combat_bundle` / `calculate_timeline`）。
- **FFI**（`jpcg_combo` dispatch）：新增 `load_combat` / `load_combat_with_skills` / `calculate_timeline`，动态模式 dll 可调。
- **Tauri 命令**：`load_combat_bundle`、`calculate_timeline_cmd`（static 直调 / dynamic 经 `ffi_bridge::call_combo`）。
- **前端预览**：排轴器新增「时间轴预览」按钮：读取门派 combat overlay → 组装 `TimelineRequestDTO` → 展示窗口伤害 / DPS / 未执行动作 / 事件流（施放、dot 跳、形态切换、buff）。

## 为什么

#92 要求「技能 CD / 秘籍 / 奇穴 / 加速 / 形态 / dot 快照 / 窗口 DPS」这些内容**不进编译期**，改由数据文件驱动，
否则每次补一个门派的时序数据都要发版。数据层 + DTO + FFI + Tauri 一条链路打通后，前端可预览、独立 dll 可分发。

## 关键决策

- overlay 采用「可选覆盖」：`skill_combat` 只写与默认不同的字段，其余回退技能表；`apply_overlay` 返回新技能列表，不改原数据。
- 数据文件解析失败 / 校验失败**不 panic**：`load_combat` 记日志并返回空数据（前端退化为「仅 GCD」预览），
  `load_skills_with_combat` 返回带文件路径的错误，方便定位手填错误。
- `TimelineRequestDTO` 的配置字段名沿用 combo 的 `coeff`（不是 `coefficient`），`buff` / `coeff` / `player` / `hostile` / `xinfa` 必填；
  `combat` 可选（为空时仅用技能池自身时序字段）。
- 旧 `engine.rs` 按步累加路径完全不动，`into_core` 只透传新增字段，旧连招结果与金标准不变。

## 验证

- `cargo test -p jpcg_core`：57 passed（新增 combat 解析/校验/loader 用例）
- `cargo test -p jpcg_combo`：25 passed（timeline 9 + timeline_host 2 + FFI 2）
- `cargo build -p jpcg_app`：通过（static 模式）
- 前端：`npm test` 44 passed；`npx tsc --noEmit` 无错误；`npm run build` 通过

## 已知限制（后续）

- `data/combat/*.toml` 尚无真实门派数据（示例/填表文档见 #92 C 阶段）。
- 加速等级暂由请求显式传入（`PlayerConfigDTO.jiasu_dengji`），前端暂无输入控件。
- CD 是否帧量化、dot 逐跳量化待游戏实测确认（当前：CD 连续缩放、dot 总时长量化后均匀分布）。

### 来自 changes/2026-10-30-202350.md

# 时序引擎 (C)：data/combat 填表文档 + 模板

## 做什么

- 新增 `data/combat/README.md`：`data/shuxing`（生成物）与 `data/combat`（手工 overlay）的分工、
  加载顺序（`JPCG_DATA_DIR` → 可执行文件候选根目录）、全部字段表
  （`[meta]` / `[[form]]` / `[[skill_combat]]` + `on_cast` / `[[buff]]` / `[[miji]]` / `[[qixue]]` / `modifiers`）、
  合并与校验规则、加速合并公式、预览与排错方式、已知待定项。
- 新增 `data/combat/_template.toml`：可解析、可通过 `validate()` 的模板（以 `_` 开头不会被加载），
  含莫问真实技能 id、双形态 / CD / 充能 / on_cast / 定时 buff / 秘籍 / 奇穴的注释示例。
- `crates/jpcg_core/src/type_set/combat.rs` 新增测试 `template_file_parses`：
  `include_str!` 模板 → 解析 + `validate()`，防止文档与代码漂移。

## 为什么

#92 的时序数据要「常改的东西放 toml、不重新编译」，维护者需要一份能照抄的字段说明；
模板做成可解析可校验，能保证文档里写的字段名与 `deny_unknown_fields` 的现实一致。

## 验证

- `cargo test -p jpcg_core`：58 passed（新增模板解析用例）
- 模板校验：`[[form]]` 初始形态唯一、`skill_combat` 引用真实技能 id、`validate()` 通过

### 来自 changes/2026-10-30-204500.md

# 2026-10-30-204500 时序引擎复审修复（#97）

## 做了什么

修复 #97（#92 切片 1）复审中的 7 条行内问题：

1. **DOT 双重计入**：`TimelineEventKind::Cast.damage` 对 DOT 技能记 0（DOT 的 `q_cal` 已返回全部跳数之和），
   窗口伤害只按实际 `DotTick` 事件计入；聚合时 `Cast` 也改为只有 `hit_time <= window` 才累加。
2. **同名 id 变体消歧**：`TimelineAction::Cast` 增加可选 `skill_name`；
   `(skill_id, sub_id)` 命中多个变体且未给名称时该动作直接跳过（计入 `skipped_actions`），不再静默取首项。
   `ComboPage` 构造动作时带上选中技能名，DTO 字段 `skill_name` 可选以兼容旧调用。
3. **充能恢复**：状态改为 `ChargeState{left,max,interval,next}`，每次施放/判断前按当前时刻结算已恢复层数并推进
   `next`（满层置 +∞）；2 层/8 秒的连续 5 次施放为 `0 / 1.5 / 8 / 16 / 24`。
4. **技能自身 GCD**：新增 `skill_gcd()`，技能 `gcd_base` 覆盖形态/心法基础，`gcd_kind=2`（固定 GCD）不受加速影响；
   额外通道（`on_cast.gcd_lanes`）仍按通道默认值。
5. **窗口命中过滤**：读条/引导技能只在 `hit_time` 落入窗口时才计入伤害，保留起手事件。
6. **常驻 buff**：`TimedBuffDef.duration = 0` 视为永久有效（`expires = +∞`），重施加/刷新逻辑一致处理。
7. **前端事件形状**：`TimelineEventKindDTO` 是 `#[serde(tag = "kind", rename_all = "snake_case")]` 的内部标签形状，
   TS 类型、mock、渲染统一为 `{ kind: "cast", ... }`；新增 `formatTimelineEvent()` 与基于真实 Rust 响应形状的渲染测试。

8. **`refresh=false` 的 buff 独立计时**：改为每个实例单独保存到期时间；`refresh=true` 才叠加层数并刷新持续时间，
   `refresh=false` 受 `max_stacks` 上限约束但各自到期（duration=3、两层在 t=0/1.5 施加 → t=3 只剩第二层）。
9. **`gcd_group` 参与 GCD 通道**：没有显式 `combat.gcd_lane` 时从技能 `gcd_group` 派生稳定通道
   （`0` → `default`，`1` → `group_1`），两组互不阻塞；`data/combat/README.md` 补充说明。

## 关键决定

- 状态表（CD/充能）改以「技能下标」为键，避免同名 id 的多个变体（如莫问「入月·N 层」）共享 GCD/CD/充能状态。
- 歧义动作语义选择「跳过 + 计数」而非猜测：宁可让玩家看见 `未执行动作`，也不静默算错层数。
- `Cast` 事件对 DOT 技能显示 0 直接伤害，伤害体现在逐跳事件上；不影响总伤口径。

## 验证

- `cargo test -p jpcg_core`：58 passed
- `cargo test -p jpcg_combo`：34 passed（timeline 新增 9 个回归用例：DOT 不重复、短窗口、歧义键、充能恢复、技能 GCD、跨窗口读条、常驻 buff、refresh=false 独立实例、gcd_group 通道）
- `cargo build -p jpcg_app`：通过
- 前端：`npx tsc --noEmit` 无错、`npm test` 45 passed（新增 `timelineFormat.test.ts`）

### 来自 changes/2026-10-31-051000.md

# 装备原始属性到玩家面板的可验证换算（#98）

- 新增显式选择的历史130级莫问换算规则，独立整数实现体质→气血、根骨→会心/心法额外攻击、元气→基础攻击/破防；固定来源提交与取整阶段，未知心法/版本及溢出报错。
- 扩展原始五维属性，保留旧主属性增量别名与旧存档语义。原始合计与派生面板分离；启用规则时配装约束使用派生点数，Pareto使用派生气血。
- 玩家DTO新增可选心法额外攻击标记；单技能、连招、时间轴、配置往返共享映射，避免重复转换并修复加速丢失。旧手填金标准不变；已转换输入的旧求导明确报错。玩家破防中间乘法扩大到u64防止溢出。
- 驾驶台提供规则选择、体质及非主属性输入、派生面板与来源限制说明；修复前端请求归一化丢弃加速和转换标记，保留手动状态与旧配装。
- 增加手算样例、边界、宿主/优化/时间轴/动态桥接与前端回归，记录上游差分验证。
- 当前只声明固定上游来源的规则一致性，尚无真实游戏面板基准；synthetic-v1仍为合成样例。其他心法、真实装备、强化/镶嵌/附魔、完整战斗增益仍待确认，#98不关闭。详见 docs/attribute-engine.md。

### 来自 changes/2026-10-31-052000.md

# 扩大可验证规则并防止新旧动态模块静默混算（#98）

- 固定同一130级来源快照，增加傲血10026、分山10390、花间10021，覆盖四类主属性；按实际外功/阴性/混元通道转换，主属性别名在通用及心法转换前相加。不新增虚构的真实装备数据。
- 区分基础气血与额外气血上限，后者在心法基础气血增益及截断后相加；新增字段默认0兼容旧配装，扩展阶段边界、TOML和宿主调用测试。
- core/combo新增只读属性能力查询，动态桥接对转换请求及配置导入/加载/导出执行能力门禁，拒绝旧模块静默丢失标记；旧请求不要求新能力。
- 直接Rust偏导明确固定面板语义，避免主属性正向不变、偏导却非零；host仍要求重新合成配装。
- 记录许可证核查及独立来源差异，继续区分上游差分与真实游戏面板验证；后者仍无完整基准。
- 将GLib/GTK/WebKit开发依赖解包到工作区，完成 make check-all：149项workspace测试、4项桌面动态测试、83项前端测试及双模式构建通过；四心法共40,000组上游差分零差异。具体边界见 docs/attribute-engine.md，本轮不执行任何远端发布。

### 来自 changes/2026-10-31-061200.md

# 修复迟到配装导入覆盖较新操作

## 问题

配装 TOML 导入在文件读取与宿主解析结束后才建立操作版本。期间用户选择其他装备、切换手动模式、恢复基准或再导入另一个文件，旧响应仍可能覆盖较新的状态并持久化，旧失败也会弹出误导性提示。

## 修改

- 在选择文件时由 App 建立共享 action revision，贯穿文件读取、宿主导入、值集和心法解析；不在后续阶段重新争取操作优先级。
- 只允许当前操作应用配装或显示错误，后续编辑、切模式、恢复基准和新导入继续复用现有失效机制。
- 文件输入立即清空，允许再次选择同一文件；读取异常统一进入当前导入的错误提示。
- 不改变 Rust 计算、数据、DTO、依赖、存储格式或组件版本。

## 验证

- 初始 6 项新增回归在原 dev 上 5 项失败、1 项通过，既有驾驶台 6 项通过；修复后新增值集等待、基准恢复和读取错误边界，共新增 9 项。
- 前端完整 76/76 通过；TypeScript + Vite 构建、Makefile 顺序守卫和 git diff --check 通过。
- make check-all 已执行，但本机没有 cargo，停在 fmt-check；Rust、金标准和桌面双模式等待 PR CI，不记为本机通过。
- 前端回归使用 jsdom 与宿主 API mock，不代表 Tauri 原生文件选择/IPC 已实测。

## PR 复审补修

- 独立复审发现首提交会把旧值集/心法的异步回填当成新操作，从而取消后选导入。保留问题记录，不把首提交 CI 全绿作为行为正确的证明。
- ConfigPanel 明确区分用户编辑/选择与元数据回填；只有前者使旧导入失效，元数据仍遵循自身已有请求版本检查。
- 补充 3 项回归，在首提交上全部失败，修复后通过；另补“导入后的真实值集选择仍取消旧导入”反向控制。最终前端 83/83、TypeScript + Vite、Makefile 顺序守卫与 diff 检查通过。未新增本机 Rust/原生桌面验证。
- 全新启动的自动值集匹配固定使用启动版本，不创建或借用较新用户操作的版本；新增此前失败的启动交错回归，以及正常自动匹配、自动响应晚于导入的反序控制。最终相对 dev 新增 16 项前端回归。

### 来自 changes/2026-10-31-070000.md

# 改为逐职业TOML驱动属性引擎（#98）

- 根据Qin确认的当前50级及各职业快照不同要求，移除生产代码中的四心法130级常数；从职业TOML的attributes读取全部转换系数，旧历史ID不自动迁移。
- 新共享契约显式表达等级、来源、数值集、主属性、伤害通道、逐项有理数转换及气血阶段。无缺省游戏常数，缺表/混级/非法分母/溢出报错。
- 职业技能编辑器与Rust保存路径保留换算表，防止旧编辑请求擦除；优化在一次搜索中固定规则快照。
- 前端统一提供职业TOML模式，支持任意职业；禁用旧历史规则的缓存结果，职业TOML模式重新打开／显式重读时须验证新结果，失败前不能沿用旧面板；保留旧手填口径。
- 提供非游戏数值的50级录入示例与端到端测试，历史差分仅作为回归。现有130级技能和synthetic-v1装备没有伪装迁移成真实50级数据。
- 发布已暂停：之前仅创建首个旧提交对应的tree对象，未创建远端提交/分支/PR。本轮变更须重新获得发布授权。

验证：make check-all通过；151项workspace、4项桌面动态、91项前端及9项既有golden通过。实际动态库50级合成管线和4,000组历史模型差分通过，仍无真实50级游戏面板基准。

### 来自 changes/2026-10-31-131500.md

# 配装驾驶台并入 dev：去掉叠放分支的 CI 触发

## 做什么

- 把 #96 从叠放草稿 `integration/91-gear-base` 基底改为基于最新 `dev`（已含 #97 时序引擎、#94 配装基础、#95 持久化驾驶台）。
- `.github/workflows/ci.yml` 移除叠放期临时触发：`pull_request.branches` 去掉 `integration/91-gear-base`，`push.branches` 去掉 `feature/91-gear-cockpit`，恢复为 `[dev, beta, release]` / `[dev]`。
- 合并冲突处理：`App.tsx`/`ConfigPanel.tsx` 保留 #96 的配装驾驶台与只读合成属性，并叠加 #95 复审的 `manualReady` 初始化时序修复（`onFormChange(update, { ready })`）；`ComboPage.tsx` 保留 #97 的时间轴预览并叠加 #96 的受控草稿 / 配置页只编辑不计算；`crates/jpcg_gear/src/optimize.rs` 补 `PlayerConfigDTO.jiasu_dengji`；配装目录/合成/导入导出相关前端与 Tauri 命令照旧。

## 为什么

#96 是为叠放审阅临时以 `integration/91-gear-base` 为目标分支的草稿；#94/#95/#97 已合入 dev 后，需要回到常规流程（base=dev、CI 只在 dev/beta/release 触发），避免临时分支长期留在工作流配置里。

## 验证

- 前端：`tsc --noEmit` 无错、`npm test` 67 passed（含 GearCockpit 6 项）。
- Rust：`cargo test -p jpcg_core -p jpcg_combo -p jpcg_gear` 全绿、`cargo fmt --all -- --check` 通过、`cargo build -p jpcg_app` 通过。

### 来自 changes/2026-11-03-061230.md

# fix(app): 分开持久化心法与系数初始化状态

- 原 `manualReady` 同时代表心法和系数，导致 A 心法就绪后切 B 失败/挂起，重启仍保留 B 标识与 A 属性；首次心法成功但值集失败时，重启会把占位系数当作自定义。反方向还会在重试心法时丢掉已经编辑的系数。
- 增加可选的 `manualHydration` 两项就绪标记，每次选择只使对应项失效；成功后分别写回。重启只补未就绪部分，保留已成功的心法属性及系数。保留 `manualReady` 与旧 v1/v2 快照兼容，不升级存储版本；历史已误标完成的旧快照无法可靠区分真实自定义，不猜测迁移。
- 存档加载、清空、基准恢复与配装导入同步标记；配装导入等待期间读取最新就绪状态，通过同步就绪 ref 覆盖同批微任务完成的交错，避免旧闭包覆盖初始化回填。心法未就绪时阻止计算/保存，当前加载错误给出重试提示。
- 22 项 App 级确定性回归在独立原 dev 副本上 18 失败 / 4 通过，修复后 22/22；另加 2 项旧快照兼容/损坏字段校验。完整前端 131/131、TypeScript/Vite 构建、Makefile 顺序守卫及 diff 检查通过。
- dot 自有云 Node 24.22.1、锁文件 `npm ci`。`make check-all CARGO_JOBS=2` 实跑在首个 fmt-check 因缺 cargo 停止（cargo 127 / make 2）；本机未跑 Rust/桌面/金标准，交由本 PR CI 核验。组件测试使用 jsdom 与 API mock，不是原生 Tauri/IPC/磁盘端到端实测。
- 仅前端状态与测试改动；无 Rust、正式游戏数据、数值公式、依赖、版本、工作流变更。保留既有 Vite 大 chunk 提示。

### 来自 changes/2026-11-05-011500.md

# 莫问战斗时序 overlay（data/combat/mowen.toml）

## 做什么

- 新增 `data/combat/mowen.toml`：把游戏内 tooltip 截图整理出的时序数据落成 #92 overlay。
  - 4 个形态：`qu-yangchun` / `qu-gaoshan` / `qu-pingsha` / `jian`，各带 `gcd_base = 1.5` 与可用技能列表。
  - 16 条 `[[skill_combat]]`：音 5（宫/商/徵/羽/角）+ 变体 2（变宫/变徵）+ 剑 5 + 奇穴主动 4（冲秋冥/不愧君/断水流/斩长鲸）。
  - 读条、调息、充能、是否吃加速、蓄力通道等字段按 tooltip 填写。
- `data/skill-shots/out/mowen-final.json`：26 条技能的原始整理结果（含伤害区间、攻击加成、倍率、四形态、35 个奇穴）。
- `server_tools/skill-shots`：小图（宽 <900px）先放大到 900px 再 OCR，提升截图识别率。

## 为什么

- 现有 `data/shuxing/mowen.toml` 只有 130 级伤害数据，没有读条/调息/充能，#92 时序引擎需要 overlay 才能排轴。
- 游戏 Lua 已把公式搬进引擎侧，tooltip 是当前唯一能直接读到「基础伤害 + 攻击加成」的来源。

## 待补

- 曲/气/影 的主动技能（高山流水曲、阳春白雪曲、平沙落雁曲、青霄飞羽、云生结海、江逐月天、迴梦逐光、笑傲光阴、疏影横斜、清绝影歌、孤影化双）在 `data/shuxing/mowen.toml` 里没有对应 skill_id，先留空。
- 清绝影歌 / 清绝影歌·回 的 `on_cast.switch_form` 待 id 补齐后接入。
- `宫 +2723`、`阳春白雪曲 +1145/+1308/+57780` 为带 buff 截图，已按 0.99694 系数校正，待替换为无 buff 截图。

### 来自 changes/2026-11-05-012500.md

# 莫问时序补全：曲/气/影 id、形态切换与默认 forms

## 做什么

- `data/shuxing/mowen.toml` 手工补充 17 个技能条目（伤害为 0，仅供时序引用）：
  高山流水 14069、平沙落雁 14072、三曲主动 14229/14230/14232、笑傲光阴 14073、江逐月天 14074、
  云生结海 14075、青霄飞羽 14076、迴梦逐光 14154、孤影化双 14081、清绝影歌 14083、清绝影歌·回 14522、
  剑·徵 14451、斩长鲸 36489、九衢闻曲 45730、太常律 30564。
  （id 以游戏内「调试用信息」显示的 ID 为准；此前 shuxing 里 40314=剑·徵群攻、34344=五音正律，均非单目标技能。）
- `data/combat/mowen.toml`：
  - 三曲形态的 skills 列表补齐三曲主动 + 气/影 + 清绝影歌；剑形态补清绝影歌·回。
  - 新增 `[[skill_combat]]`：三曲主动的 `on_cast.switch_form`、青霄飞羽 1s/40s、云生结海/江逐月天/迴梦逐光/笑傲光阴 70s、疏影横斜 28s 充能、孤影化双 150s、清绝影歌 60s → jian、清绝影歌·回 → qu-yangchun、太常律。
  - 修正 剑·徵 14451、斩长鲸 36489 的 id。
- `crates/jpcg_combo/src/timeline_host.rs`：请求未显式给 `forms` 时，从 `combat` overlay 的 `form` 列表兜底（此前空 forms 会导致形态切换事件不触发）。

## 为什么

- 截图里的「调试用信息 ID」就是技能真实 id，比按名字从 shuxing 猜准确；之前 剑·徵/斩长鲸 的 id 是错的。
- 前端只传 combat overlay 时也应能跑形态切换，避免调用方必须重复传 forms。

## 验证

- 临时测试（已删）：`load_combat_with_skills("mowen")` 通过校验（4 forms / 24 skill_combat，form 引用技能全部在池中）。
- 临时测试（已删）：`calculate_timeline` 跑「高山流水_主动 → 变宫 → 商 → 角 → 变徵 → 羽」，窗口 12s：
  `FormSwitch qu-yangchun → qu-gaoshan`，6 次 Cast 全部命中，0 skipped，伤害 14109/13113/13113/9503/14056。
- `cargo test -p jpcg_core -p jpcg_combo` 全绿（72 + 37）。

### 来自 changes/2026-11-05-013000.md

# 奇穴第 1 层（修饰类）落地 + 倍率基准改回 160

## 做什么

- `SkillModifier` 新增两个变体：`CooldownAdd(f32)`（调息 +N 秒，用于基础调息为 0 的奇穴）与 `DotTicksAdd(i32)`（Dot 跳数 +N，按 interval 折算到持续时间）；同步 `SkillModifierDTO` 与双向转换。
- `QixueDef` 新增 `applies_to: Vec<(u32, u32)>`（一个奇穴作用于多个技能），`targets()` 优先按该列表匹配；`QixueDefDTO` 同步；`validate()` 改为「replaces_skill_id 为 0 时必须有 applies_to」，`validate_against_skills()` 逐个校验 applies_to 引用。
- `data/combat/mowen.toml` 新增 7 条 `[[qixue]]`（第 1 层）：
  殊曲（商/角 间隔 -33.3%、增伤 +12）、削竹（剑·徵 增伤 +16）、参连（无视 60% 内功防御 = 614/1024，覆盖音/剑全部伤害技能）、
  冻泉（商/角 +4 跳）、冻泉·宫（宫/变宫 调息 +12s）、凌冬（徵/变徵 调息 +18s）、兼山（云生结海 调息 +20s）。
- `TimelineRequestDTO` 新增 `qixue: Vec<String>`；`timeline_host::calculate_timeline` 先按 id 取奇穴定义、对每个技能 `resolve_skill` 套 modifiers，再交给时序引擎。
- `server_tools/skill-shots` 倍率基准默认值 `--base` 由 192 改为 160（用户确认当前版本内外功统一为 160）；`data/skill-shots/out/mowen-final.json` 的 `ratio` 改为 `攻击加成 × 160 / 8438`，并保留 `ratio_raw`（= 攻击加成 / 8438）。

## 为什么

- 奇穴有大量「调息 +N 秒」「Dot +N 跳」这类非比例修正，原 `CooldownMul`/`DotDurationMul` 表达不了。
- 一个奇穴常同时作用于多个技能（殊曲→商+角、参连→音/剑全部），单一 `replaces_skill_id` 不够用。

## 验证

- 临时测试（已删）：加载 mowen overlay 后取「殊曲」对 商 执行 `resolve_skill`，`dot_interval` 3.0→2.0、`hit_up` +12 通过。
- `cargo test -p jpcg_core -p jpcg_combo` 全绿（72 + 37）；`cargo fmt --all -- --check` 通过。

## 待做

- 第 2 层：替换/新增技能（洞天、鹿鸣、鸾羽、超然、不愧君/断水流/斩长鲸/冲秋冥/太常律）→ replacement + `jihuoqixue` 门控。
- 第 3 层：触发/资源类（音彻、净植、冻泉的「下一跳立即生效」、师襄曲风会心、忘弦/捣衣等）→ 需要引擎 on_cast/on_hit/on_dot_tick/on_qifeng 钩子。
- 秘籍选择（4/4 页）需要「选中的秘籍集合」入口，目前 `[[miji]]` 结构可复用但无激活列表。

## 补充（倍率口径）

- 用户确认：倍率必须存整数 —— `倍率 = int(攻击加成 × 160 / 最终攻击力)`，结算时 `攻击加成 = 倍率 × 攻击力 / 160`（游戏内显示未取整，实际计算走整数）。`mowen-final.json` 的 `ratio/ratio_int` 已按此口径重算（宫 51、商 48、徵 35、羽 51、角 48、剑·宫 73、剑·商 160、剑·徵 62、剑·羽 101、剑·角 68）。

### 来自 changes/2026-11-05-015000.md

# 技能倍率接入：截图整数倍率写回 data/shuxing

- 新增 `data/skill-shots/mowen-final.json`（从 `out/` 提升为受版本管理），给 10 个伤害技能补 `skill_id`/`sub_id`。
- 新增 `scripts/apply-skill-coeff.py`：按 `(skill_id, sub_id)` 把 `ratio_int / 160` 写入 `data/shuxing/mowen.toml` 的 `atk_xishu`，`base_min/base_max` 写入 `base_damage1/2`；缺失字段自动插入；额外处理两条 DOT（商/角 每跳，整数倍率 14）。
- 数据修正（以 `skills.tab` 的名称为准）：音行 id 对应关系是 宫 14064/14474、商 14065/14311、角 14066/14312、徵 14067/14227、羽 14068/14100（此前把徵/羽/角的 id 记错了，已修正 json 与 TOML）。
- 新增回归测试 `crates/jpcg_core/tests/skill_coeff_data.rs`：断言 TOML 的 `atk_xishu` 与 json 的 `ratio_int/160` 一致、基础伤害一致；json-to-toml 重新生成数据若覆盖了系数，CI 会失败。
- 结果：宫 0.31875(80-90)、商 0.3(95-100)、角 0.3(95-100)、徵 0.21875(84-92)、羽 0.31875(156-175)、剑·宫 0.45625、剑·商 1.0、剑·徵 0.3875、剑·羽 0.63125、剑·角 0.425（三段蓄力暂用同一档）。
- 已知遗留：剑·角 3 段蓄力伤害（0/16/32 帧）工具只有首段与满蓄力两档数据，暂统一为 260-270；DOT 每跳只按整数倍率 14 估算。

### 来自 changes/2026-11-05-020000.md

# 战斗内曲风初始满点

- 只建模战斗场景：非战斗每秒 +2 曲风（九衢闻曲）不实现，`Sim.qifeng` 初始化为 `MAX_QIFENG = 9`。
- `crates/jpcg_combo/src/timeline.rs` 新增 `pub const MAX_QIFENG: u8 = 9`，获得/消耗后的上限判定统一引用它。
- 回归测试同步：净植按「宫 一次消耗 9 点」断言 9×1330；师襄按 9 点 ≈ +18%。

### 来自 changes/2026-11-05-021000.md

# 补齐变宫/变徵倍率

- `data/shuxing/mowen.toml` 里 变宫(14298/18860)、变徵(14299/18859) 仍是 130 级老系数（2.609375/1.7760416）；
  按「变体伤害与 宫/徵 同源」补为 0.31875 / 0.21875，基础伤害 80-90 / 84-92。
- `scripts/apply-skill-coeff.py` 增加这两条显式目标（json 里它们是 `gaoshan_variant`，不是独立技能条目）。

### 来自 changes/2026-11-05-021500.md

# 排轴器：秘籍 / 奇穴勾选接入时间轴

- `examples/jpcg_app/src/types/index.ts`：`CombatDataDTO.miji/qixue` 改为 `MijiDefDTO[]`/`QixueDefDTO[]`；`TimelineRequestDTO` 增 `miji?: string[]`、`qixue?: string[]`。
- `examples/jpcg_app/src/components/ComboPage.tsx`：面板属性同步后加载 `loadCombatBundle`，把门派 combat overlay 的秘籍与奇穴渲染成勾选项；「时间轴预览」请求带上勾选 id（`miji`/`qixue`）。
- `ComboPage.module.css`：新增 `.optionsPanel/.optionsRow/.optionsLabel/.optionChip`。
- 测试：`ComboPage.test.tsx` 新增「时间轴预览下发勾选的秘籍与奇穴」；前端 tsc / vitest（108 项）/ vite build 全过。

### 来自 changes/2026-11-05-023000.md

# 正式服 50 级数值集 live-50 + 配装模板值集切换

- 新增 `data/values/live-50.toml`（level 50）：先沿用体验服一测的 50 级观测值，待正式服快照校准。
- `data/values/index.toml`：注册 `live-50`（正式服·50级），默认值集切到 `live-50`。
- `data/gear/*`：`templates/schools/sets/overrides/sample-loadout` 的 `value_set` 由 `live-130` 改为 `live-50`，让配装模板与 50 级数值集匹配。
- 测试：`crates/jpcg_core/src/store/values.rs` 断言改为 3 个值集、`live-50` 为默认；`crates/jpcg_gear/tests/attributes.rs` 夹具拷贝 `live-50.toml` 并把夹具内值集统一替换为测试值集 `cszj-exp-260908`。
- 仍未做：`data/shuxing/mowen.toml` 缺少 `[attributes]` 50 级换算规则（需要真实面板数据），因此配装合成在真实数据下仍会被拦。

## 补充（50 级属性换算）

- `crates/jpcg_api/src/attributes.rs` 新增 `AttributeConstant { to, value }`，`AttributeRules.constants`（`serde(default)`）用于心法固定点数加成。
- `crates/jpcg_gear/src/attributes.rs` 的 `convert()` 先加 constants 再按 conversions 换算。
- `data/shuxing/mowen.toml`：`[version].level` 改 50；新增 `[attributes]`（id `jx3-50-mowen-20261030`、value_set `live-50`、main_stat gengu、damage_kind lunar、qixue_multiplier 1/1）：
  - 根骨 → 基础攻击 91/100（0.19 主属性 + 0.72 心法）
  - 根骨 → 会心等级 28/100（0.25 + 0.03）
  - 根骨 → 破防 6/100
  - 体质 → 气血基础 0（显式零，气血不参与伤害计算）
  - 常量：内功攻击 +2282、内功会心等级 +308
  - 依据：jx3box bps/109685（数值体系）、bps/109665（莫问心法加成）

## 补充（[xinfa] 老式收益清理）

- `data/shuxing/mowen.toml` 的 `[xinfa]` 由 130 级口径改为 50 级新式口径：
  - `atk_up` 1.96 → 0.91（根骨自身 0.19 + 莫问心法 0.72）
  - `pofang_up` 2.0 → 0.06（根骨自身 0.06）
  - `huixin_up` 0 → 0.28（根骨 0.25 + 心法 0.03）
- 心法固定加成（内功攻击 +2282、内功会心等级 +308）仍由 `[attributes].constants` 提供，配装路径经 `zhuanhua_gongji` 传入、不走 `atk_up` 二次换算。

## 补充（根骨不提供破防）

- `data/shuxing/mowen.toml`：`[xinfa].pofang_up` 0.06 → 0；`[attributes]` 删除 `gengu → pofang` 换算（50 级根骨不提供破防，破防来自装备直给的破防属性）。

## 补充（根骨自身不给攻击）

- 更正：`bps/109685` 的「主属性 → 基础攻击 0.190」只适用于以攻击为主属性的门派（力道/元气），根骨主属性不提供基础攻击。
- `[xinfa].atk_up` 0.91 → 0.72（仅莫问心法每点根骨 +0.72 内功攻击）；`[attributes]` 的 `gengu → gongji_base` 分子 91 → 72。
- `[xinfa].huixin_up` 保持 0.28（根骨主属性会心 0.25 + 心法 0.03），`pofang_up = 0`。

## 补充（心法根骨收益归入额外攻击）

- 攻击力模型（用户口径）：`基础攻击力 = 装备攻击词条 + 基础属性自身换算`；`最终攻击力 = 基础攻击力 + 心法额外加成`。
- 增益有两类：「基础攻击力提高 X%」只放大基础部分；「攻击力提高 X%」作用于最终攻击力（引擎目前只实现前者 `BuffConfig.base_atk_pct`）。
- 因此根骨 0.72 属心法额外，`[attributes]` 换算目标由 `gongji_base` 改为 `gongji_extra`；与 `constants.gongji_extra = 2282` 合并进入 `zhuanhua_gongji`，不被「基础攻击力提高」放大。

## 补充（旧存档版本迁移修复）

- 症状：更新模板/数值集后，配装页同时卡在「需重新合成当前职业规则」与「配装需要匹配模板版本与数值集」，点两个按钮都无法消除。
- 根因：`App.adoptValueSet` 用存档里的 `loadout.value_set`（旧 live-130）去解析数值集，又把旧值写回 loadout/form，与模板 `catalog.version`（live-50）永远不等；`compatible=false` → 不触发合成 → `rulesCurrent=false`，两条告警互锁。
- 修复：`adoptValueSet` 统一迁移到模板版本口径（`revision`/`value_set` 取 `catalog.version`），按钮文案改显示模板目标值集；冒烟：前端 108 测试、tsc 通过。

### 来自 changes/2026-11-05-032000.md

# 奇穴第 3 层：曲风资源与运行时钩子

## 做什么

- 数据模型新增 `QixueHook`（`FlatDamageOnCast { damage, per_qifeng_spent }`、`DamagePctPerQifeng { pct }`），`QixueDef.hooks`/`QixueDefDTO.hooks` 双向转换；校验允许「只有钩子、不替换也不修饰」的奇穴。
- `SkillCombatDef` 增加 `qifeng_gain` / `spend_all_qifeng`；时序条目携带这两个字段。
- 时序引擎新增曲风资源：`Sim.qifeng`（0~9）。施放时先按施放前曲风结算钩子，再结清消耗、累加获得。
- `TimelineInput.hooks` + `ActiveHook { targets, hook }`；`calculate_timeline` 把激活奇穴的 hooks（含 `applies_to` / `replaces_*` 作为作用范围，空 = 全部技能）传进引擎。

## 数据

`data/combat/mowen.toml`：

- 宫（14064/14474）、变宫（14298/18860）：`spend_all_qifeng = true`。
- 商、角、徵、羽（14065/14066/14067/14068）：`qifeng_gain = 1`。
- 新增奇穴：音彻（剑·音 命中追加 2159）、净植（每消耗 1 点曲风追加 1330）、师襄（每点未消耗曲风 +2% 伤害）。

## 验证

`crates/jpcg_combo/tests/qixue_hooks.rs` 三个回归：

- 净植：商（+1 曲风）→ 宫，宫伤害正好 +1330。
- 师襄：连放两次商，第二次伤害约为 1.02 倍。
- 音彻：剑·徵 直接伤害 +2159。

`cargo test -p jpcg_core` 72、`cargo test -p jpcg_combo` 37 + 3 全绿，`cargo fmt --all` 通过。

## 近似与未做

- 曲风上限固定 9；尚未支持「非战斗每秒 +2」「九衢闻曲 羽 +1」等被动获取。
- 音彻的理论触发条件（目标处于自身音域内）暂按「始终触发」近似，音域状态尚未进模型。
- 净植的治疗吸收盾、师襄的无双/会心效果精确换算未做，按等效伤害近似。
- 替换型奇穴（洞天/鹿鸣/鸾羽/超然）与秘籍选择入口仍未做。

### 来自 changes/2026-11-05-034500.md

# 秘籍选择入口

## 做什么

- `TimelineRequestDTO` 增加 `miji: Vec<String>`；`calculate_timeline` 按 id 取 `[[miji]]` 定义，先用 `resolve_skill` 套到技能上再进引擎。
- 新增 `SkillModifier::CastTimeAdd(f32)`（定值减读条，秘籍「运功时间减少 0.125 秒」这类），DTO 同步。
- 调整修正项顺序：先在 `TimingSkill::from_combat` 里套 overlay 时序字段（cast_time/cooldown 等），再套秘籍/奇穴修正项 —— 否则 overlay 的定值会覆盖秘籍的读条/调息修正。

## 数据

`data/combat/mowen.toml` 新增 26 条 `[[miji]]`，覆盖 6 个有数值效果的技能：宫、商、徵、羽、青霄飞羽、笑傲光阴。

- 宫：读条 -0.125×2、伤害 +3%/+4%、会心 +2/3/4%。
- 商、徵、羽：伤害 +3~5%、会心 +2~4%（按截图逐条）。
- 青霄飞羽：调息 -3 秒。
- 笑傲光阴：调息 -3/-3/-4 秒。
- 角（仅移速）、杯水留影（仅复活减伤）没有可建模的数值，未列入。
- 距离 +2/+3 尺、移速、禁轻功、音域互斥解除等非伤害效果暂不建模。

## 验证

`crates/jpcg_combo/tests/qixue_hooks.rs` 新增 2 个：

- 宫 +4% 秘籍：伤害约为 1.04 倍。
- 两条减读条秘籍：宫 之后的下一个技能提前 >0.2 秒。

`cargo test -p jpcg_core` 72、`cargo test -p jpcg_combo` 37 + 5 全绿，`cargo fmt --all` 通过。

### 来自 changes/2026-11-05-040500.md

# 奇穴替换型：洞天、超然

## 做什么

- 数据层补 `[[qixue]].replacement`（替换为独立技能条目）的实战用例：
  - **洞天**：替换「变徵」（14299/18859）为 1.5 秒 3 跳、每跳 4851 点固定伤害（`atk_xishu = 0`，dot）。激活后引擎解析出的技能名与伤害都取替换条目。
  - **超然**：不替换，用 `cooldown_mul = -60` 把孤影化双 150 秒调息压到 60 秒（替换型 + 修饰型混用的写法示范）。
- `resolve_skill` 现有替换逻辑已覆盖「replacement + modifiers 同时存在」；本次只补数据与回归。

## 验证

`crates/jpcg_combo/tests/qixue_hooks.rs::replacement_qixue_changes_skill`：先切到 `qu-gaoshan` 形态，未激活时是「变徵」原始伤害，激活 `dongtian` 后技能名变「洞天」且 dot 三段合计过万。

`cargo test -p jpcg_core` 72、`cargo test -p jpcg_combo` 37 + 6 全绿，`cargo fmt --all` 通过。

## 未做 / 待补

- 鸾羽（青霄飞羽 → 飞鸾动羽，缺「飞鸾动羽」技能 id 与伤害值）、鹿鸣（宫 → 6 秒调息 + 每 0.5 秒消耗曲风的持续运功，缺跳数与结束条件）暂未落。
- 洞天「消耗 3 点曲风」、超然「受加速率影响 + 琴影 +30% 加速突破」暂未建模。

### 来自 changes/2026-11-05-041500.md

# 奇穴：鸾羽替换 + 鹿鸣占位

- **鸾羽**：`[[qixue]] id = "luanyu"` 把青霄飞羽（14076）替换为「飞鸾动羽」：999 点固定伤害、瞬发、40 秒调息（tooltip 原文就是 999 点，已与用户确认）。锁足/眩晕等控制效果暂未建模。
- **鹿鸣**：PvE 奇穴（宫改为 6 秒调息 + 持续运功消耗曲风直到耗尽，GCD 后可用其他技能打断），本轮不建模。新增 `QixueDef.shelved` 标记（DTO/校验同步）：`shelved = true` 的奇穴只登记名称与槽位，允许没有替换/修正/钩子。
- 回归：`crates/jpcg_combo/tests/qixue_hooks.rs::luanyu_replaces_qingxiao_with_dash_attack` 验证替换后技能名为「飞鸾动羽」且有伤害；`cargo test -p jpcg_core` 72、`-p jpcg_combo` 37 + 7 全绿，fmt 通过。

### 来自 changes/2026-11-05-135500.md

# 配装器接入 50 级真实装备数据（威望/名剑/跨门派/镶嵌/附魔）

## 做什么

- `data/gear/templates.toml` 换成游戏内实测数据：
  - 恶人谷威望「妄戏浮屠·骋芳」12 部位 × 基础/升级两档；
  - 名剑大会「一诺霜明·鸾律」8 件（无饰品）× 两档；
  - 冰心「一诺霜明·星鼓」帽/护手作为跨门派装备（`cross_school = true`，穿别派装备不计套装）。
- 新增 `data/gear/sockets.toml`（镶嵌孔，按 门派×来源×档位×部位×属性套 定位）与 `data/gear/enchants.toml`（附魔，按部位 + id 选择），`Loadout.enchants` 记录选择。
- 套装语义按游戏修正：防具 [2] 属性、[4] CD（`笑傲光阴 -9s` / `孤影化双 -15s`），饰品 [2] 御劲 +120、[3] 化劲 +120 且两档同时生效；「·悟」条目仅作用于无界，不参与 PC 计算。
- `Catalog` 加载新增 `sockets.toml`/`enchants.toml`，六个文件 version 必须一致。

## 为什么

直接把配装器从合成样例切到真实数据，才能验证「属性换算 + 合成 + 优化器」整条链路。

## 关键实现

- `Template.cross_school`、`Choice.school`：跨门派装备可用但不参与该门派套装计数与覆盖率校验。
- `SocketDef`/`EnchantDef` 与人填 TOML 对接；`compose` 先加装备本体，再叠加镶嵌孔与附魔。
- 饰品套装允许历史合成样例的「2 件化劲、3 件御劲」顺序（游戏顺序与旧样例相反）。
- 测试拆出 `crates/jpcg_gear/tests/fixtures/synthetic/`（旧合成数据 + 值集），真实数据只服务应用与属性测试。

## 验证

- `cargo test --workspace` 全绿（core 72 / combo 37+7 / gear 18 + 属性 6 + host 1 + dynamic bridge 2 等）。
- `JPCG_DATA_DIR=data cargo run -p jpcg_gear --bin gear-opt -- --catalog data/gear optimize --request data/gear/sample-request.toml` 在真实数据上跑通（16,777,216 组合空间，预算内评估 64 个候选）。

## 备注

- 基础档 1260 品按品级系数推算，等拿到实装数值再替换。
- 属性换算系数来自游戏内截图反推，强化/附魔/镶嵌已计入，完整战斗增益与装备特效仍待补。

### 来自 changes/2026-11-05-140500.md

# 附魔候选按属性枚举

## 做什么

- `data/gear/enchants.toml` 从「每部位一条通用附魔」扩展为「同数值、不同属性」的候选集：
  - 戒指（jie1/jie2）：全攻击 / 根骨 / 身法 / 元气 / 力道 / 内功攻击，均 126；
  - 项链：御劲 126、体质 126；
  - 腰坠：化劲 126、体质 126、全会心 126。
- 材质名（铸世·归正 / 息烽 等）只影响外观、数值相同，数据层只保留一个 id。

## 为什么

游戏内附魔是「同数值、可选属性」，玩家会按门派主属性选；原数据只给通用一条，优化器无法权衡。

## 验证

- `cargo test -p jpcg_gear` 全绿；`gear-opt optimize` 在真实数据上跑通。

### 来自 changes/2026-11-05-141000.md

# 前端配装页接入镶嵌与附魔

## 做什么

- `types/gear.ts`：`GearLoadout.enchants`、`GearChoice.school`、`GearTemplate.cross_school`、`GearSocketDef`、`GearEnchantDef`，`GearCatalog.sockets/enchants` 可选字段。
- `state/gear.ts`：`initialLoadout` 按部位取第一条附魔作为默认选择；`isGearLoadout` 校验 `enchants` 形状。
- `components/GearPanel.tsx`：每张装备卡新增「附魔」下拉（按部位过滤候选，含「无」），并在「属性详情」前展示该装备的镶嵌孔属性。

## 为什么

配装器数据已有镶嵌/附魔表，前端需要能选、能看到，否则这些数据只停留在引擎侧。

## 验证

- `npx tsc --noEmit` 通过；`npm test` 10 文件 108 项全绿；`npm run build` 通过。

### 来自 changes/2026-11-05-141500.md

# 时间轴 Cast 事件输出 GCD 信息

## 做什么

- `crates/jpcg_combo/src/timeline.rs`：`TimelineEventKind::Cast` 新增 `gcd_secs`（本次占用的 GCD，不占 GCD 为 0）与 `gcd_ready_at`（该通道就绪时刻），在施放时一并产出。
- `crates/jpcg_api/src/lib.rs`：`TimelineEventKindDTO::Cast` 同步两个字段（`#[serde(default)]` 兼容旧数据）。
- `crates/jpcg_combo/src/timeline_host.rs` 转换同步。
- 前端：`TimelineEventKind` 类型补字段，`timelineFormat.ts` 在施放行追加「GCD 1.50s（就绪 1.50s）」，不占 GCD 的技能不显示。

## 为什么

排轴后看不到 GCD 何时就绪，无法判断技能为什么排在那一步；GCD 之前只在引擎内部计算，没有输出。

## 验证

- `cargo test -p jpcg_core -p jpcg_combo -p jpcg_gear` 全绿（含新增 GCD 字段断言）。
- 前端 `tsc` 通过、`npm test` 108 项通过。

### 来自 changes/2026-11-05-142000.md

# 镶嵌宝石化 + 伤害记录模块

## 做什么

- 后端：镶嵌从「每件装备固定孔位属性」改为「模板声明孔位数量 + 默认宝石，玩家按孔位选宝石」。
  - `jpcg_gear::model`：`SocketDef` 换成 `GemDef { id, name, stats }`；`Catalog.gems`；`Template.socket_count/default_gems`；`Loadout.sockets: BTreeMap<Slot, Vec<String>>`。
  - `catalog::compose` 按 loadout 的宝石 id（缺省回退模板默认）累加属性，校验孔位数量与宝石存在性；`validate()` 校验宝石 id 唯一、模板孔位数与默认宝石数一致、孔位 ≤3。
  - 新增 `data/gear/gems.toml`（八级宝石：主属性/根骨/体质/身法/元气/力道 38、斑质 57、全攻击/全会心/会心效果/全破防/化劲/御劲/加速 38、气血上限 577），删除 `sockets.toml`；`templates.toml` 补齐 `socket_count/default_gems`。
- 前端：配装卡新增每孔宝石下拉（八级），`initialLoadout` 用模板默认宝石初始化，`isGearLoadout` 校验 sockets。
- 前端：新增 `TimelineDamageLog` 时间轴伤害记录组件——`# / 时间(mm:ss) / 类型 / 技能 / 伤害 / 累计 / 备注`，备注里显示 GCD（或不占 GCD）、DOT 跳数、形态切换、增益层数；连招页时间轴预览改用它。

## 为什么

- 孔位宝石属于玩家可选项，固化成装备属性会让配装器无法表达「同装备不同宝石」；只录入八级符合当前版本实际。
- 时间轴原来只有一行行原始事件，看不出「几分几秒、什么伤害、打了多少」；新表格按时间顺序列出并累计。

## 验证

- `cargo test -p jpcg_gear` 全绿（含宝石校验与跨门派、饰品套装）。
- 前端 `tsc --noEmit`、`npm test` 109 项、`npm run build` 通过。

### 来自 changes/2026-11-05-143000.md

# 连招序列按形态校验可用性

## 做什么

- 新增 `examples/jpcg_app/src/utils/formLegality.ts`：按序列顺序模拟形态切换（读取 combat overlay 的 `form` 与 `on_cast.switch_form`），逐步判断技能在当前形态下是否可用。
- 连招页：不可用的步骤标红并显示「非法」+ 悬停说明（当前形态 / 所需形态）；顶部给出提示条；存在非法步骤时禁用「时间轴预览」。
- 引擎侧本来就会跳过不匹配形态的技能（`skipped_actions`），前端不再让用户提交后才看到这个结果。

## 为什么

切形态技能穿插在序列里时，形态错了的技能会被引擎静默跳过，玩家只能从 `skipped` 数字察觉；提前标红可以避免白算一遍。

## 验证

- 前端 `tsc --noEmit`、`npm test` 112 项（新增 3 项形态校验用例）、`npm run build` 通过。

### 来自 changes/2026-11-05-144500.md

# 心法固定攻击计入基础攻击力

## 做什么

- `data/shuxing/mowen.toml`：心法固定内功攻击 2282 的 `AttributeConstant` 目标从 `gongji_extra` 改为 `gongji_base`。

## 为什么

游戏面板的「基础攻击力」包含心法固定攻击（例如基础 6604 = 装备攻击 + 2282），只有「每点根骨的 0.72 攻击」属于心法额外收益、计入最终攻击而不进基础。之前把 2282 放进 extra，导致配装器显示的基础攻击偏低约 2282。

## 验证

- `cargo test -p jpcg_gear` 全绿（含 `panel.gongji == atk_with_buff 结果` 的一致性断言：基础 + 额外 = 最终，拆分变化不改变总攻击）。

### 来自 changes/2026-11-05-145000.md

# 前端可选跨门派装备

## 做什么

- `GearPanel` 的候选列表从 `t.school === loadout.school` 放宽为 `t.school === loadout.school || t.cross_school`；选择跨门派装备时把 `choice.school` 写成装备所属门派（同门派写 null）。
- 跨门派按钮前缀显示门派名（冰心），悬停提示「可穿戴但不计套装」。
- `initialLoadout` 优先选本门派模板，没有时回退跨门派模板。

## 为什么

冰心护手/帽已作为跨门派装备录入（`cross_school = true`），但前端只列本门派模板，导致用户看不到也选不了。

## 验证

- 前端 `tsc --noEmit`、`npm test` 112 项、`npm run build` 通过。

### 来自 changes/2026-11-05-150000.md

# 镶嵌孔恢复为装备固定属性

## 做什么

- `jpcg_gear`：删除宝石可选模型（`GemDef` / `Catalog.gems` / `Template.socket_count/default_gems` / `Loadout.sockets`），改为 `Template.sockets: Vec<Stats>`——每件装备的五行石孔位属性固定，直接计入装备属性。
- 数据：`data/gear/gems.toml` 删除，孔位属性写回 `templates.toml` 的 `[[templates.sockets]]`。
- 前端：配装卡去掉每孔宝石下拉，改为只读展示「镶嵌（固定）：根骨 +38、化劲 +38」。

## 为什么

五行石孔不是玩家可选项，每件装备固定这几个孔和属性；做成下拉会让人误以为能换宝石，也容易和实际装备对不上。

## 验证

- `cargo test -p jpcg_gear` 13 个目标全绿。
- 前端 `tsc --noEmit`、`npm test` 112 项、`npm run build` 通过。

### 来自 changes/2026-11-05-150500.md

# 武器五彩石计入属性

## 做什么

- `jpcg_gear::model::Template` 新增 `wucai: Option<Stats>`（武器五彩石，固定属性）；`catalog::compose` 把五彩石属性一并计入该件装备。
- `data/gear/templates.toml`：四件武器（威望/名剑 × 基础/升级）补 `[templates.wucai]`：根骨 +52、内功会心效果 +104、内功攻击 +208（游戏内 1320 武器截图；基础档按同值暂用）。
- 前端配装卡展示「五彩石：根骨 +52、会效 +104、内功攻击 +208」。

## 为什么

武器除了三个镶嵌孔还有一颗五彩石，之前的属性只加了孔位，漏掉五彩石，导致攻击与根骨都偏低。

## 验证

- `cargo test -p jpcg_gear` 13 个目标全绿。
- 前端 `tsc --noEmit`、`npm test` 112 项、`npm run build` 通过。

### 来自 changes/2026-11-05-151000.md

# 五彩石改为可选

## 做什么

- `jpcg_gear`：`Template.wucai`（固定）改为独立的五彩石候选 `Catalog.wucai: Vec<WucaiDef>` + `Loadout.wucai: Option<String>`；compose 仅在武器部位按所选 id 加成，未选则不加。
- 新增 `data/gear/wucai.toml`（目前一条：根骨 +52 / 内功会效 +104 / 内功攻击 +208，来自 1320 武器截图）。
- 前端：武器卡新增五彩石下拉（默认第一条，可选「不插五彩石」）。

## 为什么

五彩石是玩家可选择的镶嵌物，不是武器固有属性；之前烤进武器模板会导致换成别的五彩石时算错。

## 验证

- `cargo test -p jpcg_gear` 13 个目标全绿。
- 前端 `tsc --noEmit`、`npm test` 112 项、`npm run build` 通过。

### 来自 changes/2026-11-05-151500.md

# 修复旧存档 loadout 携带已删除字段导致 compose 失败

## 做什么

- 新增 `sanitizeGearLoadout`：发送给 Rust 前剔除历史遗留字段（如已删除的 `sockets`），保留当前结构字段。
- `api.composeGear` / `api.exportGear` 调用前统一 sanitize。
- 回归测试：带 `sockets` 的旧 loadout 清理后不含该字段，其余字段不变。

## 为什么

本地持久化的旧配装在版本升级后仍带 `sockets`，Rust 的 `Loadout` 用了 `deny_unknown_fields`，于是报 `unknown field sockets`，整页只能「重新读取模板」。

## 验证

- 前端 `tsc --noEmit`、`npm test` 113 项、`npm run build` 通过。

### 来自 changes/2026-11-05-152000.md

# 修正项链/腰坠附魔候选

## 做什么

- `data/gear/enchants.toml`：项链附魔 = 御劲 126 / 体质 126；腰坠附魔 = 化劲 126 / 体质 126；删除腰坠上错误的「全会心 126」。

## 为什么

游戏里项链与腰坠的附魔只有两种：一条御劲、一条化劲，且两件都可以选体质；之前多写了全会心。

## 验证

- `cargo test -p jpcg_gear` 13 个目标全绿。

### 来自 changes/2026-11-05-152500.md

# 附魔名称改成属性文案

## 做什么

- `data/gear/enchants.toml`：把「附魔提高90/126」这类占位名称改成实际属性文案（化劲等级提高90、御劲等级提高126、化劲等级提高126、加速等级提高126）。

## 为什么

下拉里显示「附魔提高126」看不出加的是什么属性；游戏内文案本身就是「化劲等级提高126」这种写法。

## 验证

- `cargo test -p jpcg_gear` 13 个目标全绿。

### 来自 changes/2026-11-05-153000.md

# 补上体质→气血换算，对齐魔盒面板

## 做什么

- `data/shuxing/mowen.toml` 的 `[[attributes.conversions]]`：`tizhi → qixue_base` 由 `0/1` 改为 `21/2`（每点体质 10.5 气血），与游戏面板「最大气血」一致。

## 为什么

之前把该换算显式写成 0，导致配装面板气血恒为「额外气血」部分。用魔盒配装器（根骨 3351 / 气血 73014）对照后确认需要 体质×10.5 这一项。

## 验证

- `cargo test -p jpcg_gear` 全绿；用魔盒配置 compose 后：气血 72206（目标 73014）、攻击 10352（10367）、破防 3585（3586）、会心 1571（1576）、御劲 246 / 化劲 4457 / 加速 974 完全一致。
- 剩余差异见后续核对（五彩石/体质数值）。

### 来自 changes/2026-11-05-154000.md

# 气血公式与角色基础属性对齐游戏面板

## 做什么

- `data/shuxing/mowen.toml`：`tizhi → qixue_base` 改为 `10/1`，并新增常量 `qixue_base = 3956`（50 级基础气血）。
- 前端 `initialLoadout` 的 `base` 补上角色自带基础属性：体质 18、根骨 18、身法 18、元气 17、力道 17。

## 为什么

游戏面板气血 tooltip：气血值提高 = 体质×10；基础气血最大值 = 体质×10 + 3956；最终 = 基础 + 额外气血上限。之前用 `体质×10.5` 且把角色基础属性当 0，导致气血与根骨各少一截。

## 验证

用魔盒配装（威望/名剑混穿，12 件全 1320）对照：
- 根骨 3351 / 会心 1576 / 气血 73014 / 御劲 246 / 化劲 4457 / 加速 974 —— 与魔盒完全一致。
- 基础攻击 7953（魔盒 7956）、攻击 10365（10367）、破防 3585（3586），差异 1~3 点，疑为单件攻击词条取整，待核。
- `cargo test --workspace` 与前端 113 项测试全绿。

### 来自 changes/2026-11-05-154500.md

# 角色自带基础攻击 +3

## 做什么

- `data/shuxing/mowen.toml` 新增 `AttributeConstant { to = GongjiBase, value = 3 }`：50 级成女无装备面板基础攻击为 3。

## 为什么

与魔盒配装对照，基础攻击差 3 点；这 3 点来自角色本身（和自带的体质/根骨 18 点一样），不属于任何装备或心法。

## 验证

- 魔盒配装 compose：基础攻击 7956（与魔盒一致）、攻击力 10368（魔盒 10367）、破防 3585（3586），其余全部一致。

### 来自 changes/2026-11-05-155000.md

# 补角色自带破防 1 点

## 做什么

- `data/shuxing/mowen.toml` 新增 `AttributeConstant { to = Pofang, value = 1 }`。

## 为什么

游戏面板「基础阴性内功破防 3586」，我们装备+镶嵌+附魔合计 3585，差 1 点来自角色/心法自带（和基础攻击 3、体质/根骨 18 同类）。

## 验证

- 魔盒配装 compose：破防 3586 ✅；基础攻击 7956 ✅；根骨/会心/气血/御劲/化劲/加速全一致；攻击力 10368（魔盒 10367，差 1，待查取整口径）。
- `cargo test -p jpcg_gear` 13 个目标全绿。

### 来自 changes/2026-11-06-065600.md

# 修复导数分析沿用旧计算环境的结果

## 什么与为什么

加点优化页面首次访问后保持挂载，但导数结果、错误和加载状态没有绑定输入版本。返回计算页修改属性、心法或值集，或配装结果暂不可用后恢复，旧推荐仍显示；在途旧请求也能回填并结束较新请求的加载状态。

- 输入引用变化时在布局 effect 内清空结果、错误、展开及图表状态，并递增请求版本；卸载同样使请求失效。
- 成功、失败、finally 均只接纳当前请求。当前请求失败与重试行为不变；不取消已经进入宿主的计算。
- 新增组件和真实 App 导航回归，验证常驻页面、迟到成功/失败、A→B→A、null 恢复和普通重渲染。

## 验证与范围

在 dot 云电脑使用原锁文件和 Node 24.22.1。新增回归先在未修改的 dev 实跑失败，再验证修复版本；完整前端测试、TypeScript/Vite 构建及 Makefile 顺序守卫通过（精确计数见 PR）。没有改公式、Rust、DTO、游戏数据、依赖、存储格式或版本。

本机 make check-all 在首步因 cargo 不存在退出；Rust、金标准与桌面双模式尚未本机执行，远端 CI 另列。组件测试使用 jsdom 和 API mock，未验证原生 Tauri IPC、macOS/Windows GUI。保留既有 Vite 大 chunk 提示。旧结果不再跨输入保留，需要用户重新点击计算导数。

### 来自 changes/2026-11-06-162000.md

# 武学页：按效果图重排三栏（技能目录 / 排轴 / 模拟伤害记录）+ 奇穴盘

## 做了什么

- `examples/jpcg_app/src/components/ComboPage.tsx`：标题改为「武学」；主体重排为三栏网格：
  左「技能目录」、中「排轴」、右「模拟伤害记录」（底部奇穴盘整宽）。
- 技能目录：由文字 chip 改为技能卡：按器形色调区分（琴绿 / 剑紫），展示基础伤害与
  `攻击加成 = atk_xishu × 面板攻击力`、吟唱/调息/充能标签；每张卡按 `skill_id/sub_id`
  列出该技能的秘籍 pill，点击即可勾选/取消（替代原来的全局秘籍列表）。
- 排轴：每步显示模拟时间、序号、技能名、器形/曲风、已选秘籍与 GCD；
  `form_switch` / `qifeng_switch` 事件渲染为步骤内的切换行；非法步骤保留红框。
- 模拟伤害记录：右侧表格 + 三个总计卡片（窗口总伤 / DPS / 未执行动作）；
  未计算时显示引导空态。
- 顶部状态条补全：器形、曲风、目标血量、内防、减伤，以及「切至阳春/高山/平沙」曲目快捷按钮。
- `src/App.tsx`：排轴器不再固定传 `configurationOnly`，武学页可直接运行「计算连招 / 时间轴预览」
  （此前该页只做配置，时间轴入口不可达 —— 对应 PR #104 复审 P1）。
- `src/types/index.ts`：`MijiDefDTO` 补 `skill_id/sub_id`；`TimelineDamageLog` 支持 `qifeng_switch`。
- `src/api/commands.ts`：浏览器（非 Tauri）预览用 mock 补两轴 combat 数据（琴/剑、三种曲风、
  秘籍、奇穴槽位），便于无桌面环境检查视觉；不参与真实计算。

## 为什么

- 用户要求按 `martial-mock5.png` 效果图「完全不缩水」实现；此前只有竖排结构，差距明显。
- 秘籍/奇穴/形态信息需要落到技能与排轴这一步，才能体现「武学系统」而不是单纯的配置表单。

## 验证

- `npx tsc --noEmit` 通过；`npm test` 115 项通过；`npm run build` 通过。
- 视觉：headless Chrome + CDP 注入草稿与浏览器 mock 数据，截图核对三栏布局与配色。

### 来自 changes/2026-11-06-163500.md

# 武学页：技能卡内置 4 个秘籍槽；奇穴盘分组重排

## 做了什么

- 秘籍从「底部全局勾选列表」改为**挂在技能卡左侧的 4 个槽位**（对应游戏内 4 个秘籍槽）：
  - 空槽显示「＋」，点击弹出该技能的秘籍候选；已装备槽显示秘籍名，点击卸下。
  - 同一技能最多装备 4 个秘籍（第 5 个点击被忽略），弹层标题显示「n/4」。
  - 没有秘籍的技能不再渲染槽位列。
- 删除底部重复的全局秘籍勾选面板（秘籍归属到技能后不需要重复入口）。
- 奇穴盘重排：第七~九重合并为单个「混选（9 选 3）」组；布局由竖排行改为 4 列分组卡片网格，
  选中项用金色描边/底色，标题栏显示「已选 n / 9」。
- 整页改为固定高度布局（头部 + 三栏 + 底部奇穴盘），三栏各自内部滚动，
  奇穴盘不再被挤出视口。

## 为什么

- 用户反馈：秘籍应放到技能左边、每个技能最多 4 个；奇穴盘视觉不好看。
- 固定高度布局让「奇穴盘放下方」在 1440×900 下成立，避免需要整页滚动才能看到。

## 验证

- `npx tsc --noEmit`、`npm test`（115 项）、`npm run build` 全绿。
- 视觉：无头 Chrome + CDP 截图核对（`.scratch/mockup/martial-implemented2.png`）。

### 来自 changes/2026-11-06-170000.md

# 武学页：独立 DOT 数据修复 + 左栏技能格重排

## 做了什么

### 1. 修复 DOT 在时间轴上丢失
- `crates/jpcg_core/src/type_set/combat.rs` / `crates/jpcg_api/src/lib.rs`：`SkillCombatDef`
  新增独立 DOT 字段 `dot_interval` / `dot_duration` / `dot_tick_base1` / `dot_tick_base2` / `dot_tick_xishu`；
  `validate()` 校验间隔、时长与系数。
- `crates/jpcg_combo/src/timeline.rs`：新增 `DotTickSpec`；`TimingSkill.dot_tick` 在
  `from_combat` 里按 overlay 克隆一份「每跳技能」（dot_flag/interval/duration/每跳伤害系数），
  施放本体后独立生成 DOT 实例并按跳结算；本体直接伤害不受影响。
- `crates/jpcg_combo/src/timeline_host.rs`：独立 DOT 同样套秘籍/奇穴修正（殊曲缩短间隔、
  冻泉加跳数），resolve 后回写 interval/duration。
- `data/combat/mowen.toml`：商（14065/14311）、角（14066/14312）补
  `dot_interval=3 / dot_duration=18 / 每跳 +750 或 +731`（`dot_tick_xishu = 750/8438`、`731/8438`）。
- 回归：`dot_ticks_come_from_combat_overlay`（6 跳）、`dot_modifiers_apply_to_overlay_dot`
  （殊曲 9 跳、冻泉 10 跳）。

### 2. 左栏技能目录重排（方案 A）
- 标题行右侧新增统一「秘籍配置」入口；弹窗左侧列技能（含 n/4），右侧 4 槽位 + 候选秘籍。
- 搜索框下新增 `全部 / 曲 / 剑 / 已收藏` 过滤。
- 技能格从 3 列 84px 改为 2 列（≥118px），技能名两行显示；去掉每格小齿轮，
  只保留右下角 `n/4` 徽标（点击打开同一个秘籍弹窗并定位该技能）。

### 3. 时间轴自动模拟（上一批改动）
- 移除「时间轴预览」按钮；排轴/秘籍/奇穴/面板属性变化后 350ms 防抖自动重算，
  revision 守卫旧请求不覆盖新结果。

## 验证
- `cargo test --workspace` 全绿、`cargo fmt --check` 通过。
- 前端 `tsc`、116 项 vitest、`npm run build` 通过。
- 视觉：headless Chrome + CDP 截图核对（`.scratch/mockup/martial-left-v2.png`、`martial-miji-v2.png`）。

## 补充（同一批）

- 殊曲 DOT 每跳递增修复：`SkillModifier::DotUpAdd`（+0.12），数据 `shuqu` 由误用的
  `hit_up_add = 12` 改为 `dot_up_add = 0.12`；商 DOT 9 跳逐跳递增（回归测试）。
- 洞天·变宫：新增 `QixueHook::AccelerateDot`，洞天 3 跳各触发一次「商/角下一跳立刻生效」
  （提前，不补跳；总跳数不变），`data/combat/mowen.toml` 的 dongtian 挂 hooks；回归测试断言
  商首跳提前且仍为 6 跳。
- 奇穴门控：左侧技能目录隐藏 `jihuoqixue` 未激活的技能（激活对应奇穴后立即出现），
  前端测试覆盖；奇穴盘中 18 个 `shelved` 描述性奇穴用斜体/虚线 + tooltip 标注「引擎暂未建模」。

## 补充 2：徵 / 变徵 倒读条三段

- `Skilltype.channel_ticks`（默认 1）与 overlay `channel_ticks`：引导技能按段结算。
- 引擎新增 `TimelineEventKind::ChannelTick`；引导施放时 `Cast` 事件伤害记 0，
  各段在其结算时刻（`start + 时长×k/段数`）输出，时长走 `cast_time_with_haste`（吃加速/帧量化）。
- `data/combat/mowen.toml`：徵 14067:14227、变徵 14299:18859 设 `cast_time = 2.8125`、
  `channel = true`、`channel_ticks = 3`；变徵保留 `requires_qifeng = "gaoshan"`。
- 回归测试 `zheng_channel_has_three_ticks`：3 段、首段 0.9375s、末段 2.8125s、Cast 伤害为 0。
- 前端 `TimelineEventKindDTO` 支持 `channel_tick`，日志显示「引导 · 第 N 段」。

## 补充 3：羽「商/角 下两跳立刻生效」

- 依据游戏内 tooltip：羽「对目标造成 156–175(+2717) 阴性内功伤害，使其下两跳自身『商』『角』持续伤害立刻生效」。
- `OnCast` 新增 `advance_dot_targets` / `advance_dot_ticks`（DTO 同步）；引擎在施放时把目标 DOT 的最早 N 跳时间改到命中时刻
  （只提前、不补跳，总跳数不变）。羽每次施放（3 层充能各一次）都会触发。
- 数据：`data/combat/mowen.toml` 的 14068:14100 增加
  `on_cast = { advance_dot_targets = [[14065,14311],[14066,14312]], advance_dot_ticks = 2 }`。
- 回归测试 `yu_triggers_two_shang_dot_ticks`：商仍 6 跳，其中 2 跳被提前到羽的命中时刻（1.5s）。

## 补充 4：秘籍只显示效果，不显示秘籍名

- 新增 `src/utils/miji.ts`：把 `SkillModifierDTO` 翻译成效果文案
  （如 `cast_time_add: -0.125` → 「运功时间减少0.125秒」、`hit_up_add: 4` → 「伤害提高4%」、
  `huixin_up_add: 4` → 「会心提高4%」、`dot_up_add` → 「每跳伤害提高12%」等），多条用「，」连接。
- 秘籍配置弹窗的技能槽位、候选列表、排轴步骤里的秘籍标签全部改为显示效果文案；
  无修正项的秘籍显示「无数值效果」。
- 前端类型补 `SkillModifierDTO` / `MijiDefDTO.modifiers`；浏览器 mock 与 UI 测试同步更新；
  弹窗背景改用 `--bg-surface`，深浅色主题都正常。

## 补充 5：武学面板改为后端组装（前端只渲染）

- 数据/组装：
  - `SkillCombatDTO.miji_ids`：后端按 `(skill_id, sub_id)` 精确配对秘籍；`sub_id=0` 仅在技能唯一时展开，
    歧义（同一 skill_id 多个变体）或引用不存在直接报错。
  - `MijiDefDTO.effects`：后端从 modifiers 生成简短文案（如「运功-0.125秒」「伤害+4%」）。
  - `QixueDefDTO.group_label` / `group_ids`：后端按槽位输出分组标签，并把「冻泉 + 冻泉·宫」合并成一格
    （group_ids 一起开关）；数组顺序即显示顺序。
  - `CombatMetaDTO`：新增 `miji_per_skill_cap` / `mixed_qixue_cap` / `mixed_slot_from`。
  - 组装入口 `host::combat::load_combat_with_skills` 统一做上述整理，FFI/Tauri 复用。
- 数据修正（错位根因）：`data/combat/mowen.toml` 的秘籍 skill_id 有 off-by-one
  （徵写成了 14066、羽写成了 14067），已改回 14067/14068；商/徵/羽补显式 sub_id
  （14311/14227/14100），否则会被组装期歧义检查拒绝。
- 前端：删除 `utils/miji.ts`、`mijiFor` 的匹配规则、奇穴分组/合并/上限逻辑；`ComboPage` 只按
  `miji_ids` 查秘籍、渲染 `effects`、按 `group_label` 顺序渲染奇穴、用 `group_ids` 开关、上限读 meta。
- 测试：Rust `panel_bundle_is_enriched_by_backend`（配对/文案/分组/合并/caps）；前端 117 项通过；build 通过。

## 补充 6：DOT「下一跳立刻生效」后计时重新起算（级联前移）

- 之前只把被点名的那一跳挪到触发时刻，后续跳仍按原时间生效（看起来「后面没挤过来」）。
- 改为：该跳移到触发时刻后，后续所有跳整体前移相同 delta（总跳数不变，DOT 结束时间提前），
  等同游戏里触发后跳间隔重新起算。
- 回归测试更新：羽后商末跳 < 17s、洞天后商末跳 < 17s（原 18s）。

## 补充 7：DOT 提前标记

- `TimelineEventKind::DotTick` 增加 `advanced: bool`（DTO/前端同步）；被洞天/羽提前的跳在日志备注显示「（提前）」，
  避免和正常跳混淆。回归测试断言羽提前的 2 跳带标记。

## 补充 8：DOT 提前改为「按次消费下一跳」（修复第三跳洞天没炸）

- 旧实现按时间搜「第一个大于触发时刻的跳」，级联前移后后面的触发可能找不到目标或跳过，导致少炸。
- 改为每个 DOT 记录 `consumed` 游标：每来一次触发事件（洞天每跳 / 羽 2 跳）消费「下一条未炸的跳」，
  该跳落到触发时刻，后续跳按 interval 重新排；因此洞天 3 跳必炸 3 次商 + 3 次角，雨露均沾。
- 回归测试：洞天场景断言带 advanced 标记的商跳恰为 3 条；羽场景为 2 条。

## 补充 9：DOT 重复施放改为刷新（覆盖旧实例）

- 之前同一技能再施放会新建 DOT 实例，日志里会出现两段「洞天第 1/2/3 跳」交错，
  看起来像跳数错位/多出第 4 跳。
- 现在施放带 DOT 的技能（商/角/洞天等）会先移除同 (skill_id, sub_id) 的旧 DOT 实例再新建，
  即刷新而不是叠加；DOT 跳序号始终从 1 开始。

## 补充 10：秘籍/奇穴选择持久化

- `ComboDraft` 增加 `miji` / `qixue` 字段（连招草稿已有持久化通道 `usePersistentState`，按心法分 key）；
- 旧存档迁移：`comboDraftSchema` version 1→2，缺省补空数组；旧 cockpit 快照里的 comboDraft 校验改为容忍缺省；
- `ComboPage` 的 `selMiji` / `selQixue` 不再用组件内 useState，改为读写草稿字段，切换页面/重挂载/重启后选择保持；
- 奇穴 chip 增加 `aria-pressed`；新增回归测试验证秘籍、奇穴写入草稿且重挂载后恢复（118 项前端测试通过）。

## 补充 11：伤害记录时间显示 0.1s 精度

- 之前用 mm:ss，洞天施放与 0.5 秒后的首跳落在同一秒里，看起来像「施放瞬间就提前了」。
- 现在时间列显示 `M:SS.s`（如 0:04.0 / 0:04.5），并新增回归断言：洞天施放时刻（1.5s）不触发提前，
  只有 2.0 / 2.5 / 3.0 三个 DOT 跳才各炸一跳。

## 补充 12：提前只作用于「尚未生效」的跳

- 问题：洞天施放时若目标 DOT 的下一跳其实已经在施放时刻前/同时生效，旧逻辑仍把它「提前」到洞天跳的时刻
  （实际被推后），却仍标「（提前）」。
- 修复：消费游标先跳过 `tick_time <= 触发时刻` 的已生效跳，只把未来跳提前到触发时刻。
- 验证：无加速场景洞天 3 跳仍炸 3 次商；有加速场景中施放时刻已过的第 1 跳不再被搬动/标记。

## 补充 13：DOT 刷新保留已生效跳

- 问题：连续两次洞天时，刷新逻辑直接删掉旧 DOT 实例，导致第一次已经打出的跳从日志里消失
  （只剩商/角的提前行，看不到洞天自己的第 1/2 跳）。
- 修复：`push_dot` 刷新时保留 `tick_time <= 施放时刻` 的已生效跳，再追加新的跳（序号连续）；
  回归测试断言两次洞天后洞天跳序号为 1..6。

## 补充 14：洞天刷新后跳序号重新从 1 开始

- 刷新（重复施放）时保留已生效跳的原序号，新的一轮 3 跳重新编号 1/2/3；
  「施放后第一跳」不会再显示成第 2 跳。回归测试：两次洞天序号为 [1,2,3,1,2,3]。

## 补充 15：修复刷新分支漏重置跳序号

- 上一版只在新实例创建时初始化 tick_labels，刷新分支没重置，导致洞天刷新后仍显示第 2/3 跳。
- 现在刷新时：保留已生效跳的原序号，新一轮标签 1..n；带加速场景（GCD 1.31s）回归测试通过。

## 补充 16：DOT 跳序号前端重复 +1（编号从 2 开始）

- 后端 `DotTick.index` 已是 1-based（洞天/商/角都是 1,2,3…），前端 `TimelineDamageLog` 又做了 `index + 1`，
  导致所有持续跳从「第 2 跳」开始显示。
- 修复：前端直接使用后端 index；测试 fixture 改为 1-based。前端 118 项测试 + build 通过。

## 补充 17：所有门派新增「停手」结构空位技能

- 需求：排轴需要一个「不打伤害」的占位技能。设计为伤害 0、无属性、占普通 GCD 的技能，名为「停手」。
- 实现：`Skilltype::idle()` + `IDLE_SKILL_ID = 999999`；
  - `store::combat::load_skills_with_combat` 注入技能表（时序可用）；
  - `host::skill::load_skill_pool` 注入技能池（所有门派目录可见）；
  - 不在任何门派 TOML 里写死，避免逐门派维护。
- 行为：瞬发、伤害 0、无系数/元素/DOT、无调息与充能；不占曲风、不要求器形/曲风，任何形态可用；占一个普通 GCD，
  因此可当作「等待/停手」时间空位。
- 测试：`idle_skill_exists_and_occupies_gcd` 断言技能池含停手且数值全 0、0s 施放伤害 0、下一个技能被顶到 1.5s。
- 前端：mock 技能池同步加入停手，截图可见技能格；DOT 首跳编号也顺带修正为 1-based。

## 补充 18：前端去计算化审计（步骤/累计/合法性全部后端）

- 后端 `TimelineResultDTO` 新增：
  - `steps[]`：每个动作的施放时刻、GCD、执行前器形/曲风、是否合法、跳过原因、缺失的器形/曲风要求、动作前的切换事件；
  - 每个事件的 `cumulative`（累计伤害）；
  - `final_instrument` / `final_qifeng`（序列执行完后的状态）。
- 前端删除：
  - `utils/formLegality.ts` + 测试（器形/曲风合法性模拟）；
  - `stepMeta`（从事件反推每步时间/GCD/切换）；
  - 伤害记录里的累计求和；技能卡上的秘籍匹配改为使用后端 `miji_ids`。
- 顺带修复：徵/变徵引导伤害（ChannelTick）之前未计入总伤/DPS，已补上并有测试。
- 新增 Rust 测试：`steps_are_computed_by_backend`、`channel_damage_is_included_in_total`。
- 前端测试从 118 降到 113（删掉的 5 项 formLegality 测试），其余全绿。

### 来自 changes/2026-11-07-015600.md

# 检查：后端是否全门派通用 + 曲风通用化计划

## 背景
用户要求确认后端代码没有针对莫问的单独优化（门派差异应只存在于 TOML），并把「曲风 → 全门派通用能量机制」
写入计划（暂不实现）。

## 审计结果（详情见 PLAN.md M5）
- 运行路径无门派分支：`mowen` / `莫问` / 技能名 / 技能 id 只出现在单测与测试默认请求中；
  门派差异全部由 `data/shuxing/*.toml` + `data/combat/*.toml` 提供。
- 3 类待泛化：
  1. 曲风点数资源写死在引擎：`MAX_QIFENG=9`、`qifeng_points u8`、初始满点、先消耗再获得；
     DTO/事件/钩子均有 qifeng 专用字段（`requires_qifeng`、`qifeng_gain`、`spend_all_qifeng`、`QifengSwitch`、
     `per_qifeng_spent`、`DamagePctPerQifeng`）。
  2. 面板规则默认值按莫问写死：miji 4、混选 3、混选从第 7 格起（`jpcg_api` 默认常量）。
  3. 前端硬编码曲风显示名 `QIFENG_NAMES`。
- 仓库目前只有莫问数据文件（+ 模板），跨门派回归需要虚构门派测试数据。

## 计划
`PLAN.md` 新增 M5：资源定义数据化 → DTO 双写兼容 → 引擎状态资源化 → 事件/钩子资源化 →
前端去曲风名称硬编码 → 大版本删除兼容字段；验收要求「加门派只改 TOML」且莫问数值不变。
本次只写计划，不改实现。

## 继续：前端技能目录收口到后端 bundle + 门控 id 后端解析

- `ComboPage` 不再单独调用 `loadSkillPool`，技能目录直接用 `load_combat_bundle` 返回的 `bundle.skills`
  （已套 combat overlay，目录上的读条/充能等时序信息与时间轴一致）；时间轴请求也只用 `bundle.skills`。
- `SkillCombatDTO` 新增 `jihuoqixue_id`：后端在组装 bundle 时把门控奇穴名称解析成 qixue id，
  前端只做 `selQixue.includes(gateId)`，不再自己拼 id/名称集合。
- 前端测试默认 fixture 改为 mock `loadCombatBundle`；新增后端断言门控 id 解析（冲秋冥）。

### 来自 changes/2026-11-07-163628.md

# 武学时间轴本机改动整理为独立 PR

## 什么与为什么

整理当前本机的战斗时间轴与武学面板改动，接续 #104，避免与其已提交内容重复审查。

- 时序引擎支持独立 DOT、引导分段、提前触发与刷新；补充真实伤害汇总、动作步骤及相关回归。
- 武学面板、秘籍匹配与效果文案、奇穴分组/门控、合法性与累计伤害改由后端 bundle/结果提供，前端按契约渲染。
- 增加所有门派可用的“停手”占位技能，保存秘籍/奇穴草稿选择，改进技能目录和伤害记录展示。
- 记录资源机制全门派通用化计划；具体行为和数据决策见本批另外两份变更日志。

## 关键决策

- 新 PR 以 `feature/timeline-miji-qixue`（#104）为目标，只展示当前本机未提交的增量；#104 合入 dev 后需更新目标并重跑 CI。
- #105 已独立 squash 合入 dev，不在本增量中重复包含；与其组合的前端隔离验证已通过。
- 不改组件版本。清除莫问 combat TOML 文件尾多余空行，满足 diff 检查。
- 现有 CI 仅触发 dev/beta/release PR；本堆叠 PR 的本机检查与后续 dev CI 分别记录，不混作同一验证。
- 修复 `test-gear-dynamic.py` 及其调用的 `test-attributes-dynamic.py` 的测试输入：正式配装数据已不再是 sample，脚本改用现有 synthetic fixture，保留示例标记、合成和目录校验断言；属性脚本同步复制附魔/五彩石配置并替换样本值集标识。

## 验证

本机 `make check-all CARGO_JOBS=2` 全部通过，包含 Rust fmt/clippy、workspace 测试（含 9 项金标准）、dynamic 模式 4 项测试、五个模块库、static/dynamic app 构建以及前端 113/113 与 TypeScript/Vite 构建。保留 Clippy 警告和 Vite 大 chunk 提示。双 cdylib 数值集恢复脚本通过；gear dynamic 脚本修正测试样本后通过（含 4 个虚构门派的属性流程）。构建顺序守卫及 diff 检查通过。先前隔离验证：本机内容叠加 #105，前端 121/121 和 TypeScript/Vite 构建通过；原生 GUI 未实测。

### 来自 changes/2026-11-07-174536.md

# 后端计算边界第一轮修复

## 改动与原因

- 桌面 static / dynamic FFI 时间轴统一从后端存储读取完整技能、奇穴、秘籍、形态和增益定义。展示分组会省略奇穴同组的附加定义，不能将其回传后作为引擎输入。
- 后端校验未知/未建模奇穴、固定重互斥、混选格数和每技能秘籍上限；同一奇穴格的多效果自动完整展开。底层 `calculate_timeline` 仍保留自定义 DTO 的引擎测试接口，桌面/FFI 必须走 `calculate_timeline_from_store`。
- 时间轴输入变化时在 layout effect 中使请求版本失效并清空旧结果，计算失败显示错误和重试入口。
- 新增 PanelStatsDTO / panel_stats（静态、动态两条链路），属性面板会心与破防换算移到 Rust，沿用引擎系数及破防量化。前端只格式化，过期或不可用结果显示占位。
- ComboResultDTO 增加追加真伤合计，前端停止汇总各步真伤。
- 减伤输入 10 按 10% 展示，修复原先放大 100 倍的问题。
- 补全主题变量、提高暗色警告/选中/次要文字对比度；奇穴盘可收起并限制滚动高度，修正技能目录位置文案。

## 设计边界

这是可独立验证的第一轮修复，尚未宣称全项目满足“前端只渲染”：ResultTable 的汇总与基准差值、CockpitCombo 的概率差、前端选择交互规则、类型自动生成及浏览器 mock 拆分仍待后续迁移。新增 panel_stats 需要配套更新 core 动态模块；旧模块不支持时不回退 JS 计算。GUI 实机验证另列，不以单元测试或构建代替。

## 验证

- 新增非法选择、同格效果计数、桌面入口拒绝客户端定义、面板量化/单位回归。
- 新增时间轴已完成结果清理、失败重试、迟到成功隔离回归。
- 执行 make check-all CARGO_JOBS=2；最终结果见本次交付说明。

### 来自 changes/2026-11-07-191200.md

# 击杀率与招式配置器（时间轴）同源

## 目标
旧版击杀率走 `calculate_combo`（独立技能逐个一击、DOT 全结算、无奇穴/秘籍/时序/形态曲风/非法跳过），
与招式配置器的时间轴引擎不一致。本次把击杀率搬进时间轴引擎，旧版 ComboResultDisplay 直接退役。

## 后端（jpcg_combo / jpcg_api）
- 时间轴每个伤害事件增加内部 `DamageDice { g, h, rate, fixed, q }`（Cast/ChannelTick/DotTick），
  保存会心分布；确定性总伤仍用 q，不改变原数值。
- `TimelineInput`/`TimelineRequestDTO` 增加 `samples`、`seed`；`TimelineResultDTO` 增加
  `final_kill_prob`、`kill_prob_curve`。
- `Sim::run` 在事件排序后新增击杀率蒙特卡洛：对同一份事件流掷会心、扣血、结算，
  记录击杀时刻 → 归入对应动作步；累计曲线 (动作序号, 概率)。samples=0 保持纯确定性。
- 击杀语义沿用 `hp_preset`（max_hp/current_hp，否则 target_hp 满血）。
- 回归测试 `kill_probability_matches_deterministic_total`：血量高于总伤→0、低于总伤→1、samples=0→无曲线。

## 前端（jpcg_app）
- `ComboPage` 删除旧 `calculateCombo`/`handleCalculate`/`ComboResultDisplay` 及「计算连招」按钮，
  击杀率改由时间轴实时给出（samples=50000，与其它组件一样自动重算）。
- 新增「击杀率」指标 + 「曲线」按钮 + 悬浮卡片（recharts 击杀概率曲线）。
- mock 时间轴补齐 `final_kill_prob`/`kill_prob_curve`；类型/测试同步。

## 验证
- `cargo test -p jpcg_combo`：38 + 20 全绿；`cargo fmt` 通过。
- 前端 `tsc` + 115 项 vitest + `npm run build` 通过。
- 注意：`cargo test --workspace` 仍有一处**既有失败**（非本次改动）：
  `jpcg_core::host::calc::panel_uses_engine_quantization_and_percentage_units` 的破防期望应为 350
  （floor(333*1024/1000)=340 + floor(1*1024/100)=10），测试却断言 351，期望值 off-by-one。

## 追加：修复既有面板破防测试 off-by-one
- `jpcg_core::host::calc::panel_uses_engine_quantization_and_percentage_units` 期望值 351 与引擎截断语义不符：
  引擎 `guo_pofang_with` 截断 `333*1024/1000 = 340.992 → 340`，加 buff `1% → 10`，正确应为 350。
  已把期望与注释改为 350，`cargo test --workspace` 全绿。

## 追加：驾驶台连招结果也切到时间轴（消除 2w vs 61548 不一致）
- `CockpitCombo` 由旧 `calculateCombo`（独立技能逐个一击）改为 `calculateTimeline`（与排轴器同一请求、samples=50000），
  显示「窗口总伤 · 击杀率」；`App`/`state/cockpit` 的快照与校验从 `ComboResultDTO` 换成 `TimelineResultDTO`。
- 前端 `GearCockpit.test` 改用 `calculateTimeline` mock 与 `loadCombatBundle` mock，115 项通过。

## 追加：退役旧连招引擎的桌面入口
- 删除前端 `api.calculateCombo`、`calculate_combo_cmd` mock、`ComboResultDTO`/`ComboStepResultDTO` TS 类型。
- 删除 Tauri `calculate_combo_cmd`（static/dynamic 两路）与 lib.rs 注册。
- 保留 `jpcg_combo::host::calculate_combo` + `engine.rs` + `jpcg_api::ComboResultDTO`：
  它们仍被配装优化器的 `Objective::KillProbability` 使用，待后续迁移到 `Objective::Timeline`
  （时间轴击杀率）后一起退役。
- `cargo test --workspace` / `cargo fmt` / 前端 tsc + 115 项 vitest + build 全绿。

### 来自 changes/2026-11-07-200500.md

# 排轴器「计算开始/结束」flag（只作用于击杀率）

## 需求
给排轴器加两个标记「计算开始」「计算结束」，各最多一个、独立序列项，用于切分垃圾时间；
只影响击杀率，总伤/DPS 不变。语义 B：把 [开始,结束] 视为独立爆发，开始时刻满血重置（前段伤害不计入）。
flag 时刻对齐「下一个施放的起手」。

## 后端（jpcg_api / jpcg_combo）
- `TimelineActionDTO::Flag { flag }`、`TimelineResultDTO.kill_start / kill_end`。
- 引擎 `TimelineAction::Flag { start }`：不占 GCD、不产生事件；时刻吸附到下一个成功施放的 `start`。
- 击杀率 MC 改为：`kill_start`（缺省 0）/`kill_end`（缺省 window）窗口内、从满血开始独立结算；
  总伤/DPS 统计逻辑不变。
- `calculate_timeline_from_store` 校验：开始/结束各≤1、开始早于结束。

## 前端（jpcg_app）
- `SequenceItem` 联合（技能步 | flag），`ComboDraft.sequence` 改为 `SequenceItem[]`，`comboDraftSchema` v2→v3 迁移。
- 排轴页新增「计算开始/结束」按钮（各最多一个）；flag 项可拖动/删除，渲染为虚线标记卡。
- 动作映射把 flag 项转成 `TimelineActionDTO::Flag`；CockpitCombo 同步；预设只读写技能步（flag 不入预设）。
- 击杀率曲线卡片标注测量窗口（前端按 kill_start/kill_end 显示）。

## 验证
- Rust 回归：`kill_flag_scopes_kill_probability_with_full_hp_reset`（B 语义满血重置、总伤不变）、
  `kill_flag_validation_rejects_duplicates_and_bad_order`（各≤1、顺序合法）。
- `cargo test --workspace` / `cargo fmt` 全绿；前端 `tsc` + 115 项 vitest + `npm run build` 通过。

### 来自 changes/2026-11-07-201500.md

# 配装自动调优（按连招选配装）第一版

## 目标
首页（驾驶台）配装面板「重置配装」左边新增「自动调优」：按当前连招（时间轴）自动搜索更优配装并应用。

## 后端
- `jpcg_gear::Objective` 新增 `Timeline { request: TimelineRequestDTO, metric: TimelineMetric }`；
  `TimelineMetric = total_damage | window_dps | kill_probability`。
- `optimize::score`：克隆时间轴请求、用 `player(composed)` 覆盖玩家属性后调 `jpcg_combo::timeline_host::calculate_timeline`，
  按 metric 取总伤/窗口DPS/击杀率；沿用 `(stats, cooldowns)` 去重缓存。
- `validate_context`：Timeline 校验动作非空；击杀率指标要求 samples>0 且目标有血量。
- `jpcg_gear::host::dispatch` 增加 `optimize`；Tauri 新增 `optimize_gear` 命令（static/dynamic 两路）。

## 前端
- `types/gear.ts` 新增 OptimizeRequest / Objective / TimelineMetric / OptimizeResult 等契约；
  `api.optimizeGear`；`utils/timelineRequest.ts` 抽公共 `buildTimelineRequest`（排轴器/调优共用）。
- `GearPanel` 增加「自动调优」按钮（重置配装左边）；`App.handleOptimize`：按连招组装时间轴请求
  （目标有血量→击杀率，否则→窗口总伤），调 optimize，应用 best 配装并 toast。

## 验证
- `cargo test --workspace` / `cargo fmt` 全绿；前端 `tsc` + 115 项 vitest + `npm run build` 通过。
- 备注：Timeline 目标暂缺独立 Rust 集成测试（依赖 data/gear + 值集），GUI 实机冒烟待桌面验证。

## 追加：自动调优进度百分比
- 问题：调优在后台跑，前端只有「调优中…」一直卡着，没有反馈。
- 后端：Tauri `optimize_gear`（static）直接调 `jpcg_gear::optimize`，用 `on_progress` 按「visited / min(组合数, 预算)」算百分比，
  整数变化才 emit `gear-optimize-progress`（最多 101 次，避免刷屏），结束时补发 100；dynamic 路径无进度（FFI 无法流式回调）。
- 前端：`api.listenGearOptimizeProgress`（处理监听注册竞态）；`App` 加 `optProgress` 状态，调优期间监听进度、
  结束后取消；`GearPanel` 按钮显示「调优中 N%」。
- 验证：cargo fmt/test 全绿；前端 tsc + 115 项测试 + build 通过。

## 追加：全核并行调优 + dev 提速 + 调优约束/目标选项

### A. 全核并行（rayon）
- `jpcg_gear` 新增 `optimize_parallel(catalog, req, &OptimizeProgress)`：
  - 候选按 flat index 用 rayon 并行枚举（默认 = 全部逻辑核；`RAYON_NUM_THREADS` 可覆盖）；
  - 线程本地缓存（`map_init` 的 `HashMap`，key=(stats,cooldowns)）去重、避免锁竞争；
  - `OptimizeProgress { visited/evaluated/total: AtomicU64, cancel/done: AtomicBool }`，主线程轮询发进度事件；
  - best 归并 + `dominates` Pareto（Mutex 小集合）；确定性 tie-break 由 flat index 顺序保证。
- Tauri `optimize_gear`（static）改调并行版：监视线程每 100ms 轮询原子并发 `gear-optimize-progress`，结束后补 100。
- 保留串行 `optimize` 供 CLI/测试/FFI(dynamic) 使用。

### B. dev 构建提速
- 根 `Cargo.toml` 给热路径 crate 加 dev 优化：`jpcg_core` / `jpcg_combo` / `jpcg_gear` `opt-level = 2`
  （debug 默认 opt-0，蒙特卡洛循环会慢 10~50x；app/UI crate 仍 debug）。

### 调优约束
- `Constraints.required_sets: BTreeMap<String, usize>`（必须凑齐的套装件数）；
  `Composed.set_counts: BTreeMap<String, usize>`（实际穿戴件数）；`satisfied()` 校验、`validate()` 校验套装存在与件数范围。
- 前端 GearPanel 新增「调优设置」栏：
  - 目标：击杀率 / 区间 DPS；
  - 必须竞技场四件套（按 catalog 的 mingjian+armor 套装动态取 id，≥4 件）；
  - 加速 ≥ 下限（`stats.jiasu.min`）。
- 「区间 DPS」目标 samples=0（纯确定性、快），「击杀率」samples=50000。

### 验证
- `cargo test --workspace` / `cargo fmt` 全绿；前端 tsc + 115 项 vitest + build 通过。
- 备注：并行版尚缺独立 Rust 集成测试（依赖 data/gear + 值集 + 连招数据），需 GUI 实机冒烟。

## 追加：调优结果常驻面板
- 之前调优完成只有 toast + 最优配装自动应用到配装卡，缺少可回看的输出。
- 现在 `GearPanel` 在「调优设置」下方新增「调优结果」面板（`App` 保存 `optResult`）：
  - 目标指标值（击杀率 % / 区间 DPS）+ 相对当前配装的提升（percentage points / DPS）；
  - 遍历状态（已遍历全部组合 / 预算内最优 / 已取消）与评估数；
  - 12 部位最优选择摘要（来源 · 绿字）；
  - 警告列表。
- 验证：前端 tsc + 115 项 vitest + build 通过。

## 追加：调优纳入附魔/五彩石
- 之前优化器只枚举 12 部位模板，附魔（`loadout.enchants`）与五彩石（`loadout.wucai`）沿用当前配装、从未被选择。
- 现在 `optimize`/`optimize_parallel` 在模板搜索结束后追加「附魔/五彩石坐标下降」：
  - 逐部位比较「各附魔 / 移除」与「各五彩石 / 移除」，取使目标分数最高者，多轮直至不再提升；
  - 旧属性模式（无 attribute_rules）下无法合成的附魔/五彩石（如带五维的五彩石）直接跳过而不是报错；
  - 结果 `best.composed.loadout` 已含最优附魔/五彩石，前端应用后配装卡与合成属性会体现。
- 调优结果面板的 12 部位摘要追加显示「附魔：xxx」与五彩石名称。
- 验证：`cargo test --workspace` / `cargo fmt` 全绿；前端 tsc + 115 项 vitest + build 通过。

## 修复：优化器选到了基础套装（被升级档支配的劣解）
### 根因
- 组合空间 37.7M，预算 100k；旧实现用 odometer 顺序从 0 开始穷举，**前 100k 个组合的前几个槽位一直固定为候选列表的第 0 项**。
- 候选按 `Choice` 排序（`Tier::Base < Tier::Upgraded`），所以基础档永远排在前面 → 预算截断后前几个部位永远选基础档，而基础档属性逐项更低，结果必然更差。

### 修复
- 新增 `build_candidates`：剪掉「被同槽/同来源/同变体的升级档逐项 >= 支配」且升级档本身被允许（`permits`）的基础档，避免劣解进入候选。
- 新增 `combination_indices`：组合数 <= 预算时全遍历；否则用固定种子 splitmix64 **确定性均匀抽样**，避免 odometer 前缀偏差。
- `optimize_with`（串行）与 `optimize_parallel`（全核）都改用上述候选/下标；`complete` 仅在空间真正全覆盖时为真。
- 附带：附魔/五彩石坐标下降改为从当前配装出发、跳过不可合成的单品，避免旧属性模式误报错。

### 验证
- `cargo test --workspace` / `cargo fmt` 全绿（jpcg_gear 既有 18 项测试通过）。
- 建议 GUI 复测：确认调优结果不再出现基础档（升级档属性更高时应选升级档）。

## 修复：调优结果 12 部位摘要被截断成省略号
- 原因：`.optResultSlots` 用 `minmax(150px,1fr)` + `white-space:nowrap; overflow:hidden; text-overflow:ellipsis`，
  窄列下「部位 · 来源 · 绿字 · 附魔 · 五彩石」被截断成 `……`。
- 修复：列最小宽 220px、允许换行（`white-space:normal; overflow-wrap:anywhere`），部位名单独一行；
  每个格子加 `title` 兜底显示完整内容。
- 验证：前端 tsc + 115 项 vitest + build 通过。

### 来自 changes/2026-11-07-220000.md

# 修复：区间 DPS 用固定 300s 窗口做分母

## 问题
- 时间轴结果 `window_dps` 一直是 `total_damage / window`，而 `window` 固定 300 秒（模拟上限）。
- 一次只有几秒的连招（如 6 万伤害）会被 300 秒稀释成 DPS≈200~300，明显错误。

## 修复
- `timeline.rs` 在汇总总伤时同时记录「最后一个伤害事件时刻」`last_damage_time`
  （Cast 用 hit_time，ChannelTick/DotTick 用事件时刻，且仅统计 damage > 0）。
- 区间 DPS 改为 `total_damage / max(last_damage_time, 1.0)`：正常连招即「总伤 ÷ 实际伤害跨度」，
  只有 0/单次瞬发等退化情况才用 1 秒兜底。
- 总伤、击杀率逻辑不变；配装调优的 `TimelineMetric::WindowDps` 自动沿用修正后的 DPS。

## 验证
- 新增 Rust 回归 `window_dps_uses_actual_damage_span`：3 次瞬发（GCD 1.5，末跳 3.0s）断言
  `window_dps ≈ total/3`，而不是 `total/300`。
- `cargo test --workspace` / `cargo fmt` 全绿；前端 tsc + build 通过。

### 来自 changes/2026-11-07-230000.md

# 修复调优输入锁定与时间轴计算边界

- 用户要求：调优期间不可更改输入。整个编辑区设为 inert，并展示全屏进度遮罩；成功、失败均解锁，同步锁防止重复提交。开始时作废先前导入/环境切换请求。
- 抽取 prepare_timeline_from_store，桌面与调优都从完整后端数据准备请求；调优只准备一次，各候选复用冻结定义，避免同组奇穴附加效果丢失。
- 串行/并行优化共用约束和预算校验，限制时间轴动作与采样数量。
- 修复 DOT 起手在 MC 中额外掷伤害；追加真伤复用 core 唯一公式，按期望路径与每条 MC 路径实时扣血结算。
- 多个相邻 flag 全部解析；没有后续施放的末尾 flag 落在当前动作结束时刻，不回退到整个模拟窗口。零长度测量窗口不产生击杀。
- 驾驶台缓存键包含秘籍/奇穴，恢复基准使用相同键。
- 调优工作线程异常时也终止进度监视线程。

验证：新增 DOT、追加真伤、相邻/末尾标记、完整奇穴定义同源、双优化入口非法预算/约束、调优成功/失败解锁回归；make check-all CARGO_JOBS=2 全部通过（fmt、clippy、全量 Rust 测试、模块库、静态/动态桌面构建、前端 117/117 与 TypeScript/Vite）。GUI 原生交互仍需实机验收。

### 来自 changes/2026-11-08-133100.md

# 整合 dev 初始化修复与过期导数保护

- 将已合入 dev 的 #101（独立持久化心法/系数就绪状态）与 #105（输入变化使导数请求失效）同步至战斗时间轴整合分支。
- 解决 App、ConfigPanel、cockpit 状态契约冲突，同时保留模板版本迁移、同步初始化 ref、调优输入锁、秘籍/奇穴缓存键、Rust 属性面板和时间轴结果类型。
- 本机前端 149/149、TypeScript/Vite 构建及 diff 检查通过。此次整合只改变前端及变更记录，Rust 代码未变；远端 CI 将针对整合后提交重跑。

### 来自 changes/2026-11-08-134200.md

# 修复 PR 审阅确认的八项问题

- DOT 每次施加保留独立进攻快照与本轮跳序，刷新保留已结算跳；按时间和动作顺序处理提前跳，避免未来 DOT 被旧事件提前，并停止被刷新实例的后续触发。
- 装备搜索存在属性上限时关闭升级档支配剪枝，保证满足上限的基础档不会被删除，串行与并行一致。
- 驾驶舱存档版本升级至 v3，迁移 v1/v2；保留输入、配装、基准和已兼容时间轴，只丢弃旧形状的连招结果，迁移内嵌草稿。
- 固定奇穴槽切换移除同槽完整 group_ids；浏览器战斗包复用已有模拟技能池。
- 跨门派装备卡与调优摘要按门派和选择键匹配模板。
- 技能添加使用原生按钮，与秘籍按钮并列，支持键盘聚焦和激活。
- 增加 DOT 刷新/先羽后商/商羽商、上限搜索双模式、存档迁移、奇穴切换、模拟技能池及跨门派显示回归。保留调优期间输入锁定。

验证：`cargo test -p jpcg_combo -p jpcg_gear`、前端 155 项测试、TypeScript/Vite 构建通过。此次未执行原生 GUI 人工验证；远端 CI 以推送后的新提交结果为准。

### 来自 changes/2026-11-08-222800.md

# 修复 PR #107 新一轮五项审阅问题

- 时间轴 Cast DTO 携带单步 overrides，Rust 使用每次施放的独立副本应用覆盖值，不修改技能目录或冷却身份。排轴器、驾驶舱和调优共用请求组装，覆盖基础伤害、攻击系数、目标减伤、化劲忽略、额外攻击、郭氏增益、会心及会效。
- 预设在既有 steps 外新增 flags（位置 + 开始/结束），保留完整序列的标记位置，包括相邻和末尾标记。旧文件缺省 flags 为空；Tauri 静态和动态/FFI 链路均支持。
- 显式 charges=1 按 recharge 恢复；charges=0 继续走普通 cooldown。真实剑·羽/疏影横斜验证 16 秒/28 秒间隔。
- 附魔和五彩石纳入模板的联合搜索维度，完整配装合成后才判断属性硬约束。取消模板最优后的贪心附魔阶段，预算、组合总数及完成状态覆盖所有维度；预算耗尽不声称穷尽。搜索空间增大时可能更早耗尽同一预算，结果仍明确为预算内候选。
- 日志按实际命中时刻排序并仅保留窗口内事件，累计值与总伤一致；Cast 的 start_time 单独保留起手时间，界面显示为“结算”，明细展示起手。
- 回归覆盖单步覆盖隔离和冷却身份、桌面/调优一致性、预设往返及旧格式、真实单充能、附魔与五彩石共同满足下限、预算状态、读条/引导跨窗口及读条期间 DOT。
- 调优期间的输入锁定保持不变。未执行原生 GUI 人工验证。

逐步减伤覆盖应用于本动作的目标快照（含 DOT 后续跳）；郭氏增益覆盖技能增益，保留全局 buff。补齐核心无视化劲百分比的实际计算及导数，零值不改变金标准。

最终验证（2026-11-09）：串行执行 `make check-all CARGO_JOBS=2` 全部通过，覆盖 fmt、Clippy、全量 Rust/金标准/文档测试、五个模块库、动态桥接冒烟、静态/动态桌面构建、前端 156/156 及 TypeScript/Vite 构建。此前交错构建的文档测试产物引用错误在串行检查中未再出现。`git diff --check` 通过。

### 来自 changes/2026-11-09-025029.md

# 修复 PR #107 商/角独立 DOT 的逐步属性覆盖

- 每次施放的增益、无视化劲、额外会心与额外会效覆盖，同时应用于本体和独立 DOT 的技能副本，确保后续 DOT 结算使用本次覆盖后的快照。
- 本体基础伤害与攻击系数的覆盖仍只影响本体，不改写独立 DOT 的每跳基础伤害和专属系数；技能目录及随后未覆盖的刷新保持不变。
- 新增真实商/角 combat overlay 回归，分别覆盖四项共享修饰、存活 DOT 刷新后的覆盖隔离，以及本体伤害/系数覆盖不污染 DOT。
- 修复前新增回归失败：商的 gain_override=100 仍得到首跳 84（与无覆盖相同）。修复后该回归与 combo/core/gear 测试通过；未执行原生 GUI 人工验收。

### 来自 changes/2026-11-09-063000.md

# 修复旧配装存档的自动调优入口

## 改动与原因

- `optimizeGear` 与已有合成、导出入口一致，在调用 Tauri 前清理已删除的 `loadout.sockets` 字段。
- 旧持久化配装可通过前端校验并正常合成，但调优原先发送未经清理的对象，触发 Rust `Loadout` 的 `deny_unknown_fields` 反序列化拒绝。
- 复用已有清理函数，不改变装备固定镶嵌、五彩石、附魔、搜索参数或计算规则，不修改调用方存档对象。

## 回归验证

- 新增真实 API 封装层测试，截获三种入口的 Tauri 调用，确认旧字段被清理、合法参数完整保留且输入不被修改。
- 先运行新测试，原实现仅调优 payload 多出 `sockets` 而失败；应用单行修复后重跑。
- 验证范围与完整检查结果见本 PR 说明。组件与数据版本不变。

## [Unreleased]

### 新增
-（下次发布的变更将列于此）

## [2.1.0-alpha.1] - 2026-06

（历史版本记录待首次正式发布时从 git log / changes/ 补充）
