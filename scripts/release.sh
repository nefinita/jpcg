#!/usr/bin/env bash
# ============================================================================
# release.sh — 发布流程（在 release/vX.Y.Z 分支上执行）
# 1. 校验工作树干净
# 2. 全量测试（含金标准、双模式、前端类型检查）
# 3. bump 版本（cargo set-version + cargo-workspace-version + sync-version.sh）
# 4. 聚合 changes/ 到 CHANGELOG.md（Unreleased → 版本段）
# 5. commit + tag v<ver> + push
#
# 用法：
#   git flow release start vX.Y.Z     # 或 git checkout -b release/vX.Y.Z develop
#   scripts/release.sh X.Y.Z          # X.Y.Z 为本次发布版本（core / 安装包命名源）
# ============================================================================
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="${1:?用法: scripts/release.sh <版本号> e.g. 2.1.0}"

echo "==> [1/6] 校验工作树干净"
if [ -n "$(git status --porcelain)" ]; then
  echo "错误：工作树不干净，请先提交或 stash。" >&2
  git status --porcelain
  exit 1
fi

echo "==> [2/6] 全量测试"
make check-all
cargo test -p jpcg_core -- golden

echo "==> [3/6] bump 版本 -> $VERSION"
# 根 workspace version 为权威源；继承 crates（api/core/update/app/forum/manifest-gen）
# 经 version.workspace = true 自动跟随。const（130.3.date）与 updater（独立）不被动。
cargo set-version "$VERSION"
scripts/sync-version.sh "$VERSION"

echo "==> [4/6] 聚合 CHANGELOG"
python3 - "$VERSION" <<'PY'
import re,sys,glob,os,datetime
ver=sys.argv[1]
# 收集 changes/*.md 内容
entries=[]
for f in sorted(glob.glob('changes/*.md')):
    entries.append(f"### 来自 {f}\n\n"+open(f).read().strip())
body="\n\n".join(entries) if entries else "（无逐条变更日志）"
today=datetime.date.today().isoformat()
section=f"\n## [{ver}] - {today}\n\n{body}\n"
text=open('CHANGELOG.md').read()
# 把新版本段插到 [Unreleased] 之后
idx=text.find('\n## [Unreleased]')
end=text.find('\n## ', idx+1)
if end==-1: end=len(text)
unreleased=text[idx:end]
# 清空 Unreleased 的变更（已并入版本段），保留标题
new_unreleased="## [Unreleased]\n\n### 新增\n-（下次发布的变更将列于此）\n"
text=text[:idx]+section+"\n"+new_unreleased+text[end:]
open('CHANGELOG.md','w').write(text)
print(f"    已更新 CHANGELOG.md -> [{ver}]")
PY

echo "==> [5/6] commit + tag"
git add -A
git commit -q -m "release: v${VERSION}

版本 ${VERSION} 发布准备（CHANGELOG 聚合 + 版本同步）"
git tag -a "v${VERSION}" -m "JPCG v${VERSION}"

echo "==> [6/6] push"
git push origin "$(git branch --show-current)"
git push origin "v${VERSION}"

echo "✅ 发布完成: v${VERSION}（已推 tag，CI release.yml 将自动构建并上传 GitHub Release）"
