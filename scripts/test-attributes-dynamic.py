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
import tempfile

ROOT = Path(__file__).resolve().parents[1]
PINNED = "21f472e211581ac1c1a7fa5ea2ff9199df6151cb"
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--reference-dir", type=Path)
parser.add_argument("--samples", type=int, default=1000)
args = parser.parse_args()
if args.samples < 1:
    parser.error("--samples must be positive")
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

rules = [
    ("mowen", "10447", "gengu", "spirit", "lunar", "mowen"),
    ("10026", "10026", "liliang", "strength", "physical", "aoxue"),
    ("10390", "10390", "shenfa", "agility", "physical", "fenshan"),
    ("10021", "10021", "yuanqi", "spunk", "neutral", "huajian"),
]
reference_data = None
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
        reference_data = json.load(source)

for school_id, source_id, major, upstream_major, kind, slug in rules:
    # Temporary synthetic catalog exercises each school through the real host. Never game gear.
    with tempfile.TemporaryDirectory(prefix="jpcg-attribute-") as folder:
        gear = Path(folder) / "gear"
        gear.mkdir()
        for filename in ("templates.toml", "schools.toml", "sets.toml", "overrides.toml"):
            content = (ROOT / "data/gear" / filename).read_text().replace('"mowen"', json.dumps(school_id))
            if filename == "schools.toml":
                content = content.replace('"gengu"', json.dumps(major))
                if kind == "physical":
                    content = content.replace('"neigong"', '"waigong"')
            (gear / filename).write_text(content)
        os.environ["JPCG_DATA_DIR"] = folder
        loadout = tomllib.loads((ROOT / "data/gear/sample-loadout.toml").read_text())
        loadout["school"] = school_id
        loadout["attribute_rules"] = f"jx3-130-{slug}-20260814"
        loadout["base"].update(tizhi=1001, yuanqi=1001, qixue_additional=123)
        result = call("compose", loadout)
        panel, raw, player = result["composed"]["panel"], result["composed"]["stats"], result["player"]
        require(panel["qixue"] == (raw["qixue"] + 10 * raw["tizhi"]) * 1331 // 1024 + 123, "HP stage mismatch")
        require(player["jichu_shuxing"] == panel[major], "primary alias dropped")
        require(player["huixin_dengji"] == panel["huixin"], "crit dropped")
        require(player["zhuanhua_gongji"] == panel["gongji_extra"], "extra attack dropped")
        require(player["jiasu_dengji"] == raw["jiasu"], "haste dropped")
        require(panel["damage_kind"] == kind, "damage channel lost")
        require(call("import", {"toml": call("export", loadout)}) == result, "TOML roundtrip mismatch")
        print(f"PASS actual gear cdylib {school_id}: HP stages/crit/attack/haste/channel/TOML", flush=True)

        if reference_data is not None:
            school = reference_data[source_id][source_id]["attributes"]
            rng = random.Random(98)
            for i in range(args.samples):
                loadout["base"] = {k: rng.randrange(1_000_001) for k in ("zhushuxing", "liliang", "shenfa", "gengu", "yuanqi", "tizhi", "gongji", "huixin", "pofang", "qixue", "qixue_additional")}
                result = call("compose", loadout)["composed"]
                raw, panel = result["stats"], result["panel"]
                x = BaseAttribute(upstream_major, kind, kind)
                for key, value in school.items():
                    if "_to_" in key or key == "max_life_gain":
                        setattr(x, key, value)
                for our, theirs in (("liliang", "strength"), ("shenfa", "agility"), ("gengu", "spirit"), ("yuanqi", "spunk"), ("tizhi", "vitality")):
                    setattr(x, theirs + "_base", raw[our] + (raw["zhushuxing"] if our == major else 0))
                setattr(x, kind + "_attack_power_base", raw["gongji"])
                setattr(x, kind + "_critical_strike_base", raw["huixin"])
                setattr(x, kind + "_overcome_base", raw["pofang"])
                x.max_life_base = raw["qixue"]
                x.max_life_add = raw["qixue_additional"]
                expected = (x.base_attack_power, getattr(x, "extra_" + kind + "_attack_power"), x.attack_power, x.final_critical_strike, x.final_overcome, x.max_life)
                actual = tuple(panel[k] for k in ("gongji_base", "gongji_extra", "gongji", "huixin", "pofang", "qixue"))
                require(actual == expected, f"school {school_id}, sample {i}: {raw}: {actual} != {expected}")
            print(f"PASS {school_id}: {args.samples} actual Rust/upstream samples, seed=98, source={PINNED}", flush=True)
