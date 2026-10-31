#!/usr/bin/env python3
"""Profession TOML -> actual gear cdylib -> player DTO/TOML roundtrip.
Default fixtures use deliberately synthetic level50 coefficients, never game values.
Optional --reference-dir checks a pinned historical130 community model as a regression,
not as current rules; constants are read at runtime, not built into the Rust engine.
"""
import argparse
import ctypes
from fractions import Fraction
import json
import os
from pathlib import Path
import random
import shutil
import subprocess
import sys
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
PINNED = "21f472e211581ac1c1a7fa5ea2ff9199df6151cb"
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--reference-dir", type=Path)
parser.add_argument("--samples", type=int, default=1000)
args = parser.parse_args()
if args.samples < 1:
    parser.error("--samples must be positive")
name = "jpcg_gear.dll" if sys.platform == "win32" else "libjpcg_gear.dylib" if sys.platform == "darwin" else "libjpcg_gear.so"
lib = ctypes.CDLL(str(ROOT / "target/debug" / name))
lib.jpcg_gear_call.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
lib.jpcg_gear_call.restype = ctypes.c_void_p
lib.jpcg_gear_free_string.argtypes = [ctypes.c_void_p]
lib.jpcg_gear_free_string.restype = None


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def call(method, request, ok=True):
    pointer = lib.jpcg_gear_call(method.encode(), json.dumps(request).encode())
    require(pointer, "null gear result")
    try:
        result = json.loads(ctypes.string_at(pointer))
    finally:
        lib.jpcg_gear_free_string(pointer)
    require((result["status"] == "ok") == ok, result)
    return result.get("result")


def term(basic, target, n, d=1):
    return dict(from_=basic, to=target, numerator=n, denominator=d)


def table(rule):
    text = "\n[attributes]\n"
    for k in ("id", "level", "value_set", "source", "main_stat", "damage_kind"):
        text += f"{k} = {json.dumps(rule[k])}\n"
    n, d = rule["qixue_multiplier"]
    text += f"qixue_multiplier = {{ numerator = {n}, denominator = {d} }}\n"
    for t in rule["conversions"]:
        text += "\n[[attributes.conversions]]\n"
        for k, v in t.items():
            text += f"{k.removesuffix('_')} = {json.dumps(v)}\n"
    return text


schools = [("mowen", "10447", "gengu", "spirit", "lunar"),
           ("10026", "10026", "liliang", "strength", "physical"),
           ("10390", "10390", "shenfa", "agility", "physical"),
           ("10021", "10021", "yuanqi", "spunk", "neutral")]
reference_data = None
if args.reference_dir:
    reference = args.reference_dir.resolve()
    require(subprocess.check_output(["git", "-C", str(reference), "rev-parse", "HEAD"], text=True).strip() == PINNED, "Reference must be pinned")
    subprocess.run(["git", "-C", str(reference), "diff", "--exit-code", "HEAD", "--", "base", "tools", "assets/json/belongs.json"], check=True, stdout=subprocess.DEVNULL)
    sys.path.insert(0, str(reference))
    from base.attribute import BaseAttribute
    from base import constant
    require(constant.LEVEL == 130, "Historical reference level mismatch")
    reference_data = json.loads((reference / "assets/json/belongs.json").read_text())

