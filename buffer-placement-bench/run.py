#!/usr/bin/env python3
"""Runs every buffer-placement-bench group and packs the results into a tarball.

Usage, from anywhere in the wgpu repository:

    python3 buffer-placement-bench/run.py

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

GROUPS = [
    "info",
    "upload",
    "write_texture",
    "staging_belt",
    "mappable_write",
    "mappable_readwrite",
    "controls",
    "bar_pressure",
]
TIMEOUT_SECONDS = 30 * 60


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def run_quiet(command):
    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=30)
        return result.stdout.strip()
    except (OSError, subprocess.SubprocessError):
        return ""


def pick_fields(text: str, keys) -> dict:
    """Keeps only the `Key: value` lines named in `keys`. Tools like
    system_profiler also print serial numbers, which must not be collected."""
    fields = {}
    for line in text.splitlines():
        key, sep, value = line.strip().partition(":")
        if sep and key in keys and value.strip():
            fields.setdefault(key, []).append(value.strip())
    return {k: v[0] if len(v) == 1 else v for k, v in fields.items()}


def powershell(command: str) -> str:
    return run_quiet(["powershell", "-NoProfile", "-NonInteractive", "-Command", command])


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
        info["model_identifier"] = run_quiet(["sysctl", "-n", "hw.model"])
        info["memory_bytes"] = run_quiet(["sysctl", "-n", "hw.memsize"])
        info["hardware"] = pick_fields(
            run_quiet(["system_profiler", "SPHardwareDataType"]),
            {"Model Name", "Chip", "Total Number of Cores", "Memory"},
        )
        info["gpus"] = pick_fields(
            run_quiet(["system_profiler", "SPDisplaysDataType"]),
            {"Chipset Model", "Vendor", "Total Number of Cores", "VRAM (Total)", "Metal Support"},
        )
        battery = run_quiet(["pmset", "-g", "batt"])
        info["power"] = battery.splitlines()[0] if battery else ""
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
        # Only the vendor and product name; the serial number is never read.
        for field in ("sys_vendor", "product_name", "product_version"):
            try:
                info[field] = Path(f"/sys/class/dmi/id/{field}").read_text().strip()
            except OSError:
                pass
        info["gpus"] = [
            line
            for line in run_quiet(["lspci"]).splitlines()
            if re.search(r"VGA|3D controller|Display controller", line)
        ]
        power = {}
        for supply in Path("/sys/class/power_supply").glob("*"):
            try:
                kind = (supply / "type").read_text().strip()
                if kind == "Mains":
                    power["ac_online"] = (supply / "online").read_text().strip() == "1"
                elif kind == "Battery":
                    power["battery_status"] = (supply / "status").read_text().strip()
            except OSError:
                pass
        info["power"] = power
    elif system == "Windows":
        info["windows"] = platform.version()
        info["os"] = powershell("(Get-CimInstance Win32_OperatingSystem).Caption")
        info["model"] = powershell(
            "$c = Get-CimInstance Win32_ComputerSystem; $c.Manufacturer + ' ' + $c.Model"
        )
        info["cpu"] = powershell("(Get-CimInstance Win32_Processor).Name")
        info["gpus"] = powershell(
            "Get-CimInstance Win32_VideoController | "
            "ForEach-Object { $_.Name + ', driver ' + $_.DriverVersion }"
        ).splitlines()
        # BatteryStatus 2 means on AC power; no battery means a desktop.
        info["battery_status"] = powershell("(Get-CimInstance Win32_Battery).BatteryStatus")
    info["git_commit"] = run_quiet(["git", "-C", str(repo_root()), "rev-parse", "HEAD"])
    return info


def describe(info: dict) -> str:
    """A one-line summary of the machine for the console."""
    parts = []
    hardware = info.get("hardware", {})
    for value in (
        hardware.get("Model Name") or info.get("model") or info.get("product_name"),
        hardware.get("Chip") or info.get("cpu"),
        info.get("os") or (f"macOS {info['macos']}" if info.get("macos") else None),
    ):
        if value:
            parts.append(str(value))
    return ", ".join(parts) or info["platform"]


def adapters_in(results: Path) -> list:
    """The GPUs the benchmark ran on, from the headings of its reports."""
    names = []
    for report in sorted(results.glob("run1-*.md")):
        for line in report.read_text(encoding="utf-8", errors="replace").splitlines():
            if line.startswith("## ") and line[3:] not in names:
                names.append(line[3:])
    return names


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
    print("Building buffer-placement-bench in release mode (this can take a few minutes)...", flush=True)
    # Cargo's progress goes to stderr, which is left attached so it's visible.
    result = subprocess.run(
        ["cargo", "build", "--release", "-p", "buffer-placement-bench", "--message-format=json"],
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
            and message.get("target", {}).get("name") == "buffer-placement-bench"
            and message.get("executable")
        ):
            return Path(message["executable"])
    sys.exit("Cargo didn't report where it built the benchmark.")


def scrub(text: str) -> str:
    """Removes the home directory and the repository path, which can appear in
    panic messages and logs."""
    for path, name in ((repo_root(), "<repo>"), (Path.home(), "~")):
        text = text.replace(str(path), name)
    return text


def write_text(path: Path, text: str):
    path.write_text(scrub(text), encoding="utf-8", errors="replace")


def decode(output) -> str:
    if output is None:
        return ""
    if isinstance(output, bytes):
        return output.decode("utf-8", errors="replace")
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument(
        "--note",
        "--name",
        dest="note",
        default="",
        help="anything worth knowing about the machine or the run, e.g. \"external GPU\"",
    )
    parser.add_argument("--runs", type=int, default=3, help="how many times to run everything")
    parser.add_argument(
        "--out", type=Path, default=Path.cwd(), help="directory to write the tarball to"
    )
    parser.add_argument("bench_args", nargs="*", help="extra arguments for the benchmark, after --")
    args = parser.parse_args()

    info = system_info()
    print(f"Machine: {describe(info)}", flush=True)
    binary = build()
    env = benchmark_env()
    stamp = datetime.datetime.now().strftime("%Y%m%d-%H%M%S")
    manifest = {
        "note": args.note,
        "started": stamp,
        "system": info,
        "bench_args": args.bench_args,
        "runs": [],
    }

    args.out.mkdir(parents=True, exist_ok=True)
    results = args.out / f"buffer-placement-bench-{stamp}"
    results.mkdir()
    interrupted = False
    try:
        for run in range(1, args.runs + 1):
            for group in GROUPS:
                label = f"run{run}-{group}"
                skip = [arg for other in GROUPS if other != group for arg in ("--skip", other)]
                csv = results / f"{label}.csv"
                command = [str(binary), *skip, "--csv", str(csv), *args.bench_args]
                if group == "info" and run == 1:
                    command += ["--memory-json", str(results / "memory-properties.json")]
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
                    print(f"  {group} failed ({entry['exit_code']}):", flush=True)
                    for line in scrub(stderr).strip().splitlines()[-10:]:
                        print(f"    {line}", flush=True)
    except KeyboardInterrupt:
        interrupted = True
        manifest["interrupted"] = True
        print("\nInterrupted; packing the results so far.", flush=True)

    manifest["adapters"] = adapters_in(results)
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
    if not manifest["adapters"]:
        print("No GPU was benchmarked, so the file has no results. The errors above say why.")
    else:
        print(f"Benchmarked: {', '.join(manifest['adapters'])}")
        print("Please send that file back.")
    if interrupted:
        sys.exit(130)
    if not manifest["adapters"]:
        sys.exit(1)


if __name__ == "__main__":
    main()
