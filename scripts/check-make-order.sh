#!/usr/bin/env bash
# check-make-order.sh — Makefile 依赖顺序回归守卫
#
# 背景：Tauri 的 build script（tauri-build）在**每次 cargo 编译 app 时**都会校验
# frontendDist（examples/jpcg_app/dist/）并复制 externalBin（jpcg_updater-<triple>）。
# 缺任一项即 cargo build 阶段直接失败——不只是 `tauri build`/打包时才需要。
# 因此 Makefile 里凡会编译 app 的目标都必须先准备这两项资源（$(APP_BUILD_DEPS)）。
#
# 本守卫不真正构建，只对 `make -n`（dry-run）的执行计划做顺序断言：
# 前端构建（npm run build）与 updater（cargo build -p jpcg_updater）必须出现在
# 第一条会编译 app 的命令（cargo build -p jpcg_app / cargo clippy --workspace）之前。
# 这样无需网络/UI 依赖即可在 CI 里便宜地防止顺序回退（回归 PR #78 的 review P2）。
set -uo pipefail

cd "$(dirname "$0")/.." || exit 1

# 目标 → 说明（空字符串 = 默认 goal）
TARGETS=("all" "check-all" "build-static" "build-dynamic" "test" "")

# 会编译 app（进而触发 tauri-build 校验）的第一批命令（build / clippy / test 都会编译 app）
APP_PATTERN='^[[:space:]]*cargo (build -p jpcg_app|clippy --workspace|test --workspace)'
# 前置资源命令
FRONTEND_PATTERN='npm run build'
UPDATER_PATTERN='^[[:space:]]*cargo build -p jpcg_updater'

fail=0

first_line() { # $1=pattern $2=file → 输出首次匹配行号（无匹配则空）
  grep -n -m1 -E -- "$1" "$2" 2>/dev/null | cut -d: -f1
}

check_target() {
  local target="$1" label="$2" plan app_line front_line upd_line
  plan="$(mktemp "${TMPDIR:-/tmp}/jpcg-make-plan.XXXXXX")" || exit 1

  if [ -n "$target" ]; then
    if ! make -n "$target" >"$plan" 2>&1; then
      echo "✗ make -n $target 失败："
      sed 's/^/    /' "$plan"
      rm -f "$plan"
      fail=1
      return
    fi
  else
    if ! make -n >"$plan" 2>&1; then
      echo "✗ make -n（默认 goal）失败："
      sed 's/^/    /' "$plan"
      rm -f "$plan"
      fail=1
      return
    fi
  fi

  app_line="$(first_line "$APP_PATTERN" "$plan")"
  front_line="$(first_line "$FRONTEND_PATTERN" "$plan")"
  upd_line="$(first_line "$UPDATER_PATTERN" "$plan")"

  if [ -z "$app_line" ]; then
    echo "✗ ${label}：dry-run 计划里找不到编译 app 的命令，Makefile 结构已变，守卫需同步更新"
    fail=1
  elif [ -z "$front_line" ] || [ "$front_line" -ge "$app_line" ]; then
    echo "✗ ${label}：编译 app（计划第 $app_line 行）前未构建前端 dist（npm run build 行：${front_line:-无}）"
    fail=1
  elif [ -z "$upd_line" ] || [ "$upd_line" -ge "$app_line" ]; then
    echo "✗ ${label}：编译 app（计划第 $app_line 行）前未准备 updater sidecar（cargo build -p jpcg_updater 行：${upd_line:-无}）"
    fail=1
  else
    echo "✓ ${label}：前端 dist（行 ${front_line}）与 updater（行 ${upd_line}）均先于 app 编译（行 ${app_line}）"
  fi

  rm -f "$plan"
}

for t in "${TARGETS[@]}"; do
  if [ -n "$t" ]; then
    check_target "$t" "make $t"
  else
    check_target "" "make（默认 goal = build-static）"
  fi
done

if [ "$fail" -ne 0 ]; then
  echo
  echo "错误：Makefile 的 app 编译前置顺序不符合要求（见上方 ✗）。" >&2
  echo "修复方向：给相关目标挂上 APP_BUILD_DEPS（frontend updater）前置，见 Makefile 头注释。" >&2
  exit 1
fi

echo "全部通过：所有会编译 app 的目标都先准备 frontend dist + updater sidecar"
