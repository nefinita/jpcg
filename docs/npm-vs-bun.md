# npm vs bun：JPCG 前端包管理器对比与迁移评估

> 状态：调研报告（未改默认工具链）。日期：2026-10-20。
> 适用对象：JPCG 仓库 `examples/jpcg_app`（Tauri v2 + React 19 + TypeScript + Vite + Vitest）。
> 结论先行：**短期维持 npm（`npm ci`）**；如需探索，建议以**非阻塞试点**方式验证 bun，
> 满足本文章末「试点检查清单」全部条件后再考虑迁移。

## TL;DR

| 维度 | npm | bun | 对 JPCG 的影响 |
|------|-----|-----|----------------|
| 冷装（本机实测，空缓存） | 8.8 s / 13.5 s（两次） | 12.3 s / 9.1 s（两次） | 同量级；小项目上未复现「快 7×」的营销数字，网络是主要变量 |
| 暖装（缓存命中，删 node_modules） | 2.64 s（`npm ci` 每次全量重装） | 0.14 s / 0.08 s（`bun ci` 0.15 s） | bun 明显更快，但绝对值只有秒级 |
| 构建 / 测试 | 5.84 s / 2.79 s（24 passed） | 5.45 s / 2.89 s（24 passed） | 等价，均通过 |
| 体积 | node_modules 165 MB（9,723 文件） | node_modules 165 MB（9,722 文件） | 基本一致 |
| 缓存 | 41 MB（压缩 tarball） | 165 MB（解包副本，可硬链接复用） | bun 缓存更大；同盘硬链接时额外占用很小 |
| 锁文件 | `package-lock.json` 119,763 B | `bun.lock` 69,872 B（自动迁移生成） | bun 锁更紧凑，但引入双锁管理风险 |
| Tauri v2 CLI | `npx tauri` 2.11.2 | `bun x tauri` 2.11.2（实测可用） | CLI 本身是 Rust 二进制，风险低 |
| 安全默认值 | 依赖生命周期脚本**默认执行** | 依赖生命周期脚本**默认阻止**（需 `trustedDependencies`） | bun 默认更安全，但需维护可信清单 |
| 审计 | `npm audit` 退出码 1（有漏洞） | `bun audit` 有漏洞时**实测退出码 0** | 迁 CI 门禁需显式解析，不能只看退出码 |
| Dependabot | version + security updates 完整支持 | version updates GA；**security updates 尚未支持** | 当前维持 npm 的决定性因素之一 |
| 迁移成本 | — | 中高（CI/Makefile/Tauri 脚本/文档/Dependabot 均需改） | 收益主要在本地反复安装，CI 收益被 Rust 构建时长淹没 |

## 1. 测试环境与方法

| 项 | 值 |
|----|----|
| 机器 | Linux x86_64 容器（本机），单机单次实验 |
| Node / npm | v22.22.0 / 10.9.4 |
| bun | 1.4.2（`/home/qin13/.bun/bin/bun`） |
| 项目 | `examples/jpcg_app`：247 个 lock 条目、实际安装 188 个包、1 个 vitest 测试文件（24 用例） |
| 隔离方式 | 复制整个 app 到工作区 `.scratch/meta-dev/bench/{npm-app,bun-app}`，**不触碰主工作区 `node_modules`** |
| 缓存 | `$HOME` 只读，npm 用 `--cache`、bun 用 `BUN_INSTALL_CACHE_DIR` 指向工作区；每个样本使用独立空缓存目录 |
| 计时 | `date +%s%N` 前后取差；每命令 1–2 个样本 |

方法说明：

- `npm` 侧用 **`npm ci`**（CI 实际命令，语义是「删除 node_modules 后按 lock 全量重装」）。
- `bun` 侧先验证「无 `bun.lock`、只有 `package-lock.json`」时 `bun install --frozen-lockfile` 的行为，
  再在生成 `bun.lock` 后测 `bun install --frozen-lockfile` 与 `bun ci`。
- 依赖是否被升级：安装完成后逐一读取两侧 `node_modules/**/package.json` 的 `name→version` 做全量对比，
  **185 个顶层/作用域包版本完全一致，无新增/缺失包**——即 bun 迁移不会改变解析结果。
- 构建/测试：两侧分别执行 `npm run build`/`npm test` 与 `bun run build`/`bun run test` 并计时。

## 2. 安装速度与磁盘（实测）

### 2.1 耗时

