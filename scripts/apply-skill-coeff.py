#!/usr/bin/env python3
"""把 data/skill-shots/mowen-final.json 的整数倍率写回 data/shuxing/mowen.toml。

倍率口径：ratio_int = int(攻击加成 * 160 / 攻击力)；写 atk_xishu = ratio_int / 160，
base_min/base_max 写回 base_damage1/base_damage2。按 (skill_id, sub_id) 精确匹配。
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
JSON_PATH = ROOT / "data/skill-shots/mowen-final.json"
TOML_PATH = ROOT / "data/shuxing/mowen.toml"
BASE = 160


def fmt(v: float) -> str:
    text = f"{v:.8f}".rstrip("0").rstrip(".")
    return text or "0"


def set_key(block: str, key: str, value: str) -> str:
    pattern = rf"^{key}\s*=.*$"
    if re.search(pattern, block, flags=re.M):
        return re.sub(pattern, f"{key} = {value}", block, count=1, flags=re.M)
    # 缺失则插到 skill_id 行之后
    m = re.search(r"^skill_id\s*=.*$", block, flags=re.M)
    if m:
        pos = m.end()
        insert = f"\n{key} = {value}"
        return block[:pos] + insert + block[pos:]
    return block


def main() -> int:
    data = json.loads(JSON_PATH.read_text(encoding="utf-8"))
    targets = {}
    for s in data["skills"]:
        if s.get("skill_id") and s.get("ratio_int") is not None:
            targets[(int(s["skill_id"]), int(s.get("sub_id") or 0))] = s
    # DOT 每跳：tooltip 只给「每 3 秒 1 点(+750)/(+731)」，按整数倍率 14 处理
    for key, ratio in {(14065, 9357): 14, (14067, 9361): 14}.items():
        targets[key] = {"ratio_int": ratio, "base_min": 1, "base_max": 1}
    # 高山流水变体：变宫/变徵 与 宫/徵 同源（伤害一致，仅可移动吟唱）
    targets[(14298, 18860)] = {"ratio_int": 51, "base_min": 80, "base_max": 90}
    targets[(14299, 18859)] = {"ratio_int": 35, "base_min": 84, "base_max": 92}

    lines = TOML_PATH.read_text(encoding="utf-8").splitlines(keepends=True)
    out: list[str] = []
    i = 0
    updated = []
    while i < len(lines):
        if lines[i].strip() != "[[skill]]":
            out.append(lines[i])
            i += 1
            continue
        j = i + 1
        while j < len(lines) and not lines[j].startswith("["):
            j += 1
        block = "".join(lines[i:j])
        sid = re.search(r"^skill_id\s*=\s*(\d+)", block, flags=re.M)
        sub = re.search(r"^sub_id\s*=\s*(\d+)", block, flags=re.M)
        if sid:
            key = (int(sid.group(1)), int(sub.group(1)) if sub else 0)
            target = targets.get(key)
            if target:
                ratio = int(target["ratio_int"])
                block = set_key(block, "atk_xishu", fmt(ratio / BASE))
                if target.get("base_min") is not None:
                    block = set_key(block, "base_damage1", str(target["base_min"]))
                if target.get("base_max") is not None:
                    block = set_key(block, "base_damage2", str(target["base_max"]))
                updated.append(key)
        out.append(block)
        i = j
    TOML_PATH.write_text("".join(out), encoding="utf-8")
    for key in sorted(updated):
        print("updated", key)
    print(f"total {len(updated)} / {len(targets)}")
    missing = sorted(set(targets) - set(updated))
    if missing:
        print("missing:", missing)
    return 0


if __name__ == "__main__":
    sys.exit(main())
