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
import shutil
import subprocess
import sys
import tarfile
import time
from pathlib import Path

if sys.version_info < (3, 8):
    sys.exit("This script needs Python 3.8 or later.")

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
    if shutil.which("cargo") is None:
        sys.exit("cargo was not found. Install Rust from https://rustup.rs and try again.")
    print("Building host-copy-bench in release mode (this can take a few minutes)...", flush=True)
    # Cargo's progress goes to stderr, which is left attached so it's visible.
    result = subprocess.run(
        ["cargo", "build", "--release", "-p", "host-copy-bench", "--message-format=json"],
        cwd=repo_root(),
        stdout=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        sys.exit("Building the benchmark failed; the error is above.")
    for line in result.stdout.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if (
            message.get("reason") == "compiler-artifact"
            and message.get("target", {}).get("name") == "host-copy-bench"
            and message.get("executable")
        ):
            return Path(message["executable"])
    sys.exit("Cargo didn't report where it built the benchmark.")


def write_text(path: Path, text: str):
    path.write_text(text, encoding="utf-8", errors="replace")


def decode(output) -> str:
    if output is None:
        return ""
    if isinstance(output, bytes):
        return output.decode("utf-8", errors="replace")
    return output


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

    args.out.mkdir(parents=True, exist_ok=True)
    results = args.out / f"host-copy-bench-{stamp}"
    results.mkdir()
    interrupted = False
    try:
        for run in range(1, args.runs + 1):
            for group in GROUPS:
                label = f"run{run}-{group}"
                skip = [arg for other in GROUPS if other != group for arg in ("--skip", other)]
                csv = results / f"{label}.csv"
                command = [str(binary), *skip, "--csv", str(csv), *args.bench_args]
                print(f"[run {run}/{args.runs}] {group}...", flush=True)
                start = time.monotonic()
                entry = {"run": run, "group": group, "csv": csv.name}
                try:
                    result = subprocess.run(
                        command,
                        cwd=repo_root(),
                        env=env,
                        capture_output=True,
                        text=True,
                        encoding="utf-8",
                        errors="replace",
                        timeout=TIMEOUT_SECONDS,
                    )
                    entry["exit_code"] = result.returncode
                    stdout, stderr = result.stdout, result.stderr
                except subprocess.TimeoutExpired as e:
                    entry["exit_code"] = "timeout"
                    stdout, stderr = decode(e.stdout), decode(e.stderr)
                entry["seconds"] = round(time.monotonic() - start, 1)
                write_text(results / f"{label}.md", stdout)
                write_text(results / f"{label}.log", stderr)
                manifest["runs"].append(entry)
                if entry["exit_code"] != 0:
                    print(
                        f"  {group} failed ({entry['exit_code']}); its log is in the tarball",
                        flush=True,
                    )
    except KeyboardInterrupt:
        interrupted = True
        manifest["interrupted"] = True
        print("\nInterrupted; packing the results so far.", flush=True)

    write_text(results / "manifest.json", json.dumps(manifest, indent=2))
    tarball = args.out / f"{results.name}.tar.gz"
    with tarfile.open(tarball, "w:gz") as tar:
        tar.add(results, arcname=results.name)
    shutil.rmtree(results)

    failed = [r for r in manifest["runs"] if r["exit_code"] != 0]
    print()
    print(f"Wrote {tarball}")
    if failed:
        print(f"{len(failed)} of {len(manifest['runs'])} benchmark runs failed; the tarball has their logs.")
    print("Please send that file back.")
    if interrupted:
        sys.exit(130)


if __name__ == "__main__":
    main()
