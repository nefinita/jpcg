#!/usr/bin/env bash
# ============================================================================
# sync-version.sh — 版本同步
# 读取根 workspace 版本（core / release tag / 安装包命名源），写入：
#   - examples/jpcg_app/package.json           (version)
#   - examples/jpcg_app/src-tauri/tauri.conf.json (version)
#   - examples/jpcg_app/src/api/commands.ts    (模拟串 v<ver>)
# 用法：scripts/sync-version.sh [版本号]（缺省从根 Cargo.toml 读取）
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

# 3. commands.ts 模拟版本串（v<ver>）
python3 - "$VERSION" <<'PY'
import re,sys
ver=sys.argv[1]
p='examples/jpcg_app/src/api/commands.ts'
text=open(p).read()
text=re.sub(r'current_app_version:\s*"[^"]*"', f'current_app_version: "v{ver}"', text)
text=re.sub(r'latest_app_version:\s*"[^"]*"', f'latest_app_version: "v{ver}"', text)
open(p,'w').write(text)
print(f"    更新 {p} 模拟版本串 -> v{ver}")
PY

echo "==> 版本同步完成: $VERSION"
