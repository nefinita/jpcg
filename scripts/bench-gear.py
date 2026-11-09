#!/usr/bin/env python3
"""#90 合成样例的 CLI release 微基准，不是全规模或真实配装验收。"""
import json
import os
from pathlib import Path
import statistics
import subprocess
import sys
import time
import tomllib

root = Path(__file__).resolve().parents[1]
binary = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else root / "target/release/gear-opt"
request = tomllib.loads((root / "data/gear/sample-request.toml").read_text())
environment = dict(os.environ, JPCG_DATA_DIR=str(root / "data"))
measurements = {}
for objective in ["expected", "kill_probability"]:
    if objective == "kill_probability":
        request["config"]["hostile"]["target_hp"] = 40000
        request["objective"] = {
            "kind": "kill_probability", "samples": 50000, "seed": 42,
            "allow_unapplied_cd": True,
            "steps": [{"skill": {"skill_name": "synthetic-strike", "base_damage1": 10000,
                                 "base_damage2": 10000, "atk_xishu": 1.0}}],
        }
    times = []
    scores = []
    for _ in range(5):
        start = time.perf_counter()
        result = subprocess.run(
            [str(binary), "--catalog", str(root / "data/gear"), "api"],
            input=json.dumps({"method": "optimize", "request": request}),
            text=True, capture_output=True, env=environment, check=True,
        )
        times.append((time.perf_counter() - start) * 1000)
        envelope = json.loads(result.stdout)
        if envelope["status"] != "ok":
            raise RuntimeError(envelope)
        response = envelope["result"]
        assert response["complete"] and response["progress"]["visited"] == 16
        assert response["best"]["score"] >= response["baseline"]["score"]
        scores.append(response["best"]["score"])
    assert len(set(scores)) == 1
    measurements[objective] = {
        "milliseconds": [round(t, 3) for t in times],
        "median_ms": round(statistics.median(times), 3),
        "baseline": response["baseline"]["score"], "best": scores[0],
        "combinations": response["progress"]["combinations"],
        "note": "synthetic data, fixed sequence; cooldown not applied",
    }
print(json.dumps(measurements, ensure_ascii=False, indent=2))
