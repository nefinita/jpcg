#!/usr/bin/env python3
"""Actual gear cdylib conversion and optional pinned upstream differential check.

cargo build -p jpcg_gear
python3 scripts/test-attributes-dynamic.py
python3 scripts/test-attributes-dynamic.py --reference-dir /path/to/trusted/Generator

Reference must be a clean checkout of PINNED. No third-party code/data is vendored.
This executes that checkout's Python model and never implies in-game verification.
"""
import argparse
import ctypes
import json
import os
from pathlib import Path
import random
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
PINNED = "21f472e211581ac1c1a7fa5ea2ff9199df6151cb"
RULE = "jx3-130-mowen-20260814"
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--reference-dir", type=Path)
parser.add_argument("--samples", type=int, default=1000)
args = parser.parse_args()
os.environ["JPCG_DATA_DIR"] = str(ROOT / "data")
name = "jpcg_gear.dll" if sys.platform == "win32" else "libjpcg_gear.dylib" if sys.platform == "darwin" else "libjpcg_gear.so"
lib = ctypes.CDLL(str(ROOT / "target/debug" / name))
lib.jpcg_gear_call.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
lib.jpcg_gear_call.restype = ctypes.c_void_p
lib.jpcg_gear_free_string.argtypes = [ctypes.c_void_p]
lib.jpcg_gear_free_string.restype = None

def require(condition, message):
    if not condition:
        raise AssertionError(message)

def call(method, request):
    pointer = lib.jpcg_gear_call(method.encode(), json.dumps(request).encode())
    require(pointer, "null gear result")
    try:
        result = json.loads(ctypes.string_at(pointer))
    finally:
        lib.jpcg_gear_free_string(pointer)
    require(result["status"] == "ok", result)
    return result["result"]

loadout = tomllib.loads((ROOT / "data/gear/sample-loadout.toml").read_text())
loadout["attribute_rules"] = RULE
loadout["base"]["tizhi"] = 1001
loadout["base"]["yuanqi"] = 1001
result = call("compose", loadout)
panel, raw, player = result["composed"]["panel"], result["composed"]["stats"], result["player"]
require(panel["qixue"] == (raw["qixue"] + 10 * raw["tizhi"]) * 1331 // 1024, "HP mismatch")
require(player["huixin_dengji"] == panel["huixin"], "crit dropped")
require(player["zhuanhua_gongji"] == panel["gongji_extra"], "extra attack dropped")
require(player["jiasu_dengji"] == raw["jiasu"], "haste dropped")
require(call("import", {"toml": call("export", loadout)}) == result, "TOML roundtrip mismatch")
print("PASS actual gear cdylib: converted HP/crit/extra attack/haste and TOML roundtrip")

if args.reference_dir:
    reference = args.reference_dir.resolve()
    head = subprocess.check_output(["git", "-C", str(reference), "rev-parse", "HEAD"], text=True).strip()
    require(head == PINNED, "Reference must be pinned to " + PINNED)
    subprocess.run(["git", "-C", str(reference), "diff", "--exit-code", "HEAD", "--", "base", "tools", "assets/json/belongs.json"], check=True, stdout=subprocess.DEVNULL)
    sys.path.insert(0, str(reference))
    from base.attribute import BaseAttribute
    from base.constant import LEVEL
    require(LEVEL == 130, "Reference level mismatch")
    with (reference / "assets/json/belongs.json").open() as source:
        school = json.load(source)["10447"]["10447"]["attributes"]
    rng = random.Random(98)
    for i in range(args.samples):
        loadout["base"] = {k: rng.randrange(1_000_001) for k in ("zhushuxing", "gengu", "yuanqi", "tizhi", "gongji", "huixin", "pofang", "qixue")}
        result = call("compose", loadout)["composed"]
        raw, panel = result["stats"], result["panel"]
        x = BaseAttribute("spirit", "lunar", "lunar")
        for key in ("spirit_to_lunar_attack_power", "spirit_to_lunar_critical_strike", "max_life_gain"):
            setattr(x, key, school[key])
        x.spirit_base = raw["zhushuxing"] + raw["gengu"]
        x.spunk_base = raw["yuanqi"]
        x.vitality_base = raw["tizhi"]
        x.lunar_attack_power_base = raw["gongji"]
        x.lunar_critical_strike_base = raw["huixin"]
        x.lunar_overcome_base = raw["pofang"]
        x.max_life_base = raw["qixue"]
        expected = (x.base_lunar_attack_power, x.extra_lunar_attack_power, x.lunar_attack_power, x.final_lunar_critical_strike, x.final_lunar_overcome, x.max_life)
        actual = tuple(panel[k] for k in ("gongji_base", "gongji_extra", "gongji", "huixin", "pofang", "qixue"))
        require(actual == expected, f"sample {i}: {raw}: {actual} != {expected}")
    print(f"PASS {args.samples} actual Rust/upstream differential samples, seed=98, source={PINNED}")
