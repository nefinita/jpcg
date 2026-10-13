#!/usr/bin/env python3
"""Snapshot-only recovery across the real, separately loaded core/combo cdylibs.

Run from any directory (Python 3.11+, standard library only):
    cargo build -p jpcg_core -p jpcg_combo
    python3 scripts/test-values-dynamic.py
    python3 scripts/test-values-dynamic.py --modules-dir target/release

Missing modules are an error, never a skipped/passing test. Each case uses a fresh
subprocess and temporary data root, so no prior successful cache entry can mask
negative-cache poisoning. Both libraries stay loaded, with the same handles,
through failure and repair. No reload/invalidation/broadcast command is called.
Only live-130.toml is repaired; index bytes, inode, size and mtime are checked.

The explicit-coefficient baseline bypasses value-set denominators, while recovery
requests omit them. This tests selection/recovery, not the underlying formulas
(the Rust golden tests cover those). Zero target HP skips combo Monte Carlo,
so the complete response is deterministic and inexpensive to compare.
"""

import argparse
import ctypes
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
CASES = ("missing", "malformed", "invalid")
COEFFICIENT_KEYS = (
    "pofang_xishu", "huixin_xishu", "huixiao_xishu", "yujin_xishu",
    "yuhui_xishu", "huajin_xishu", "fangyu_xishu", "pvp_global_jianshang",
)
SKILL = {
    "skill_name": "snapshot-recovery-smoke", "skill_id": 1,
    "base_damage1": 100, "base_damage2": 100, "atk_xishu": 5.0,
}
XINFA = {
    "profession": "recovery_smoke", "xinfa_name": "smoke", "xinfa_nom": "gengu",
    "atk_up": 1.96, "pofang_up": 2.0, "huixin_up": 0.0,
}


def require(condition, message):
    # Unlike assert, these checks remain active with python -O.
    if not condition:
        raise AssertionError(message)


class Module:
    def __init__(self, path, prefix):
        # RTLD_LOCAL (ctypes default) mirrors independent module loading rather
        # than intentionally making one module's symbols global to the other.
        self.lib = ctypes.CDLL(str(path))
        for suffix, args, result in (
            ("handle_create", [ctypes.c_char_p], ctypes.c_void_p),
            ("handle_free", [ctypes.c_void_p], None),
            ("call", [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p], ctypes.c_void_p),
            ("last_error", [], ctypes.c_void_p),
            ("free_string", [ctypes.c_void_p], None),
            ("abi_version", [], ctypes.c_uint32),
        ):
            fn = getattr(self.lib, f"{prefix}_{suffix}")
            fn.argtypes, fn.restype = args, result
            setattr(self, suffix, fn)
        require(self.abi_version() == 1, f"Unsupported ABI: {path}")
        self.handle = self.handle_create(b"{}")
        require(self.handle, f"Cannot create handle: {path}")

    def take_string(self, pointer):
        try:
            return ctypes.string_at(pointer).decode("utf-8")
        finally:
            self.free_string(pointer)

    def invoke(self, method, request):
        pointer = self.call(
            self.handle, method.encode("utf-8"),
            json.dumps(request, allow_nan=False).encode("utf-8"),
        )
        if not pointer:
            error = self.last_error()
            raise RuntimeError(f"{method}: {self.take_string(error) if error else 'unknown FFI error'}")
        return json.loads(self.take_string(pointer))

    def close(self):
        if self.handle:
            self.handle_free(self.handle)
            self.handle = None


def request(coefficient, combo=False):
    result = {
        "player": {
            "jcsx": "gengu", "jichu_shuxing": 21371, "jichu_gongji": 64329,
            "huixin_dengji": 61877, "huixin_xiaoguo": 2925,
            "pofang_dengji": 109160, "wuqi_shanghai": 0,
        },
        "hostile": {
            "waigong_fangyu": 15176, "neigong_fangyu": 21388,
            "yujin_dengji": 5047, "huajin_dengji": 59402,
            "jianshang_bili": 0, "target_hp": 0, "max_hp": 0, "current_hp": 0,
        },
        "xinfa" if combo else "xinfa_config": XINFA,
        "buff": {}, "coefficient": coefficient, "value_set": "live-130",
    }
    if combo:
        result["steps"] = [{"skill": SKILL}]
    return result


def index_stamp(path):
    stat = path.stat()
    return (path.read_bytes(), stat.st_ino, stat.st_size, stat.st_mtime_ns)


