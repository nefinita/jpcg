#!/usr/bin/env bash
# ============================================================================
# release.sh — 三分支发布脚本（dev → beta → release）
#
# 用法：
#   scripts/release.sh <stage> [版本号]
#     stage   = alpha | beta | release
#     [版本]  缺省从当前 branch 推断（见下）
#
# 模型：
#   dev    （alpha.n，不 tag）       → 切/提升 beta 时
#   beta   （beta.n，tag vX.Y.Z-beta.n）
#   release（X.Y.Z，tag vX.Y.Z）
#
# 流程：校验干净 → 全量测试 → bump 版本 → 聚合 CHANGELOG → commit → [tag] → push
# ============================================================================
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

STAGE="${1:?用法: scripts/release.sh <alpha|beta|release> [版本号]}"
case "$STAGE" in
  alpha|beta|release) ;;
  *) echo "错误: stage 必须是 alpha|beta|release" >&2; exit 1 ;;
esac

CUR_BRANCH="$(git branch --show-current)"

# 分支约束：alpha→dev、beta→beta、release→release
EXPECT_BRANCH="$STAGE"
if [ "$STAGE" = "alpha" ]; then EXPECT_BRANCH="dev"; fi
if [ "$CUR_BRANCH" != "$EXPECT_BRANCH" ]; then
  echo "错误: stage=$STAGE 应在 $EXPECT_BRANCH 分支，当前在 $CUR_BRANCH" >&2
  exit 1
fi

VERSION="${2:-}"

echo "==> [1/6] 校验工作树干净"
if [ -n "$(git status --porcelain)" ]; then
  echo "错误：工作树不干净，请先提交或 stash。" >&2
  git status --porcelain
  exit 1
fi

echo "==> [2/6] 全量测试"
make check-all
cargo test -p jpcg_core -- golden

echo "==> [3/6] 版本处理（stage=$STAGE, 版本=${VERSION:-未显式给定}）"
if [ -z "$VERSION" ]; then
  # 未给版本：从当前 workspace 版本按 stage 规整（dev 保持 alpha，beta→beta，release 去后缀）
  CUR="$(python3 -c "import re;print(re.search(r'^version\s*=\s*\"([^\"]+)\"',open('Cargo.toml').read(),re.M).group(1))")"
  case "$STAGE" in
    alpha)   VERSION="$CUR" ;;                                  # 保持 alpha.n
    beta)    VERSION="$(echo "$CUR" | sed -E 's/-alpha\.[0-9]+/-beta.1/')" ;;
    release) VERSION="$(echo "$CUR" | sed -E 's/-alpha\.[0-9]+//; s/-beta\.[0-9]+//')" ;;
  esac
  echo "  推断版本: $VERSION"
fi

cargo set-version "$VERSION"
scripts/sync-version.sh "$VERSION"

echo "==> [4/6] 聚合 CHANGELOG"
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

echo "==> [5/6] commit + tag"
git add -A
git commit -q -m "release($STAGE): v${VERSION}"
if [ "$STAGE" != "alpha" ]; then
  git tag -a "v${VERSION}" -m "JPCG v${VERSION}"
  echo "  已打 tag: v${VERSION}"
else
  echo "  alpha 阶段不 tag"
fi

echo "==> [6/6] push"
git push origin "$CUR_BRANCH"
if [ "$STAGE" != "alpha" ]; then
  git push origin "v${VERSION}"
fi

echo "✅ 完成: stage=$STAGE version=$VERSION（$CUR_BRANCH）"
if [ "$STAGE" = "beta" ]; then echo "  提示: 公测稳定后，在 beta 分支再跑 scripts/release.sh release 发布稳定版"; fi
