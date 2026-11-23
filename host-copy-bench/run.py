#!/usr/bin/env python3
"""Runs every host-copy-bench benchmark and packs the results into a tarball.

Usage, from anywhere in the wgpu repository:

    python3 host-copy-bench/run.py --name "MacBook Air M2, macOS 15.3"

Each benchmark group runs in its own process, one after another, so a driver
crash in one group doesn't lose the others. Extra arguments after `--` are
passed to the benchmark, for example `-- --adapter radeon`.
"""

import argparse
import datetime
import json
import os
import platform
import re
import subprocess
import sys
import tarfile
import tempfile
import time
from pathlib import Path

GROUPS = ["textures", "buffers", "scenarios", "barrier"]
TIMEOUT_SECONDS = 30 * 60


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def run_quiet(command):
    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=30)
        return result.stdout.strip()
    except (OSError, subprocess.SubprocessError):
        return ""


def system_info() -> dict:
    info = {
        "platform": platform.platform(),
        "machine": platform.machine(),
        "processor": platform.processor(),
        "python": platform.python_version(),
    }
    system = platform.system()
    if system == "Darwin":
        info["macos"] = run_quiet(["sw_vers", "-productVersion"])
        info["cpu"] = run_quiet(["sysctl", "-n", "machdep.cpu.brand_string"])
        info["model"] = run_quiet(["sysctl", "-n", "hw.model"])
        info["memory_bytes"] = run_quiet(["sysctl", "-n", "hw.memsize"])
    elif system == "Linux":
        try:
            info["os_release"] = Path("/etc/os-release").read_text()
        except OSError:
            pass
        try:
            cpuinfo = Path("/proc/cpuinfo").read_text()
            match = re.search(r"^model name\s*:\s*(.+)$", cpuinfo, re.MULTILINE)
            if match:
                info["cpu"] = match.group(1)
        except OSError:
            pass
        info["kernel"] = platform.release()
    elif system == "Windows":
        info["windows"] = platform.version()
    info["git_commit"] = run_quiet(["git", "-C", str(repo_root()), "rev-parse", "HEAD"])
    return info


def benchmark_env() -> dict:
    env = dict(os.environ)
    # The Vulkan loader on macOS is usually in /usr/local/lib, which the dynamic
    # linker doesn't search by default.
    if platform.system() == "Darwin" and "DYLD_FALLBACK_LIBRARY_PATH" not in env:
        if Path("/usr/local/lib/libvulkan.dylib").exists():
            env["DYLD_FALLBACK_LIBRARY_PATH"] = "/usr/local/lib"
    return env


def build() -> Path:
    print("Building host-copy-bench in release mode...", flush=True)
    subprocess.run(
        ["cargo", "build", "--release", "-p", "host-copy-bench"],
        cwd=repo_root(),
        check=True,
    )
    exe = "host-copy-bench.exe" if platform.system() == "Windows" else "host-copy-bench"
    target = Path(os.environ.get("CARGO_TARGET_DIR", repo_root() / "target"))
    path = target / "release" / exe
    if not path.exists():
        sys.exit(f"built binary not found at {path}")
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument(
        "--name",
        help='a description of the machine, e.g. "Dell XPS 15, RTX 4060, Windows 11"',
    )
    parser.add_argument("--runs", type=int, default=3, help="how many times to run everything")
    parser.add_argument(
        "--out", type=Path, default=Path.cwd(), help="directory to write the tarball to"
    )
    parser.add_argument("bench_args", nargs="*", help="extra arguments for the benchmark, after --")
    args = parser.parse_args()

    name = args.name
    if not name:
        try:
            name = input("Describe this machine (model, GPU, OS): ").strip()
        except EOFError:
            name = ""

    binary = build()
    env = benchmark_env()
    stamp = datetime.datetime.now().strftime("%Y%m%d-%H%M%S")
    manifest = {
        "name": name,
        "started": stamp,
        "system": system_info(),
        "bench_args": args.bench_args,
        "runs": [],
    }

    with tempfile.TemporaryDirectory() as tmp:
        results = Path(tmp) / f"host-copy-bench-{stamp}"
        results.mkdir()
        for run in range(1, args.runs + 1):
            for group in GROUPS:
                label = f"run{run}-{group}"
                skip = [arg for other in GROUPS if other != group for arg in ("--skip", other)]
                csv = results / f"{label}.csv"
                command = [str(binary), *skip, "--csv", str(csv), *args.bench_args]
                print(f"[{run}/{args.runs}] {group}...", flush=True)
                start = time.monotonic()
                entry = {"run": run, "group": group, "csv": csv.name}
                try:
                    result = subprocess.run(
                        command,
                        cwd=repo_root(),
                        env=env,
                        capture_output=True,
                        text=True,
                        timeout=TIMEOUT_SECONDS,
                    )
                    entry["exit_code"] = result.returncode
                    stdout, stderr = result.stdout, result.stderr
                except subprocess.TimeoutExpired as e:
                    entry["exit_code"] = "timeout"
                    stdout = e.stdout.decode() if isinstance(e.stdout, bytes) else (e.stdout or "")
                    stderr = e.stderr.decode() if isinstance(e.stderr, bytes) else (e.stderr or "")
                entry["seconds"] = round(time.monotonic() - start, 1)
                (results / f"{label}.md").write_text(stdout)
                (results / f"{label}.log").write_text(stderr)
                manifest["runs"].append(entry)
                if entry["exit_code"] != 0:
                    print(
                        f"  {group} failed ({entry['exit_code']}), see {label}.log in the tarball",
                        flush=True,
                    )

        (results / "manifest.json").write_text(json.dumps(manifest, indent=2))
        args.out.mkdir(parents=True, exist_ok=True)
        tarball = args.out / f"{results.name}.tar.gz"
        with tarfile.open(tarball, "w:gz") as tar:
            tar.add(results, arcname=results.name)

    failed = [r for r in manifest["runs"] if r["exit_code"] != 0]
    print()
    print(f"Wrote {tarball}")
    if failed:
        print(f"{len(failed)} of {len(manifest['runs'])} benchmark runs failed; the tarball has their logs.")
    print("Please send that file back.")


if __name__ == "__main__":
    main()
