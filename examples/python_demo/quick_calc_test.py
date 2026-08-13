#!/usr/bin/env python3
"""JPCG 全量快测 — 直连 libjpcg_core，用预设真实属性计算整个 toml。

用法：
    cargo build -p jpcg_core
    python3 examples/python_demo/quick_calc_test.py [--data <数据目录>] [--lib <core库路径>]

功能：
    1. 全量计算 mowen.toml 所有技能（calculate）
    2. DOT 专项断言：商/角 普通 6 跳每跳相等 q=Σ；疏曲 9 跳等比 ×1.12^k
    3. 面板百分比对照（化劲/御劲/防御，校验引擎系数）
    4. 输出完整 JSON（供金标准回填）

退出码：0 全部通过 / 1 有失败
"""

import argparse
import ctypes
import json
import os
import sys
import tempfile
from pathlib import Path

# 预设属性（用户真实面板 + 木桩目标 + pvp 0.9）
PRESET = {
    "player": {
        "jcsx": "根骨",
        "jichu_shuxing": 21371,
        "jichu_gongji": 64329,
        "huixin_dengji": 61877,
        "huixin_xiaoguo": 2925,
        "pofang_dengji": 109160,
        "wuqi_shanghai": 0,
    },
    "hostile": {
        "waigong_fangyu": 15176,
        "neigong_fangyu": 21388,
        "yujin_dengji": 5047,
        "huajin_dengji": 59402,
        "jianshang_bili": 0,
        "target_hp": 0,
    },
    "xinfa_config": {
        "profession": "mowen",
        "xinfa_name": "莫问",
        "xinfa_nom": "根骨",
        "atk_up": 1.96,
        "pofang_up": 2.0,
        "huixin_up": 0.0,
    },
    "buff": {
        "base_atk_pct": 0.0, "huixin_pct": 0.0, "huixiao_pct": 0.0,
        "pofang_pct": 0.0, "wushi_fangyu_pct": 0.0,
        "shanghai_pct": 0.0, "mode_is_point": True,
    },
    "coefficient": {
        "pofang_xishu": 225957.6,
        "huixin_xishu": 197703.0,
        "huixiao_xishu": 72844.2,
        "huajin_xishu": 30115.8,
        "fangyu_xishu": 126007.2,
        "pvp_global_jianshang": 0.9,
    },
}

# 面板百分比对照（引擎公式换算 vs 用户面板）
PANEL_EXPECT = [
    # (名称, 用 hostile 的哪个字段, 公式, 期望面板%)
    ("化劲", "huajin_dengji", lambda h: (h / (h + 30115.8) + 102 / 1024) * 100, 76.32),
    ("御劲会心", "yujin_dengji", lambda h: h / 197703.0 * 100, 2.55),
    ("外防", "waigong_fangyu", lambda h: h / (h + 126007.2) * 100, 10.75),
    ("内防", "neigong_fangyu", lambda h: h / (h + 126007.2) * 100, 14.51),
]

# DOT 期望规则: (技能名子串, 跳数, dot_up)
DOT_RULES = [
    ("商（dot）", 6, 0.0, "普通"),
    ("角（dot）", 6, 0.0, "普通"),
]


class Core:
    def __init__(self, path: str):
        self.lib = ctypes.CDLL(path)
        self.lib.jpcg_handle_create.restype = ctypes.c_void_p
        self.lib.jpcg_handle_create.argtypes = [ctypes.c_char_p]
        self.lib.jpcg_handle_free.argtypes = [ctypes.c_void_p]
        self.lib.jpcg_call.restype = ctypes.c_void_p
        self.lib.jpcg_call.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
        self.lib.jpcg_last_error.restype = ctypes.c_void_p
        self.lib.jpcg_free_string.argtypes = [ctypes.c_void_p]
        self.lib.jpcg_abi_version.restype = ctypes.c_uint32

        self.handle = self.lib.jpcg_handle_create(b"{}")
        if not self.handle:
            raise SystemExit("jpcg_handle_create 失败")

    def call(self, method: str, request: dict) -> dict:
        resp = self.lib.jpcg_call(
            self.handle, method.encode(), json.dumps(request, ensure_ascii=False).encode()
        )
        if not resp:
            err_ptr = self.lib.jpcg_last_error()
            err = ctypes.string_at(err_ptr).decode() if err_ptr else "未知错误"
            if err_ptr:
                self.lib.jpcg_free_string(err_ptr)
            raise RuntimeError(f"[{method}] {err}")
        text = ctypes.string_at(resp).decode()
        self.lib.jpcg_free_string(resp)
        return json.loads(text)

    def close(self):
        if self.handle:
            self.lib.jpcg_handle_free(self.handle)
            self.handle = None


