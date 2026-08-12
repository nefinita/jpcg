#!/usr/bin/env python3
"""JPCG ctypes demo — 直接连接 libjpcg_core.{dylib,so,dll}，无需 Tauri。

用法：
    cargo build -p jpcg_core          # 先生成 dylib
    python3 examples/python_demo/jpcg_demo.py [lib路径]

协议（见 crates/jpcg_core/src/ffi.rs）：
    jpcg_handle_create(session_config) -> handle
    jpcg_call(handle, method, request_json) -> response_json | NULL(出错)
    jpcg_last_error() -> 错误字符串（需 jpcg_free_string 释放）
    jpcg_free_string(s)
    jpcg_handle_free(handle)
    jpcg_abi_version() -> u32
"""

import ctypes
import json
import os
import sys

LIB_NAMES = {
    "darwin": "libjpcg_core.dylib",
    "linux": "libjpcg_core.so",
    "win32": "jpcg_core.dll",
}


def find_lib() -> str:
    candidates = []
    if len(sys.argv) > 1:
        candidates.append(sys.argv[1])
    name = LIB_NAMES.get(sys.platform)
    if name:
        candidates.append(f"target/debug/{name}")
        candidates.append(f"target/release/{name}")
    for c in candidates:
        if os.path.isfile(c):
            return c
    raise SystemExit(
        f"找不到 core 库，请先执行 cargo build -p jpcg_core，或传入库路径参数"
    )


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


def main():
    lib_path = find_lib()
    print(f"==> 加载 {lib_path}")
    core = Core(lib_path)
    try:
        print(f"==> abi_version = {core.lib.jpcg_abi_version()}")

        profs = core.call("list_professions", {})
        print(f"==> list_professions -> {profs}")

        req = {
            "player": {
                "jcsx": "gengu", "jichu_shuxing": 18888, "jichu_gongji": 4666,
                "huixin_dengji": 33000, "huixin_xiaoguo": 22000,
                "pofang_dengji": 25000, "wuqi_shanghai": 2800,
            },
            "hostile": {
                "waigong_fangyu": 21000, "neigong_fangyu": 21000,
                "yujin_dengji": 8500, "huajin_dengji": 35000,
                "jianshang_bili": 35, "target_hp": 200,
            },
            "xinfa_config": {
                "profession": "mowen", "xinfa_name": "莫问", "xinfa_nom": "根骨",
                "atk_up": 1.96, "pofang_up": 2.0, "huixin_up": 0.0,
            },
            "buff": {
                "base_atk_pct": 0.0, "huixin_pct": 0.0, "huixiao_pct": 0.0,
                "pofang_pct": 0.0, "wushi_fangyu_pct": 0.0,
                "shanghai_pct": 0.0, "mode_is_point": True,
            },
            "coefficient": {
                "pofang_xishu": 225957.6, "huixin_xishu": 197703.0,
                "huixiao_xishu": 3970.0, "huajin_xishu": 107361.7,
                "fangyu_xishu": 2453.5, "pvp_global_jianshang": 0.0,
            },
        }
        try:
            results = core.call("calculate", req)
            print(f"==> calculate -> {len(results)} 个技能")
            for r in results[:3]:
                print(f"    {r['skill_name']}: 伤害y={r['y']} b={r['b']} h={r['h']} q={r['q']}")
        except RuntimeError as e:
            print(f"    (数据目录缺失时预期失败) {e}")
    finally:
        core.close()
        print("==> 句柄已释放")


if __name__ == "__main__":
    main()
