#!/bin/sh
# ============================================================================
# fetch-json.sh — 拉取 IcyTide/Generator 数据源 JSON（技能/持续伤害/归属）
# 下载到 data/raw-src/（gitignore，不入库），供 json-to-toml 转换使用。
# 用法: ./fetch-json.sh
# ============================================================================
set -e

BASE="https://raw.githubusercontent.com/IcyTide/Generator/master/assets/json"
OUT_DIR="$(cd "$(dirname "$0")/../../data" && pwd)/raw-src"
mkdir -p "$OUT_DIR"

for f in skills dots belongs; do
  url="$BASE/$f.json"
  echo "==> $f.json"
  curl -sSL -o "$OUT_DIR/$f.json" "$url"
  ls -l "$OUT_DIR/$f.json" | awk '{print "    "$5" bytes  "$6" "$7" "$8}'
done

echo
echo "完成。转换命令示例："
echo "  cargo run -p json-to-toml -- --skills $OUT_DIR/skills.json --dots $OUT_DIR/dots.json \\"
echo "      --overrides server_tools/json-to-toml/overrides.json --out data/shuxing --xinfa 10786"
echo "（全职业列表见 data/shuxing/datamake.md）"
