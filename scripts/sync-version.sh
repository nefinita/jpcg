#!/usr/bin/env bash
# ============================================================================
# sync-version.sh — 版本同步
# 读取根 workspace 版本（core / release tag / 安装包命名源），写入：
#   - examples/jpcg_app/package.json              (version)
#   - examples/jpcg_app/src-tauri/tauri.conf.json (version)
#   - examples/jpcg_app/src/api/commands.ts       (模拟串 v<ver> + get_module_versions 模块版本)
#   - examples/jpcg_app/package-lock.json         (根 version / packages[""].version)
# 用法：scripts/sync-version.sh [版本号]（缺省从根 Cargo.toml 读取）
#      可选环境变量 NPM_CACHE=<dir> 指定 npm 缓存（HOME 只读 / CI 环境用）
# ============================================================================
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# 从根 Cargo.toml [workspace.package] version 读取（如未显式传入）
if [ "$#" -ge 1 ]; then
  VERSION="$1"
else
  VERSION="$(python3 - <<'PY'
import re,sys
text=open('Cargo.toml').read()
m=re.search(r'\[workspace\.package\][^[]*?^\s*version\s*=\s*"([^"]+)"', text, re.M|re.S)
print(m.group(1) if m else '')
PY
)"
fi

if [ -z "$VERSION" ]; then
  echo "错误：无法确定版本号（请传参数或检查根 Cargo.toml [workspace.package].version）" >&2
  exit 1
fi
echo "==> 同步版本: $VERSION"

# 1. package.json
python3 - "$VERSION" <<'PY'
import json,sys
ver=sys.argv[1]
p='examples/jpcg_app/package.json'
d=json.load(open(p)); d['version']=ver
json.dump(d,open(p,'w'),indent=2,ensure_ascii=False); open(p,'a').write('\n')
print(f"    更新 {p} -> {ver}")
PY

# 2. tauri.conf.json
python3 - "$VERSION" <<'PY'
import json,sys
ver=sys.argv[1]
p='examples/jpcg_app/src-tauri/tauri.conf.json'
d=json.load(open(p)); d['version']=ver
json.dump(d,open(p,'w'),indent=2,ensure_ascii=False); open(p,'a').write('\n')
print(f"    更新 {p} -> {ver}")
PY

# 3. commands.ts 模拟版本串：
#    - check_update: current_app_version / latest_app_version（带 v 前缀）
#    - get_module_versions: core/update/combo/app（继承 workspace 版本，无 v 前缀；
#      const 为独立的「等级.赛季.日期」，不在此同步）
python3 - "$VERSION" <<'PY'
import re,sys
ver=sys.argv[1]
p='examples/jpcg_app/src/api/commands.ts'
text=open(p).read()
text=re.sub(r'current_app_version:\s*"[^"]*"', f'current_app_version: "v{ver}"', text)
text=re.sub(r'latest_app_version:\s*"[^"]*"', f'latest_app_version: "v{ver}"', text)
# 只改 get_module_versions mock 段内的模块版本，避免影响文件其他处同名键
m=re.search(r'(case "get_module_versions":\s*return \{)(.*?)(\};)', text, re.S)
if m:
    block=m.group(2)
    for key in ('core','update','combo','app'):
        block=re.sub(rf'({key}:\s*")[^"]*(")',
                     lambda mo: f'{mo.group(1)}{ver}{mo.group(2)}', block)
    text=text[:m.start(2)]+block+text[m.end(2):]
    print(f"    更新 {p} 模拟版本串 + get_module_versions(core/update/combo/app) -> {ver}")
else:
    # 形状被重构时只告警、不中断：其余版本同步仍有效，const 永远不碰
    print(f"    警告：未在 {p} 找到 get_module_versions mock，跳过模块版本同步", file=sys.stderr)
open(p,'w').write(text)
PY

# 4. package-lock.json 根 version 与 packages[""].version
#    优先用 npm 重新生成 lock（--package-lock-only：只改 lock，不装依赖、不动 node_modules，
#    也不会升级任何依赖版本）；npm 不可用/执行失败时回退到 python 最小编辑（只改 version 字段）。
PKG_LOCK="examples/jpcg_app/package-lock.json"
LOCK_SYNCED=0
if command -v npm >/dev/null 2>&1; then
  NPM_ARGS=(install --package-lock-only --ignore-scripts --no-audit --no-fund)
  if [ -n "${NPM_CACHE:-}" ]; then
    mkdir -p "$NPM_CACHE"
    NPM_ARGS+=(--cache "$NPM_CACHE")
  fi
  if (cd examples/jpcg_app && npm "${NPM_ARGS[@]}" >/dev/null 2>&1); then
    LOCK_SYNCED=1
  else
    echo "    警告：npm 同步 package-lock 失败，回退到最小编辑" >&2
  fi
fi

lock_version_ok() {
  python3 - "$VERSION" "$PKG_LOCK" <<'PY'
import json,sys
ver,p=sys.argv[1],sys.argv[2]
d=json.load(open(p))
root=d.get('packages',{}).get('',{})
sys.exit(0 if d.get('version')==ver and root.get('version')==ver else 1)
PY
}

if [ "$LOCK_SYNCED" -eq 1 ] && lock_version_ok; then
  echo "    更新 $PKG_LOCK -> $VERSION (npm --package-lock-only)"
else
  python3 - "$VERSION" "$PKG_LOCK" <<'PY'
import json,sys
ver,p=sys.argv[1],sys.argv[2]
d=json.load(open(p))
d['version']=ver
if isinstance(d.get('packages'),dict) and '' in d['packages']:
    d['packages']['']['version']=ver
json.dump(d,open(p,'w'),indent=2,ensure_ascii=False); open(p,'a').write('\n')
print(f"    更新 {p} -> {ver} (最小编辑)")
PY
fi

echo "==> 版本同步完成: $VERSION"
