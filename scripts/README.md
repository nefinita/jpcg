# scripts/ — 发布与版本同步脚本

本目录包含项目发布/版本管理的自动化脚本。全部为普通 shell 脚本，**无需特殊系统权限**，
只在你显式调用时在当前 git 仓库内执行。

## 文件

| 脚本 | 作用 |
|------|------|
| `release.sh` | 三分支发布脚本（`alpha`/`beta`/`release`）：测试 → bump 版本 → 聚合 CHANGELOG → commit + tag → 推 prep 分支并开 PR → 推 tag |
| `sync-version.sh` | 读取 workspace 版本，同步到 package.json / tauri.conf.json / commands.ts 模拟串 / package-lock.json |

---

## release.sh 操作说明

### 模型（三分支线性）

```
dev      (X.Y.Z-alpha.n, 不 tag)   ← 所有 feature 合入
beta     (X.Y.Z-beta.n,  tag vX.Y.Z-beta.n)   ← 公测
release  (X.Y.Z,          tag vX.Y.Z)          ← 稳定
```

> `X.Y.Z` 以根 `Cargo.toml` 的 workspace version 为准（当前稳定版 v2.1.0）。

### 用法

```sh
scripts/release.sh <stage> [版本号]
# stage = alpha | beta | release
```

- `stage` 必填；`版本号` 可选（缺省从当前 workspace 版本按 stage 规整推断）

### 各阶段操作步骤

**alpha（dev，日常集成，不 tag）**
```sh
git checkout dev && git pull origin dev
scripts/release.sh alpha
# 仅 bump alpha.n + 聚合 CHANGELOG + commit → 推 prep/<版本> 并开 PR 到 dev
```

**beta（公测）**
```sh
git checkout beta && git pull origin beta
scripts/release.sh beta            # → 打 beta tag（vX.Y.Z-beta.n），触发 beta 通道构建
```

**release（稳定）**
```sh
git checkout release && git pull origin release
scripts/release.sh release         # → tag v2.1.0，触发 stable 通道构建
```

### 脚本内部 8 步

1. **校验**：工作树干净；所在分支与 stage 匹配（alpha→dev、beta→beta、release→release）；需已安装并登录 `gh`
2. **同步**：确保基于最新目标分支（ff-only）
3. **全量测试**：`make check-all` + 金标准回归
4. **bump 版本**：仅改根 workspace 版本（继承者自动生效；`jpcg_const` / `jpcg_updater` 等独立版本号不动）+ `sync-version.sh` + 刷新 lock
5. **聚合 CHANGELOG**：把 `changes/*.md` 并入 `CHANGELOG.md`
6. **commit + tag**：alpha 不 tag；beta/release 打对应 tag
7. **推 prep 合并分支 + 开 PR**：推 `prep/<ver>`，`gh pr create` 到目标分支
8. **推 tag**（beta/release）：推 tag 触发 release.yml

### 安全性说明（可放心分发）

- **无特殊权限**：普通脚本，仅操作当前仓库，不需要 sudo/系统目录
- **防呆**：缺 `stage` 退出；分支不匹配退出；工作树不干净退出；缺 `gh` 退出 —— 不会误提交
- **只显式触发**：不会被自动调用
- **不会乱提交**：所有改动先经 git add + 明确 commit，且仅在满足上述校验后
- **不直接 push 受保护分支**：改动提交到 prep 合并分支并开 PR，由 review 合并落地

### ⚠️ 受保护分支的处理方式

`dev`/`beta`/`release` 均已开启**分支保护**（仓库已转 Public）：

- 禁直接 push / 禁强推 / 禁删除；需 PR + 至少 1 个 approve review（CODEOWNERS 生效）
- 3 个必需状态检查：`Rust 构建 + 测试 + lint`、`金标准回归（计算数值）`、`前端构建 (tsc + vite)`
- `linear history` + conversation resolution；`beta`/`release` 额外 strict，`release` 额外 require last push approval

因此脚本**不直接 push 目标分支**，改为：

1. 在本地对目标分支做 commit + tag
2. 把该提交推到 prep 分支：`prep/<版本>`
3. 用 `gh` 开 PR：`prep/<版本>` → 目标分支
4. 推 tag（beta/release）触发 release.yml 构建

由 reviewer 批准并 **squash 合并** PR 后，版本提交落地目标分支。合并后可删除 prep 分支：
`git push origin --delete prep/<版本>`

### 首次/常规发布速查

```sh
# 1. 日常：feature 合入 dev（经 PR）
# 2. 公测：
git checkout beta && git pull origin beta
scripts/release.sh beta              # 打 vX.Y.Z-beta.n → beta 通道
# 3. 稳定版：
git checkout release && git pull origin release
scripts/release.sh release           # 打 v2.1.0 → stable 通道
# 4. hotfix：从 release 切，修后向前传播 release→beta→dev
```

---

## sync-version.sh

读取根 workspace 版本（core/tag 源），同步到：
- `examples/jpcg_app/package.json`
- `examples/jpcg_app/src-tauri/tauri.conf.json`
- `examples/jpcg_app/src/api/commands.ts`（模拟版本串 `v<ver>`）
- `examples/jpcg_app/package-lock.json`（根 `version` 与 `packages[""].version`；优先 `npm install --package-lock-only`，失败回退最小编辑）

```sh
scripts/sync-version.sh            # 缺省读根 Cargo.toml
scripts/sync-version.sh 2.1.0      # 显式指定

# $HOME 只读环境（如本机沙箱/CI）可指定 npm 缓存：
NPM_CACHE=./.scratch/npm-cache scripts/sync-version.sh 2.1.0
```