for school_id, source_id, major, upstream_major, kind in schools:
    level, value_set = (130, "live-130") if reference_data else (50, "cszj-exp-260908")
    rule = dict(id=f"fixture-{school_id}-{level}", level=level, value_set=value_set,
                source="synthetic pipeline fixture; not game data", main_stat=major,
                damage_kind=kind, qixue_multiplier=(3, 2), conversions=[
                    term("tizhi", "qixue_base", 7), term(major, "gongji_extra", 5, 2),
                    term(major, "huixin", 9, 10), term(major, "huixin", 2, 5),
                    term("yuanqi", "gongji_base", 1, 5), term("liliang", "pofang", 3, 10)])
    if reference_data:
        school = reference_data[source_id][source_id]["attributes"]
        rule["source"] = "historical community model: " + PINNED
        rule["qixue_multiplier"] = (1024 + school["max_life_gain"], 1024)
        rule["conversions"] = []
        generic = [("tizhi", "qixue_base", constant.VITALITY_TO_MAX_LIFE)]
        generic += [("liliang", "gongji_base", constant.STRENGTH_TO_ATTACK_POWER), ("shenfa", "huixin", constant.AGILITY_TO_CRITICAL_STRIKE), ("liliang", "pofang", constant.STRENGTH_TO_OVERCOME)] if kind == "physical" else [("yuanqi", "gongji_base", constant.SPUNK_TO_ATTACK_POWER), ("gengu", "huixin", constant.SPIRIT_TO_CRITICAL_STRIKE), ("yuanqi", "pofang", constant.SPUNK_TO_OVERCOME)]
        for basic, target, coefficient in generic:
            f = Fraction(str(coefficient))
            rule["conversions"].append(term(basic, target, f.numerator, f.denominator))
        for our, theirs in (("liliang", "strength"), ("shenfa", "agility"), ("gengu", "spirit"), ("yuanqi", "spunk")):
            for target, stat in (("gongji_extra", "attack_power"), ("huixin", "critical_strike"), ("pofang", "overcome")):
                key = f"{theirs}_to_{kind}_{stat}"
                if key in school:
                    rule["conversions"].append(term(our, target, school[key], 1024))
    with tempfile.TemporaryDirectory(prefix="jpcg-attribute-") as folder:
        folder = Path(folder)
        for d in ("gear", "shuxing"):
            (folder / d).mkdir()
        shutil.copytree(ROOT / "data/values", folder / "values")
        for filename in ("templates.toml", "schools.toml", "sets.toml", "overrides.toml"):
            content = (ROOT / "data/gear" / filename).read_text().replace('"mowen"', json.dumps(school_id)).replace('"live-130"', json.dumps(value_set))
            if filename == "schools.toml":
                content = content.replace('"gengu"', json.dumps(major)).replace('"neigong"', '"waigong"' if kind == "physical" else '"neigong"')
            (folder / "gear" / filename).write_text(content)
        profession = folder / "shuxing" / (school_id + ".toml")
        prefix = f'[version]\nlevel = {level}\n[xinfa]\nxinfa_nom = "{major}"\n'
        profession.write_text(prefix + table(rule))
        os.environ["JPCG_DATA_DIR"] = str(folder)
        loadout = tomllib.loads((ROOT / "data/gear/sample-loadout.toml").read_text())
        loadout.update(school=school_id, value_set=value_set, attribute_rules="profession-toml")
        loadout["base"].update(tizhi=123, yuanqi=1001, qixue_additional=123)
        result = call("compose", loadout)
        raw, panel = result["composed"]["stats"], result["composed"]["panel"]
        require(panel["level"] == level and panel["value_set"] == value_set, "metadata lost")
        require(panel["damage_kind"] == kind and not panel["verified_in_game"], "channel/evidence marker lost")
        require(result["player"]["zhuanhua_gongji"] == panel["gongji_extra"], "double conversion marker lost")
        require(result["player"]["huixin_dengji"] == panel["huixin"], "derived crit lost")
        require(result["player"]["jiasu_dengji"] == raw["jiasu"], "haste lost")
        require(call("import", {"toml": call("export", loadout)}) == result, "TOML roundtrip mismatch")
        # Same file edits must take effect without rebuilding; missing table never falls back.
        profession.write_text(prefix)
        call("compose", loadout, ok=False)
        profession.write_text(prefix.replace(f"level = {level}", f"level = {level+1}") + table(rule))
        call("compose", loadout, ok=False)
        profession.write_text(prefix + table(rule))
        print(f"PASS profession TOML/{level}/{school_id}: stages, DTO, missing/mismatched rules, roundtrip", flush=True)
        if reference_data is not None:
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
                x.max_life_base, x.max_life_add = raw["qixue"], raw["qixue_additional"]
                expected = (x.base_attack_power, getattr(x, "extra_" + kind + "_attack_power"), x.attack_power, x.final_critical_strike, x.final_overcome, x.max_life)
                actual = tuple(panel[k] for k in ("gongji_base", "gongji_extra", "gongji", "huixin", "pofang", "qixue"))
                require(actual == expected, f"{school_id}/{i}: {actual} != {expected}")
            print(f"PASS historical regression {school_id}: {args.samples} Rust/upstream samples, seed=98; NOT current50 calibration", flush=True)
