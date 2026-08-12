#!/usr/bin/env bash
# ============================================================================
# release.sh — 三分支发布脚本（dev → beta → release）
#
# 用法：
#   scripts/release.sh <stage> [版本号]
#     stage   = alpha | beta | release
#
# 模型：
#   dev    （alpha.n，不 tag）
#   beta   （beta.n，tag vX.Y.Z-beta.n）
#   release（X.Y.Z，tag vX.Y.Z）
#
# 流程：校验 → 全量测试 → bump 版本 → 聚合 CHANGELOG → commit + tag
#       → 推 prep 合并分支 → 开 PR 到目标分支 → 推 tag（beta/release）
#
# 说明：目标分支（dev/beta/release）受保护，脚本不直接 push 分支，
#       而是把改动提交到 prep 分支并开 PR，由 review 后合并落地。
# ============================================================================
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

STAGE="${1:?用法: scripts/release.sh <alpha|beta|release> [版本号]}"
case "$STAGE" in
  alpha|beta|release) ;;
  *) echo "错误: stage 必须是 alpha|beta|release" >&2; exit 1 ;;
esac

# 目标分支：alpha→dev、beta→beta、release→release
case "$STAGE" in
  alpha)   TARGET_BRANCH="dev" ;;
  beta)    TARGET_BRANCH="beta" ;;
  release) TARGET_BRANCH="release" ;;
esac

CUR_BRANCH="$(git branch --show-current)"
if [ "$CUR_BRANCH" != "$TARGET_BRANCH" ]; then
  echo "错误: stage=$STAGE 应在 $TARGET_BRANCH 分支，当前在 $CUR_BRANCH" >&2
  exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "错误: 需要 gh（GitHub CLI）来创建 PR，请先安装并 gh auth login" >&2
  exit 1
fi

VERSION="${2:-}"

echo "==> [1/8] 校验工作树干净"
if [ -n "$(git status --porcelain)" ]; then
  echo "错误：工作树不干净，请先提交或 stash。" >&2
  git status --porcelain
  exit 1
fi

echo "==> [2/8] 确保基于最新 $TARGET_BRANCH"
git fetch origin "$TARGET_BRANCH"
git merge --ff-only "origin/$TARGET_BRANCH"

echo "==> [3/8] 全量测试"
make check-all
cargo test -p jpcg_core -- golden

echo "==> [4/8] 版本处理（stage=$STAGE, 版本=${VERSION:-未显式给定}）"
if [ -z "$VERSION" ]; then
  CUR="$(python3 -c "import re;print(re.search(r'^version\s*=\s*\"([^\"]+)\"',open('Cargo.toml').read(),re.M).group(1))")"
  case "$STAGE" in
    alpha)   VERSION="$CUR" ;;
    beta)    VERSION="$(echo "$CUR" | sed -E 's/-alpha\.[0-9]+/-beta.1/')" ;;
    release) VERSION="$(echo "$CUR" | sed -E 's/-alpha\.[0-9]+//; s/-beta\.[0-9]+//')" ;;
  esac
  echo "  推断版本: $VERSION"
fi

cargo set-version "$VERSION"
scripts/sync-version.sh "$VERSION"

echo "==> [5/8] 聚合 CHANGELOG"
python3 - "$VERSION" <<'PY'
import re,sys,glob,datetime
ver=sys.argv[1]
entries=[]
for f in sorted(glob.glob('changes/*.md')):
    entries.append(f"### 来自 {f}\n\n"+open(f).read().strip())
body="\n\n".join(entries) if entries else "（无逐条变更日志）"
today=datetime.date.today().isoformat()
section=f"\n## [{ver}] - {today}\n\n{body}\n"
text=open('CHANGELOG.md').read()
idx=text.find('\n## [Unreleased]')
end=text.find('\n## ', idx+1)
if end==-1: end=len(text)
text=text[:idx]+section+"\n## [Unreleased]\n\n### 新增\n-（下次发布的变更将列于此）\n"+text[end:]
open('CHANGELOG.md','w').write(text)
print(f"    已更新 CHANGELOG.md -> [{ver}]")
PY

echo "==> [6/8] commit + tag"
git add -A
git commit -q -m "release($STAGE): v${VERSION}"
if [ "$STAGE" != "alpha" ]; then
  git tag -a "v${VERSION}" -m "JPCG v${VERSION}"
  echo "  已打 tag: v${VERSION}"
else
  echo "  alpha 阶段不 tag"
fi

echo "==> [7/8] 推 prep 合并分支 + 开 PR"
PREP_BRANCH="release/prep-${VERSION}"
git push origin "HEAD:$PREP_BRANCH" >/dev/null
PR_URL="$(gh pr create --base "$TARGET_BRANCH" --head "$PREP_BRANCH" \
  --title "release($STAGE): v${VERSION}" \
  --body "发布准备（$STAGE，v${VERSION}）：版本 bump + CHANGELOG 聚合。\n\n请 review 并 squash 合并到 $TARGET_BRANCH。")"
echo "  PR: $PR_URL"

echo "==> [8/8] 推 tag（beta/release）"
if [ "$STAGE" != "alpha" ]; then
  git push origin "v${VERSION}"
  echo "  已推 tag: v${VERSION}（触发 release.yml）"
fi

echo ""
echo "✅ 完成: stage=$STAGE version=$VERSION"
echo "   下一步：reviewer 合并 PR ${PR_URL:-} 后，版本提交即落地 $TARGET_BRANCH。"
echo "   PR 合并后可删除 prep 分支: git push origin --delete $PREP_BRANCH"
