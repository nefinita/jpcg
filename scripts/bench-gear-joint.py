#!/usr/bin/env python3
"""完整联合搜索的端到端测速；一次只运行一个后端，不含编译耗时。"""
import argparse
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time


def main():
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--backend", required=True, help="cpu 或 backends 列表中的单个 wgpu:N")
    parser.add_argument("--request", type=Path,
                        default=root / "scripts/benchmarks/gear-full-20261117.json")
    parser.add_argument("--binary", type=Path, default=root / "target/release/gear-opt")
    parser.add_argument("--output", type=Path, help="新建结果目录，默认 .scratch/gear-full/<时间-后端-pid>")
    args = parser.parse_args()
    if not re.fullmatch(r"cpu|wgpu:\d+", args.backend):
        parser.error("--backend 必须是 cpu 或 wgpu:N")
    binary = args.binary.resolve()
    if not binary.is_file():
        parser.error("请先运行 cargo build --release --locked -p jpcg_gear --bin gear-opt")
    request = json.loads(args.request.read_text())
    request["compute"] = args.backend
    output = args.output or root / ".scratch/gear-full" / (
        time.strftime("%Y%m%d-%H%M%S-") + args.backend.replace(":", "-") + f"-{os.getpid()}"
    )
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    request_path = output / "request.json"
    request_path.write_text(json.dumps(request, ensure_ascii=False, indent=2) + "\n")
    environment = dict(os.environ)
    environment.setdefault("JPCG_DATA_DIR", str(root / "data"))
    environment["JPCG_DATA_DIR"] = str(Path(environment["JPCG_DATA_DIR"]).resolve())
    command = [str(binary), "--catalog", str(Path(environment["JPCG_DATA_DIR"]) / "gear"),
               "joint", "--request", str(request_path)]
    print(f"后端 {args.backend}，完整搜索；日志和结果：{output}", flush=True)
    interrupted = False
    with (output / "result.json").open("w") as result_file, \
            (output / "progress.log").open("w") as log:
        started = time.perf_counter()
        process = subprocess.Popen(command, cwd=root, env=environment, stdout=result_file,
                                   stderr=subprocess.PIPE, text=True)
        try:
            for line in process.stderr:
                log.write(line)
                log.flush()
                sys.stderr.write(line)
            process.wait()
        except KeyboardInterrupt:
            # 终端通常已向整个进程组发 SIGINT；再次设置 CLI 取消标记也安全。
            interrupted = True
            if process.poll() is None:
                process.send_signal(signal.SIGINT)
            _, remaining_stderr = process.communicate()
            if remaining_stderr:
                log.write(remaining_stderr)
                sys.stderr.write(remaining_stderr)
        elapsed = time.perf_counter() - started
    if process.returncode != 0:
        print(f"运行失败（退出码 {process.returncode}），请查看 {output / 'progress.log'}", file=sys.stderr)
        return 1
    result = json.loads((output / "result.json").read_text())
    progress = result["progress"]
    summary = {
        "backend": args.backend,
        "elapsed_seconds": elapsed,
        "complete": result["complete"],
        "stop_reason": result["stop_reason"],
        "progress": progress,
        "mc_evaluations": result["mc_evaluations"],
        "evaluated_per_second": progress["evaluated"] / elapsed,
        "best_score": result["best"]["score"] if result["best"] else None,
        "baseline_score": result["baseline"]["score"],
        "data_dir": environment["JPCG_DATA_DIR"],
        "interrupted": interrupted,
    }
    text = json.dumps(summary, ensure_ascii=False, indent=2)
    (output / "summary.json").write_text(text + "\n")
    print(text)
    # 被中断的部分结果仍保存，但不能作为全量测速成功。
    return 0 if result["complete"] and result["stop_reason"] == "exhausted" else 1


if __name__ == "__main__":
    sys.exit(main())
