#!/usr/bin/env python3
"""JPCG 全量快测 — 直连 libjpcg_core，用预设真实属性计算整个 toml。

用法：
    cargo build -p jpcg_core
    python3 examples/python_demo/quick_calc_test.py [--data <数据目录>] [--lib <core库路径>]

功能：
    1. 全量计算 mowen.toml 所有技能（calculate）
    2. DOT 专项断言：跳数按 toml 的 dot_duration/dot_interval 推导（浮点秒，支持 0.25s），
       dot_up>0 等比递增 ×(1+up)^k，否则每跳相等；q = Σ jumps；非 dot 空
    3. 面板百分比对照（化劲/御劲/防御，校验引擎系数）
    4. 无质断言（has_critical_strike=true 技能自动收集，q 固定=期望 Q）
    5. 输出完整 JSON（供金标准回填）

退出码：0 全部通过 / 1 有失败
"""

import argparse
import ctypes
import json
import os
import sys
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


def load_data() -> tuple[list[dict], bool]:
    """读取 mowen.toml 全部 [[skill]] 条目（按文件顺序）。返回 (skills, skipped)。
    缺 tomllib 或找不到 toml 时 skipped=True，调用方跳过依赖数据的断言。"""
    base = Path(os.environ["JPCG_DATA_DIR"])
    cand = [base / "shuxing" / "mowen.toml", base / "mowen.toml"]
    path = next((c for c in cand if c.is_file()), None)
    if path is None:
        return [], True
    try:
        import tomllib
    except ImportError:
        return [], True
    with open(path, "rb") as f:
        return tomllib.load(f).get("skill", []), False


def check_dot(results: list[dict], data: list[dict]) -> list[str]:
    """DOT 专项断言：期望跳数按 toml 的 dot_duration/dot_interval 推导（浮点秒），
    dot_up>0 等比递增 ×(1+up)^k，否则每跳相等；q = Σ jumps；非 dot 返回空集合。
    同名条目（商（dot）普通/疏曲）按 TOML 出现顺序与 results 同名顺序匹配。"""
    errors: list[str] = []
    expected: dict[str, list[tuple[float, float, float]]] = {}
    for sk in data:
        if not sk.get("dot_flag"):
            continue
        dur = sk.get("dot_duration")
        itv = sk.get("dot_interval")
        if dur is None or itv is None or itv <= 0:
            continue
        expected.setdefault(sk["skill_name"], []).append((dur, itv, sk.get("dot_up", 0.0)))
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
        queue = expected.get(name, [])
        if not queue:
            errors.append(f"[{name}#{idx}] toml 中无对应 dot 条目")
            continue
        dur, itv, up = queue.pop(0)
        n = round(dur / itv)
        if len(jumps) != n:
            errors.append(f"[{name}#{idx}] 跳数 {len(jumps)} != 期望 {n}（duration {dur}/interval {itv}）")
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

    for name, rest in expected.items():
        for i in range(len(rest)):
            errors.append(f"[{name}#{i + 1}] toml 有条目但 results 未匹配")
    return errors


def check_panel() -> list[str]:
    errors: list[str] = []
    h = PRESET["hostile"]
    for label, field, fn, expect in PANEL_EXPECT:
        got = fn(h[field])
        if abs(got - expect) > 0.5:
            errors.append(f"[面板] {label} 引擎换算 {got:.2f}% != 面板 {expect}%")
    return errors


def check_wuzhi(results: list[dict], wuzhi_names: list[str]) -> list[str]:
    """无质断言：无质技能 q 满足期望公式 q ≈ N×(1-p) + H×p
    （p = 玩家会心率 61877/197703 − 目标御劲减免 5047/197703，buff 全 0）
    wuzhi_names 由 toml 自动收集（has_critical_strike=true），须全部在 results 中出现。"""
    errors: list[str] = []
    crit = 61877 / 197703.0 - 5047 / 197703.0
    seen: set[str] = set()
    for r in results:
        if r["skill_name"] not in wuzhi_names:
            continue
        seen.add(r["skill_name"])
        n, h, q = r["n"], r["h"], r["q"]
        expect = n * (1.0 - crit) + h * crit
        if abs(q - expect) > 100:
            errors.append(f"[无质] {r['skill_name']} q={q} != 期望公式 {expect:.0f} (±100)")
    for name in wuzhi_names:
        if name not in seen:
            errors.append(f"[无质] {name} 未出现在计算结果中")
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

        data, data_skipped = load_data()
        wuzhi_names = [sk["skill_name"] for sk in data if sk.get("has_critical_strike")]
        errors = check_panel()
        if not data_skipped:
            errors += check_dot(results, data) + check_wuzhi(results, wuzhi_names)
        else:
            print("（跳过 DOT/无质数据断言：缺 tomllib 或找不到 mowen.toml）")
        print("\n==> 面板对照")
        for label, field, fn, expect in PANEL_EXPECT:
            print(f"    {label}: 引擎 {fn(PRESET["hostile"][field]):.2f}% 期望 {expect}%")
        print("==> DOT 断言")
        for r in results:
            jumps = r.get("dot_jumps") or []
            if jumps:
                print(f"    {r['skill_name']}: {len(jumps)} 跳, q={r['q']}")
                for k, j in enumerate(jumps):
                    print(f"        第{k + 1}跳: {j}")
        print("==> 无质断言（has_critical_strike=true 技能）")
        for r in results:
            if r["skill_name"] in wuzhi_names:
                print(f"    {r['skill_name']}: N={r['n']} H={r['h']} Q={r['q']}（应固定=期望Q）")
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
