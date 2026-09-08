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

## [Unreleased]

### 新增
-（下次发布的变更将列于此）

## [2.1.0-alpha.1] - 2026-06

（历史版本记录待首次正式发布时从 git log / changes/ 补充）