| 场景 | npm | bun |
|------|-----|-----|
| 冷装样本 1（空缓存） | 8.77 s | 12.31 s（`--frozen-lockfile`，bun.lock 已存在） |
| 冷装样本 2（空缓存） | 13.52 s | 9.11 s |
| 暖缓存重装（删 `node_modules`，缓存已有） | 2.64 s（`npm ci`） | 0.14 s / 0.08 s（`--frozen-lockfile`）；`bun ci` 0.15 s |
| 首次迁移（无 `bun.lock`，从 `package-lock.json` 迁移，冷缓存） | — | 9.51 s |
| 构建 | 5.84 s | 5.45 s |
| 测试（vitest 24 用例） | 2.79 s | 2.89 s |
| Tauri CLI | `npx tauri --version` → 2.11.2 | `bun x tauri --version` → 2.11.2 |

解读：

- **冷装两者同量级**（8.8–13.5 s vs 9.1–12.3 s），本机网络波动大于包管理器差异。
  bun 官方宣称的「比 npm 快 ~7×」（[Bun 官方博客](https://bun.com/blog/behind-the-scenes-of-bun-install)）主要来自
  `bun install` 对 `npm install` 的系统调用/解包优化，在「小而依赖不多」的项目 + 冷缓存场景收益有限；
  JPCG 前端仅 188 个包，属于小项目。
- **暖缓存差距巨大**：`npm ci` 每次都会删除并全量重装 `node_modules`（2.6 s），
  而 bun 从缓存硬链接/复用（0.1 s 级）。这对**本地频繁切分支/重装**的开发体验有意义。
- 构建、测试耗时几乎相同；bun 运行 vitest 也通过（但见 §4 的「非官方支持」说明）。

### 2.2 磁盘

| 项 | npm | bun |
|----|-----|-----|
| `node_modules` 表观大小 | 144,080,973 B（137.4 MiB） | 143,999,561 B（137.3 MiB） |
| `node_modules` `du`（块占用） | 165 MB | 165 MB |
| 文件数 | 9,723 | 9,722 |
| 包管理器缓存 | 41 MB（压缩 tarball） | 165 MB（解包后的包副本，9,722 文件） |
| lockfile | `package-lock.json` 119,763 B | `bun.lock` 69,872 B |

- `node_modules` 的实际内容基本相同（bun 会略去一个 `.bin` 差异项）。
- bun 缓存更「重」：约等于一份解包后的 `node_modules`。**同文件系统**上 bun 用**硬链接**把缓存接到
  `node_modules`（实测 inode `nlink=2`），所以实测额外占用只有 ~1 MB；若缓存放在默认的 `~/.bun`
  （与工作区不同文件系统，无法硬链接）则会**复制**，峰值磁盘约为 `node_modules` 的 2 倍。
  JPCG 的 CI 与只读 `$HOME` 环境必须把缓存放工作区（见 §5）。
- npm 缓存是压缩 tarball（41 MB），重装时解包成真实文件；`npm ci` 的「全量删除 + 重装」也带来更多磁盘 I/O。

## 3. 锁文件与 workspace

### 3.1 bun 的锁文件与自动迁移

- bun 使用文本锁文件 `bun.lock`（v1.2 起默认），体积约为 `package-lock.json` 的 58%。
- 项目只有 `package-lock.json` 时，`bun install` 会**自动迁移** npm 锁并生成 `bun.lock`
  （[Bun Lockfile 文档](https://bun.sh/docs/pm/lockfile)）。支持 npm lockfileVersion 2/3/4，
  迁移后**保留原 `package-lock.json` 不动**。
- **实测注意事项（bun 1.4.2）**：在只有 `package-lock.json`、没有 `bun.lock` 时执行
  `bun install --frozen-lockfile`，bun **仍然写入了 `bun.lock`**（输出 `migrated lockfile ... Saved lockfile`），
  退出码 0。这与当前文档「`--frozen-lockfile` 迁移 npm 锁时不写文件、只打印 note」的描述不一致
  （文档可能对应更高版本）。若在 CI 里用 `--frozen-lockfile` 守卫，必须额外用 `git diff --exit-code`
  检查工作树没有被写入 `bun.lock`，否则「冻结」会名不副实。
- 迁移后 `bun.lock` 成为事实锁；若两把锁同时存在，**bun 优先 `bun.lock`**，npm 继续用 `package-lock.json`，
  极易漂移。**必须二选一**。
- `bun.lock` **不记录根 package 的 `version` 字段**（只记录 `workspaces{}` 的依赖范围与解析结果），
  因此若未来迁移，`scripts/sync-version.sh` 中「同步 package-lock 根 version」这一步可以直接移除，
  不需要为 `bun.lock` 编写版本同步逻辑。

### 3.2 workspace

- JPCG 前端是**单包**（`examples/jpcg_app`，`private: true`），没有 npm workspaces；
  Rust 侧是 cargo workspace，与 JS 包管理器无关。
- bun 支持 `workspaces`/`catalogs`/`--filter`（[Bun install 文档](https://bun.sh/docs/pm/cli/install)），
  但对当前的单包结构**没有收益**，反而多一层认知成本。
- 当前 `package-lock.json` 与 `package.json` 的依赖范围严格一致（本次 A 段已校验），迁移不会有解析漂移。

## 4. Node 兼容与原生模块 / Tauri v2 + vitest 生态

### 4.1 依赖里真正「有原生/脚本需求」的部分

`package-lock.json` 中 63 个平台专用可选依赖，关键为：

- `@tauri-apps/cli-{linux,darwin,win32}-*`：Tauri CLI 本体（Rust 二进制，按平台分发，无编译）。
- `@esbuild/*`：esbuild 平台二进制（通过可选依赖分发；`esbuild` 自身带 `postinstall`）。
- `@rollup/rollup-*`：Vite/Rollup 的平台二进制（可选依赖）。
- `fsevents`：macOS 专用可选依赖（带 `install` 脚本）。
- `vitest`/`tinypool`：纯 JS，但依赖 Node `worker_threads` 做 worker 池。

### 4.2 Node 兼容

- bun 声称对 Node API 高兼容，并用 Node 官方测试套件自测；官方兼容表中 `node:fs`/`node:path`/`node:stream` 等为
  🟢、`node:worker_threads`/`node:inspector`/`node:v8` 等为 🟡 部分实现
  （[Bun Node.js Compatibility](https://bun.sh/docs/runtime/nodejs-compat)）。
- 本项目实测：`bun run build`（tsc + vite）与 `bun run test`（vitest）均通过，24/24 用例。
  但注意 `bun run` 会用 bun 运行时执行 `node_modules/.bin` 里的工具（不是 node），
  这等于**把构建/测试工具链也切到 bun 运行时**；vitest 官方并未把 bun 列为受支持运行时
  （[Vitest 文档](https://vitest.dev/guide/)），属于「可用但无官方保证」，升级 vitest 时需回归。
- `bun test` 是另一套测试运行器，与 vitest API 不兼容；本项目测试文件直接 `import { describe, it, vi } from "vitest"`
  并使用 `vi.mock`，迁移到 `bun test` 需要改写测试，不在本次讨论范围。

### 4.3 Tauri v2

- Tauri CLI 是 Rust 二进制，通过 `@tauri-apps/cli` 的**平台可选依赖**分发；
  实测 `bun x tauri --version` 正常（2.11.2），所以 bun 作为「包管理器/运行器」对 Tauri 构建链风险低。
- 但 Tauri 官方文档与模板默认 npm/pnpm/yarn（[Tauri 前置依赖](https://v2.tauri.app/start/prerequisites/)、
  [创建项目](https://v2.tauri.app/start/create-project/)）；JPCG 的 `tauri.conf.json` 还把
  `beforeDevCommand`/`beforeBuildCommand` 硬编码为 `npm run dev` / `npm run build`。
  **若只迁移安装而保留 npm 跑脚本，就是双工具链**；若要统一，必须修改 `tauri.conf.json`
  （属构建配置写作用域，需单独评审）。
- Tauri 的 `bundle`、`externalBin`（`jpcg_updater`）、三平台矩阵与包管理器无关，不受影响。

## 5. CI 与缓存

当前 JPCG CI（`.github/workflows/ci.yml`）两个前端相关 job 均使用：

```yaml
- uses: actions/setup-node@v4
  with:
    node-version: 20
    cache: npm
    cache-dependency-path: examples/jpcg_app/package-lock.json
- run: npm ci
```

本地 `Makefile` 的 `setup` 也用 `npm ci`，缓存指向工作区 `.cache/npm`（规避只读 `$HOME`）。

| 项 | 维持 npm | 切 bun |
|----|----------|--------|
| 安装命令 | `npm ci`（语义固定：删 node_modules 全量重装） | `bun ci`（1.4.2 已提供，等价 `--frozen-lockfile`）或 `bun install --frozen-lockfile` |
| 缓存 action | `actions/setup-node` 内置 `cache: npm` | `oven-sh/setup-bun` + `actions/cache` 缓存 `~/.bun/install/cache`（官方指南：[Bun CI 指南](https://bun.sh/guides/install/cicd)） |
| 缓存 key | 基于 `package-lock.json` 哈希 | 需 `runner.os + bun.lock` 哈希（不同平台可选二进制不同；跨 OS 共享缓存不安全） |
| 缓存体积 | ~41 MB（压缩） | ~165 MB（解包；Actions 缓存上限内，但上传/下载更慢） |
| 单次节省 | — | 前端 job 约省 2–3 s（冷装几乎无差） |
| JPCG 实际收益 | Rust 构建/测试是分钟级，前端安装秒级 → **收益被淹没** | 同左 |

结论：**CI 不是迁移的强动机**。JPCG 的 CI 时间大头在 cargo 构建与金标准测试，前端安装快 2 秒对总时长与排队无感。
真正收益在开发者本地反复 `node_modules` 重建（bun 暖装 0.1 s 级 vs npm 2.6 s），但 `Makefile setup`
已有 stamp 跳过机制，实际重建频率不高。

## 6. 安全（audit / 生命周期脚本）

### 6.1 生命周期脚本

- npm 默认执行所有依赖的 `preinstall`/`postinstall` 等脚本（[npm scripts 文档](https://docs.npmjs.com/cli/v10/using-npm/scripts)），
  这是供应链攻击的常见入口；`npm ci --ignore-scripts` 可关闭但需全项目统一。
- bun **默认不执行**依赖生命周期脚本：「default-secure」，只有 `package.json.trustedDependencies`
  白名单（及内置默认可信列表）中的包才执行（[Bun Lifecycle scripts](https://bun.sh/docs/pm/lifecycle)、
  [trustedDependencies 指南](https://bun.com/guides/install/trusted)）。
- 实测：bun 安装时 **blocked 1 postinstall（`esbuild`）**；由于 esbuild 平台二进制来自可选依赖，
  构建/测试仍全部通过。若未来有包必须跑 postinstall，需要显式加入 `trustedDependencies`
  （注意该字段是「替换」而非「追加」默认列表）。这是 bun 的**安全优势**，但也是**维护负担**。

### 6.2 审计

实测同一份依赖（npm 10.9.4 / bun 1.4.2，各运行一次）：

| 工具 | 结果 | 退出码 |
|------|------|--------|
| `npm audit --json` | 8 个漏洞（2 critical / 4 high / 2 moderate） | **1**（可直接用作 CI 门禁） |
| `bun audit` | 12 个漏洞（2 critical / 6 high / 4 moderate） | **0**（有漏洞仍返回 0） |

- 两者都能发现 vitest/tinypool 等 critical 问题；bun 报告数量更多（advisory 数据库/聚合口径不同）。
- **bun audit 的退出码语义与 npm 不同**：本次实测有 12 个漏洞时仍 `exit 0`。若迁移，
  `.github/workflows/deps.yml` 的门禁必须改为解析 `--json` 输出（或显式判断漏洞数），不能只依赖退出码。
- 依赖升级自动化：GitHub Dependabot 的 **bun version updates 已 GA**，但官方公告明确
  「bun **security updates** 将在未来加入」（[GitHub Changelog, 2025-02-13](https://github.blog/changelog/2025-02-13-dependabot-version-updates-now-support-the-bun-package-manager-ga/)）。
  JPCG 当前 `dependabot.yml` 使用 npm 生态 + security updates；切 bun 会**降低安全更新覆盖**，
  除非同时保留 npm 锁（则回到双锁漂移问题）。

## 7. 三平台成熟度

- bun 提供 macOS / Linux / Windows 单文件可执行安装（[Bun 安装文档](https://bun.sh/docs/installation)），
  三平台都有官方 CI action 可用；Windows 支持已可用，但生态/边缘 Node API 差异在 Windows 上通常暴露更多
  （Bun 兼容表中有若干 🟡 项，如 `worker_threads`、`inspector`）。
- **本次实验只在 Linux x86_64 上完成**；JPCG 发布矩阵是 Linux/macOS/Windows 三平台，
  且 frontend 构建产物（`dist/`）由 Tauri 编译期嵌入，三平台必须一致。
- 若试点，应至少在 CI 矩阵（ubuntu-latest / macos-latest / windows-latest）跑通
  `bun ci && bun run build && bun run test`，再谈迁移；仅本机 Linux 证据不足。

## 8. 迁移成本

若从 npm 切到 bun，需要同时改动（按当前仓库实际文件）：

| 文件/位置 | 改动 |
|-----------|------|
| `examples/jpcg_app/bun.lock` | 新提交（替换 `package-lock.json`） |
| `examples/jpcg_app/package.json` | 可选加 `packageManager: "bun@x.y.z"`；可能需要 `trustedDependencies` |
| `.github/workflows/ci.yml` | 两个前端 job：setup-node+cache npm → setup-bun + 缓存 `~/.bun/install/cache` + `bun ci` |
| `.github/workflows/release.yml` | 前端构建步骤同样切换 |
| `Makefile` | `setup`/`frontend`/`frontend-test`：`npm ci` → `bun ci`；`NPM_CACHE` → `BUN_INSTALL_CACHE_DIR`（只读 `$HOME` 环境必须） |
| `examples/jpcg_app/src-tauri/tauri.conf.json` | `beforeDevCommand`/`beforeBuildCommand` 若统一 bun 需改为 `bun run dev/build` |
| `scripts/sync-version.sh` | 移除 package-lock 版本同步（bun.lock 不含根 version），或保持 npm 双轨时保留 |
| `.github/dependabot.yml` | `package-ecosystem: "npm"` → `"bun"`（security updates 覆盖下降，见 §6） |
| `.github/workflows/deps.yml` | `npm audit` → `bun audit` + JSON 解析门禁（退出码不可靠） |
| 文档 | `AGENTS.md`/`CONTRIBUTING.md`/`README.md`/`examples/jpcg_app/README.md` 中所有 `npm install`/`npx tauri` 说明 |
| 开发者机器 | 统一 bun 版本（`setup-bun` 固定版本）；Windows/macOS 开发者需要安装 bun |

**成本评估：中高。** 改动面覆盖 CI、发布、构建脚本、文档与依赖自动化；
而收益主要是本地暖装 2.5 s → 0.1 s 与 bun 默认更安全，CI 总时长收益可忽略。

## 9. 结合 JPCG 的建议

### 9.1 结论

1. **短期（当前发布线）维持 npm**：
   - CI 省时几乎为零（Rust 构建主导），代价却是改 CI/发布/Makefile/文档/Dependabot 全链路；
   - Dependabot **security updates 尚不支持 bun**，而 JPCG 是公开仓库、有 `deps.yml` 安全门禁；
   - `bun audit` 退出码语义不适合现成门禁（本次实测有漏洞仍 exit 0）；
   - Tauri 官方模板与 `tauri.conf.json` 硬编码的 `npm run build` 都默认 npm，双工具链风险高；
   - 现有 `Makefile setup` + `NPM_CACHE` 已解决只读 `$HOME` 的缓存问题。
2. **可选试点 bun（不阻塞主线）**：
   - 在独立分支或一个**非必需**的 CI job 里跑 `bun ci && bun run build && bun run test`（三平台矩阵），
     收集实际 CI 时长与稳定性数据；不修改默认 `npm ci` 路径。
   - 试点门槛（任一项不满足就不迁移）：三平台全绿、`bun audit` 门禁可行、Dependabot security updates
     （或替代方案）可用、`tauri.conf.json` 的 before 命令方案确定、团队接受 bun 版本锁定。
3. **不推荐**现在全量迁移：收益/风险比不划算，且以上任一项失败都会影响发布。

### 9.2 试点检查清单（若决定试点/迁移）

- [ ] 固定 bun 版本：`packageManager: "bun@<ver>"` + CI `oven-sh/setup-bun@v2` 指定同版本。
- [ ] `bun install` 生成 `bun.lock`；**只提交一把锁**（删除 `package-lock.json` 或明确仅保留 npm 为主、bun 分支不合并）。
- [ ] 依赖树一致性验证：脚本对比两侧 `node_modules/**/package.json` 的 `name→version`（本次方法可直接复用）。
- [ ] 生命周期脚本审计：确认被 block 的 `esbuild` 无需 postinstall；若需要，显式维护 `trustedDependencies`
      并理解「该字段替换默认可信列表」的语义。
- [ ] CI：`bun ci`（或 `bun install --frozen-lockfile`）+ `git diff --exit-code` 守卫（bun 1.4.2 在迁移场景会写 `bun.lock`）。
- [ ] CI 缓存：`actions/cache` 缓存 `~/.bun/install/cache`，key 含 `runner.os` 与 `bun.lock` 哈希；
      只读 `$HOME` 环境用 `BUN_INSTALL_CACHE_DIR` 指向工作区。
- [ ] `tauri.conf.json`：`beforeDevCommand`/`beforeBuildCommand` 与包管理器一致（避免 npm/bun 混用）。
- [ ] `Makefile`：`setup` 改 `bun ci`，`NPM_CACHE` 改为 bun 缓存变量；`FORCE_SETUP` 语义保留。
- [ ] `scripts/sync-version.sh`：移除/调整 package-lock 同步步骤（`bun.lock` 无根 version）。
- [ ] `dependabot.yml`：`package-ecosystem: bun`（注意 security updates 缺口，必要时保留 npm 生态或人工补位）。
- [ ] `deps.yml`：`bun audit` 的 JSON 解析 + 明确失败阈值；不要依赖退出码。
- [ ] 三平台 CI 矩阵（ubuntu/macos/windows）全部通过 `bun ci + build + test`。
- [ ] 文档全量更新（AGENTS/CONTRIBUTING/README/app README/Makefile help）。
- [ ] 回滚方案：保留 tag/分支可一键恢复 `package-lock.json` + `npm ci`；首个 beta 发布周期观察后再收敛。

## 10. 实测局限

- 单机、Linux x86_64、网络可达 npm registry；冷装耗时受网络波动影响大（npm 两次 8.8/13.5 s），
  样本量小（每场景 1–2 次），不代表大型 monorepo 或弱网环境。
- 未在 macOS/Windows 实测；bun Windows/macOS 的构建与缓存行为需在 CI 矩阵复核。
- bun 版本 1.4.2（实测样本）：`--frozen-lockfile` 迁移时**写入 `bun.lock`** 的行为、`bun ci`、
  `bun audit` 有漏洞仍 `exit 0` 的行为均为**单次实测样本**；不同 bun 版本/未来文档口径可能变化，
  迁移决策前必须在目标版本上复测，不应把本文数字当作长期结论。
- `npm audit` / `bun audit` 的漏洞计数（8 vs 12）与退出码是同一 lock、同一 registry 时点的单次快照，
  仅用于说明「两工具门禁语义不同」，不代表哪个数据库更全/更准。
- 磁盘对比为单机同一文件系统；bun 缓存跨文件系统时的复制行为为推断（依据「同 FS 硬链接、跨 FS 无法硬链接」）。
- 测试文件为 1 个（24 用例），vitest 在 bun 下通过不代表所有 React/jsdom 场景无差异。

## 11. 参考来源

- Bun 官方博客：*Behind The Scenes of Bun Install* — <https://bun.com/blog/behind-the-scenes-of-bun-install>
- Bun 文档：*bun install* — <https://bun.sh/docs/pm/cli/install>
- Bun 文档：*Lockfile*（自动迁移 npm/yarn/pnpm 锁、frozen-lockfile 行为）— <https://bun.sh/docs/pm/lockfile>
- Bun 文档：*Lifecycle scripts* — <https://bun.sh/docs/pm/lifecycle>
- Bun 指南：*Add a trusted dependency* — <https://bun.com/guides/install/trusted>
- Bun 文档：*Node.js Compatibility* — <https://bun.sh/docs/runtime/nodejs-compat>
- Bun 文档：*Installation* — <https://bun.sh/docs/installation>
- Bun 指南：*Install dependencies in GitHub Actions* — <https://bun.sh/guides/install/cicd>
- npm 文档：*npm ci* — <https://docs.npmjs.com/cli/v10/commands/npm-ci>
- npm 文档：*npm scripts / lifecycle* — <https://docs.npmjs.com/cli/v10/using-npm/scripts>
- Tauri v2 文档：*Prerequisites* — <https://v2.tauri.app/start/prerequisites/>
- Tauri v2 文档：*Create a Project* — <https://v2.tauri.app/start/create-project/>
- Vitest 文档：*Guide* — <https://vitest.dev/guide/>
- GitHub Changelog：*Dependabot version updates now support the bun package manager [GA]*（2025-02-13）— <https://github.blog/changelog/2025-02-13-dependabot-version-updates-now-support-the-bun-package-manager-ga/>

> 上述外部内容仅作为调研数据引用；实测数字以本文 §1–§2 的本机实验为准。