def find_lib(arg: str | None) -> str:
    candidates = [arg] if arg else []
    name = {"darwin": "libjpcg_core.dylib", "linux": "libjpcg_core.so", "win32": "jpcg_core.dll"}.get(sys.platform)
    if name:
        candidates += [f"target/debug/{name}", f"target/release/{name}"]
    for c in candidates:
        if c and os.path.isfile(c):
            return c
    raise SystemExit("找不到 core 库，请先 cargo build -p jpcg_core 或传 --lib")


def check_dot(results: list[dict]) -> list[str]:
    """DOT 专项断言：普通 6 跳相等；疏曲 9 跳等比 1.12；q = Σ jumps；非 dot 空。
    同名条目（商（dot）普通/疏曲）按 TOML 出现顺序区分：第 1 次=普通，第 2 次=疏曲。"""
    errors: list[str] = []
    seen: dict[str, int] = {}

    for r in results:
        jumps = r.get("dot_jumps") or []
        name = r["skill_name"]
        is_dot = "（dot）" in name or "(dot)" in name.lower()
        if not is_dot:
            if jumps:
                errors.append(f"[{name}] 非 dot 技能却返回 dot_jumps: {jumps}")
            continue

        idx = seen.get(name, 0) + 1
        seen[name] = idx
        up = 0.12 if idx > 1 else 0.0
        n = 9 if idx > 1 else 6
        if len(jumps) != n:
            errors.append(f"[{name}#{idx}] 跳数 {len(jumps)} != {n}")
            continue
        first = jumps[0]
        if up == 0.0:
            if any(j != first for j in jumps):
                errors.append(f"[{name}#{idx}] 非递增条目每跳不等: {jumps}")
        else:
            for k, j in enumerate(jumps):
                expect = int(first * (1.0 + up) ** k)
                if abs(j - expect) > 1:
                    errors.append(f"[{name}#{idx}] 第{k + 1}跳 {j} != {expect} (±1)")
        if r["q"] != sum(jumps):
            errors.append(f"[{name}#{idx}] q={r['q']} != Σjumps={sum(jumps)}")
    return errors


def check_panel() -> list[str]:
    errors: list[str] = []
    h = PRESET["hostile"]
    for label, field, fn, expect in PANEL_EXPECT:
        got = fn(h[field])
        if abs(got - expect) > 0.5:
            errors.append(f"[面板] {label} 引擎换算 {got:.2f}% != 面板 {expect}%")
    return errors


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", help="数据目录（含 shuxing/ 或直接 shuxing 目录）")
    ap.add_argument("--lib", help="libjpcg_core 路径")
    ap.add_argument("--out", help="结果 JSON 输出路径（供金标准回填）")
    args = ap.parse_args()

    if args.data:
        os.environ["JPCG_DATA_DIR"] = args.data
    else:
        repo = Path(__file__).resolve().parents[2]
        os.environ.setdefault("JPCG_DATA_DIR", str(repo / "data"))

    core = Core(find_lib(args.lib))
    failed = False
    try:
        print(f"==> abi_version = {core.lib.jpcg_abi_version()}")
        print("==> 预设属性（真实面板 + 木桩 + pvp 0.9）")
        results = core.call("calculate", PRESET)
        print(f"==> 计算 {len(results)} 个技能\n")

        print(f"{'技能':<22}{'Y':>6}{'B':>9}{'I':>9}{'N':>9}{'H':>9}{'Q':>9}  dot_jumps")
        for r in results:
            jumps = r.get("dot_jumps") or []
            jumps_txt = ",".join(map(str, jumps)) if jumps else "-"
            print(
                f"{r['skill_name']:<22}{r['y']:>6}{r['b']:>9}{r['i']:>9}"
                f"{r['n']:>9}{r['h']:>9}{r['q']:>9}  {jumps_txt}"
            )

        if args.out:
            with open(args.out, "w") as f:
                json.dump(results, f, ensure_ascii=False, indent=1)
            print(f"\n==> 结果已写入 {args.out}")

        errors = check_panel() + check_dot(results)
        print("\n==> 面板对照")
        for label, field, fn, expect in PANEL_EXPECT:
            print(f"    {label}: 引擎 {fn(PRESET["hostile"][field]):.2f}% 期望 {expect}%")
        print("==> DOT 断言")
        for r in results:
            if r.get("dot_jumps"):
                print(f"    {r['skill_name']}: {len(r['dot_jumps'])} 跳, q={r['q']}")
        if errors:
            print("\n!!! 失败:")
            for e in errors:
                print(f"    {e}")
            failed = True
        else:
            print("\n==> ALL PASS")
    finally:
        core.close()

    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
