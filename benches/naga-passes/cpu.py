#!/usr/bin/env python3
"""Run the CPU-only naga benchmark (WGSL parse, validate, process_overrides and the SPIR-V, HLSL,
MSL and GLSL writers) on two checkouts and compare them. No GPU is used.

    python benches/naga-passes/cpu.py BASE_CHECKOUT NEW_CHECKOUT [OUT_DIR [ROUNDS]]

On Windows use `py benches\\naga-passes\\cpu.py`. Python 3.9 or newer, standard library only.
The checkouts are the `trunk-bench` and `naga-passes-bench` branches. See README.md.
"""

import argparse
import datetime
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
EXE = ".exe" if os.name == "nt" else ""
BENCH = ["bench", "-p", "wgpu-benchmark", "--bench", "naga-passes-cpu"]


def cargo():
    found = shutil.which("cargo")
    if found:
        return found
    candidate = Path.home() / ".cargo" / "bin" / ("cargo" + EXE)
    if candidate.is_file():
        return str(candidate)
    sys.exit("cargo not found; install the Rust toolchain from https://rustup.rs")


def main():
    parser = argparse.ArgumentParser(
        description="Run the CPU-only naga benchmark on two checkouts and compare them.",
        usage="cpu.py BASE_CHECKOUT NEW_CHECKOUT [OUT_DIR [ROUNDS]]",
    )
    parser.add_argument("base", metavar="BASE_CHECKOUT")
    parser.add_argument("new", metavar="NEW_CHECKOUT")
    parser.add_argument("out", nargs="?", metavar="OUT_DIR")
    parser.add_argument("rounds", nargs="?", type=int, default=3, metavar="ROUNDS")
    args = parser.parse_args()

    base = Path(args.base).resolve()
    new = Path(args.new).resolve()
    out = Path(args.out) if args.out else (
        Path.cwd() / "naga-passes-cpu-results" / datetime.datetime.now().strftime("%Y%m%d-%H%M%S")
    )
    out.mkdir(parents=True, exist_ok=True)
    out = out.resolve()
    exe = cargo()

    for directory in (base, new):
        if subprocess.run([exe, *BENCH, "--no-run"], cwd=directory).returncode != 0:
            sys.exit(1)

    # Alternate the order each round so drift in machine speed does not favor one checkout.
    for round_number in range(1, args.rounds + 1):
        pair = [("trunk", base), ("combined", new)]
        if round_number % 2 == 0:
            pair.reverse()
        for label, directory in pair:
            print(f"== round {round_number}: {label}", flush=True)
            dest = out / label / "cpu" / f"r{round_number}"
            dest.mkdir(parents=True, exist_ok=True)
            env = dict(os.environ, NAGA_PASSES_OUT=str(dest))
            log = dest / "cpu_compile.log"
            with open(log, "wb") as f:
                status = subprocess.run(
                    [exe, *BENCH, "--", "naga_passes::cpu_compile"],
                    cwd=directory, env=env, stdout=f, stderr=subprocess.STDOUT,
                ).returncode
            if status != 0:
                print(f"   {label} failed; see {log}", flush=True)
                tail = log.read_text(errors="replace").strip().splitlines()[-3:]
                failure = {
                    "bench": "cpu_compile",
                    "label": label,
                    "backend": "cpu",
                    "attempts": 1,
                    "exit_status": status,
                    "deterministic": False,
                    "error": f"exit status {status}: " + " / ".join(t.strip() for t in tail),
                }
                (dest / "cpu_compile.failed.json").write_text(
                    json.dumps(failure, indent=1) + "\n", encoding="utf-8"
                )

    subprocess.run(
        [sys.executable, str(HERE / "compare.py"), str(out), "--base", "trunk", "--new", "combined"]
    )
    print(f"Results: {out}")


if __name__ == "__main__":
    main()
