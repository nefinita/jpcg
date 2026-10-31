#!/usr/bin/env python3
"""真实 gear cdylib ABI + 宿主目录 + 合成/TOML 往返；不依赖桌面系统库。"""
import ctypes
import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib

root = Path(__file__).resolve().parent.parent
if len(sys.argv) == 1:
    for case in ("root", "shuxing", "missing"):
        subprocess.run([sys.executable, __file__, case], check=True)
    subprocess.run([sys.executable, str(root / "scripts/test-attributes-dynamic.py")], check=True)
    print("PASS gear dynamic: catalog, composition, TOML, strict path errors, attribute conversion")
    sys.exit(0)
case = sys.argv[1]
os.environ["JPCG_DATA_DIR"] = str(root / "data" / ("shuxing" if case == "shuxing" else "absent")) if case != "root" else str(root / "data")
filename = "jpcg_gear.dll" if sys.platform == "win32" else "libjpcg_gear.dylib" if sys.platform == "darwin" else "libjpcg_gear.so"
lib = ctypes.CDLL(str(root / "target/debug" / filename))
lib.jpcg_gear_call.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
lib.jpcg_gear_call.restype = ctypes.c_void_p
lib.jpcg_gear_free_string.argtypes = [ctypes.c_void_p]
lib.jpcg_gear_abi_version.restype = ctypes.c_uint32
assert lib.jpcg_gear_abi_version() == 1

def call(method, request):
    pointer = lib.jpcg_gear_call(method.encode() if method else None, json.dumps(request).encode())
    assert pointer
    try:
        return json.loads(ctypes.string_at(pointer))
    finally:
        lib.jpcg_gear_free_string(pointer)

assert call(None, {})["status"] == "error"
if case == "missing":
    assert call("catalog", {})["status"] == "error"
else:
    catalog = call("catalog", {})["result"]
    assert catalog["version"]["sample"] is True
    loadout = tomllib.loads((root / "data/gear/sample-loadout.toml").read_text())
    composition = call("compose", loadout)["result"]
    assert composition["player"]["jcsx"] == "根骨"
    assert composition["player"]["jichu_gongji"] == composition["composed"]["stats"]["gongji"]
    text = call("export", loadout)["result"]
    assert call("import", {"toml": text})["result"] == composition
    assert call("catalog", {"path": str(root)})["status"] == "error"
    loadout["revision"] = "old"
    assert call("compose", loadout)["status"] == "error"
    assert call("import", {"toml": "bad"})["status"] == "error"
print(f"PASS {case}")