def run_case(case, core_path, combo_path):
    live = (ROOT / "data/values/live-130.toml").read_bytes()
    parsed = tomllib.loads(live.decode("utf-8"))
    coefficients = {key: parsed[key] for key in COEFFICIENT_KEYS}
    # Global mitigation is an explicit override (0 is valid), unlike denominators.
    selected = {"pvp_global_jianshang": coefficients["pvp_global_jianshang"]}
    with tempfile.TemporaryDirectory(prefix=f"jpcg-values-{case}-") as temporary:
        root = Path(temporary)
        values = root / "values"
        values.mkdir()
        shuxing = root / "shuxing"
        shuxing.mkdir()
        (shuxing / "recovery_smoke.toml").write_text(
            "[xinfa]\n" + "\n".join(f"{k} = {json.dumps(v)}" for k, v in XINFA.items())
            + "\n[[skill]]\n" + "\n".join(f"{k} = {json.dumps(v)}" for k, v in SKILL.items())
            + "\n", encoding="utf-8",
        )
        index = values / "index.toml"
        index.write_text(
            'default = "live-130"\n[[value_sets]]\nid = "live-130"\n'
            'name = "Live recovery smoke"\nlevel = 130\nfile = "live-130.toml"\n',
            encoding="utf-8",
        )
        snapshot = values / "live-130.toml"
        if case == "malformed":
            snapshot.write_text("[broken\n", encoding="utf-8")
        elif case == "invalid":
            invalid = dict(parsed, huixin_xishu=0.0)
            snapshot.write_text(
                "\n".join(f"{k} = {v}" for k, v in invalid.items()) + "\n",
                encoding="utf-8",
            )
        unchanged_index = index_stamp(index)
        os.environ["JPCG_DATA_DIR"] = str(root)
        core = Module(core_path, "jpcg")
        try:
            combo = Module(combo_path, "jpcg_combo")
            try:
                # Warm failure in BOTH cdylibs before repairing anything.
                fallback_info = core.invoke("resolve_value_set", {"value_set": "live-130"})
                require(fallback_info["source"] == "builtin", f"{case}: core did not fall back")
                fallback_core = core.invoke("calculate", request(selected))
                fallback_combo = combo.invoke("calculate_combo", request(selected, combo=True))
                expected_core = core.invoke("calculate", request(coefficients))
                expected_combo = combo.invoke("calculate_combo", request(coefficients, combo=True))
                require(len(expected_core) == 1, "Core must load exactly the fixture skill")
                require(len(expected_combo["steps"]) == 1, "Combo must calculate exactly one step")
                require(expected_core[0]["q"] > 0, "Live core baseline must deal damage")
                require(expected_combo["steps"][0]["q_damage"] > 0, "Live combo baseline must deal damage")
                require(fallback_core != expected_core, "Fixture must distinguish core fallback from live")
                require(fallback_combo != expected_combo, "Fixture must distinguish combo fallback from live")

                snapshot.write_bytes(live)  # The only repair; no index touch/broadcast.
                require(index_stamp(index) == unchanged_index, "Repair changed index identity/content/mtime")
                # Query combo FIRST, before any post-repair core call, to exclude
                # recovery depending on a successful core resolve/broadcast.
                recovered_combo = combo.invoke("calculate_combo", request(selected, combo=True))
                require(recovered_combo == expected_combo, f"{case}: combo retained fallback after repair")
                recovered_info = core.invoke("resolve_value_set", {"value_set": "live-130"})
                require(
                    recovered_info["id"] == "live-130" and recovered_info["source"] == "data"
                    and recovered_info["available"], f"{case}: core resolve retained fallback",
                )
                for key, value in coefficients.items():
                    require(
                        math.isclose(recovered_info["coefficient"][key], value, rel_tol=1e-6),
                        f"{case}: wrong resolved live coefficient {key}",
                    )
                require(core.invoke("calculate", request(selected)) == expected_core,
                        f"{case}: core calculation retained fallback after repair")
                require(index_stamp(index) == unchanged_index, "FFI calls changed the index")
            finally:
                combo.close()
        finally:
            core.close()
    print(f"PASS {case}: both cdylibs recovered after snapshot-only repair", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--modules-dir", type=Path, default=ROOT / "target/debug")
    parser.add_argument("--case", choices=CASES, help=argparse.SUPPRESS)
    args = parser.parse_args()
    names = {
        "linux": ("libjpcg_core.so", "libjpcg_combo.so"),
        "darwin": ("libjpcg_core.dylib", "libjpcg_combo.dylib"),
        "win32": ("jpcg_core.dll", "jpcg_combo.dll"),
    }
    require(sys.platform in names, f"Unsupported platform: {sys.platform}")
    modules = args.modules_dir.resolve()
    core, combo = (modules / name for name in names[sys.platform])
    for path in (core, combo):
        require(path.is_file(), f"Missing module {path}; run cargo build -p jpcg_core -p jpcg_combo")
    if args.case:
        run_case(args.case, core, combo)
    else:
        for case in CASES:
            subprocess.run(
                [sys.executable, str(Path(__file__).resolve()), "--modules-dir", str(modules), "--case", case],
                check=True,
            )
        print("PASS: all dual-cdylib snapshot recovery cases")


if __name__ == "__main__":
    main()
