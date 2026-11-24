#!/usr/bin/env python3
"""Build and run the driver benchmarks (pipeline creation and shader runtime) on two checkouts
and compare them. The CPU-only naga benchmark is separate: see benches/naga-passes/cpu.py.

    python benches/naga-passes/driver/run.py [options] [BASE_CHECKOUT NEW_CHECKOUT]

On Windows use `py benches\\naga-passes\\driver\\run.py`. Python 3.9 or newer, standard library only.

BASE_CHECKOUT and NEW_CHECKOUT are checkouts of the `trunk-bench` and `naga-passes-bench`
branches. If they are left out, the script makes git worktrees of those two branches (local or
on `origin`) in <repo>/target/naga-passes-worktrees. See benches/naga-passes/driver/README.md.
"""

import argparse
import datetime
import fnmatch
import json
import os
import platform
import re
import shutil
import socket
import subprocess
import sys
import tarfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import loadguard  # noqa: E402

REPO = HERE.parents[2]
DEFAULT_ROUNDS = 3
MIN_ROUNDS = 3
QUICK_WARNING = "QUICK RUN: a smoke check of the setup only. INVALID FOR COMPARISONS."
COMPARE = HERE.parent / "compare.py"
EXE = ".exe" if os.name == "nt" else ""
BENCH_MAIN = Path("benches") / "benches" / "naga-passes-driver" / "main.rs"
MAX_ATTEMPTS = 10
ANSI = re.compile(r"\x1b\[[0-9;]*m")


def say(*args):
    print(*args, flush=True)


def default_backends():
    if sys.platform == "darwin":
        return "metal"
    if os.name == "nt":
        return "dx12,vulkan"
    return "vulkan"


def find_tool(name):
    """The path of an executable, also in ~/.cargo/bin, which is not always on PATH."""
    found = shutil.which(name)
    if found:
        return found
    candidate = Path.home() / ".cargo" / "bin" / (name + EXE)
    return str(candidate) if candidate.is_file() else None


def capture(cmd, cwd=None, timeout=60):
    """The stdout of a command, or an empty string if it cannot run."""
    try:
        result = subprocess.run(
            cmd, cwd=cwd, capture_output=True, text=True, errors="replace", timeout=timeout
        )
    except (OSError, subprocess.SubprocessError):
        return ""
    return result.stdout.strip()


def git(*args, cwd=REPO):
    return subprocess.run(
        ["git", "-C", str(cwd), *args], capture_output=True, text=True, errors="replace"
    )


def rev(cwd, short=False):
    result = git("rev-parse", *(["--short"] if short else []), "HEAD", cwd=cwd)
    return result.stdout.strip() if result.returncode == 0 else "unknown"


def make_worktrees():
    root = REPO / "target" / "naga-passes-worktrees"
    root.mkdir(parents=True, exist_ok=True)
    dirs = {}
    for kind, branch in (("base", "trunk-bench"), ("new", "naga-passes-bench")):
        target = root / branch
        if not target.is_dir():
            ref = next(
                (
                    c
                    for c in (branch, f"origin/{branch}")
                    if git("rev-parse", "--verify", "--quiet", f"{c}^{{commit}}").returncode == 0
                ),
                None,
            )
            if ref is None:
                sys.exit(
                    f"branch {branch} not found; fetch it or pass the two checkouts as arguments"
                )
            result = git("-c", "core.longpaths=true", "worktree", "add", "--detach", str(target), ref)
            if result.returncode != 0:
                sys.exit(f"git worktree add failed:\n{result.stderr}")
        dirs[kind] = target
    return dirs["base"], dirs["new"]


def write_run_info(out, args, base_dir, new_dir, backends):
    lines = [
        "date: " + datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "uname: " + (capture(["uname", "-a"]) or " ".join(platform.uname())),
        "rustc: " + (capture([find_tool("rustc") or "rustc", "-V"]) or "unknown"),
        f"base: {args.base_label} {rev(base_dir)}",
        f"new: {args.new_label} {rev(new_dir)}",
        f"backends: {backends}",
        f"rounds: {args.rounds}",
        f"quick: {int(args.quick)}",
        f"load_limit: {args.max_load:.2f} busy cores ({loadguard.cores()} cores)",
    ]
    if args.quick:
        lines.append("quick_note: " + QUICK_WARNING)
    if sys.platform == "darwin":
        lines += capture(["sw_vers"]).splitlines()
    elif sys.platform.startswith("linux"):
        try:
            lines += Path("/etc/os-release").read_text(errors="replace").splitlines()[:5]
        except OSError:
            pass
    else:
        lines.append("os: " + platform.platform())
    lines += machine_info()
    (out / "run-info.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")


def machine_info():
    """`cpu:`, `gpu:` and `metal:` lines for the report, best effort."""
    lines = []
    if sys.platform == "darwin":
        cpu = capture(["sysctl", "-n", "machdep.cpu.brand_string"])
        if cpu:
            lines.append("cpu: " + cpu)
        try:
            data = json.loads(capture(["system_profiler", "SPDisplaysDataType", "-json"], timeout=30))
            for gpu in data.get("SPDisplaysDataType", []):
                name = gpu.get("sppci_model") or gpu.get("_name") or "unknown"
                cores = gpu.get("sppci_cores")
                lines.append("gpu: " + name + (f" ({cores} cores)" if cores else ""))
                metal = gpu.get("spdisplays_mtlgpufamilysupport", "")
                if metal:
                    lines.append("metal: " + re.sub(r"metal(\d+)", r"Metal \1", metal.replace("spdisplays_", "")))
        except (ValueError, AttributeError):
            pass
    elif os.name == "nt":
        powershell = shutil.which("powershell") or shutil.which("pwsh")
        if powershell:
            cpu = capture(
                [powershell, "-NoProfile", "-Command",
                 "(Get-CimInstance Win32_Processor | Select-Object -First 1).Name"]
            )
            if cpu:
                lines.append("cpu: " + cpu)
            video = capture(
                [powershell, "-NoProfile", "-Command",
                 "Get-CimInstance Win32_VideoController | "
                 "ForEach-Object { $_.Name + '|' + $_.DriverVersion }"]
            )
            for entry in video.splitlines():
                name, _, driver = entry.partition("|")
                if name.strip():
                    lines.append(f"gpu: {name.strip()}, driver {driver.strip()}")
    else:
        try:
            for line in Path("/proc/cpuinfo").read_text(errors="replace").splitlines():
                if line.lower().startswith("model name"):
                    lines.append("cpu: " + line.split(":", 1)[1].strip())
                    break
        except OSError:
            pass
    return lines


def build(out, label, directory, env):
    say(f"== building {label}")
    log = out / f"build-{label}.log"
    with open(log, "wb") as f:
        status = subprocess.run(
            [find_tool("cargo") or "cargo", "bench", "-p", "wgpu-benchmark", "--bench",
             "naga-passes-driver", "--no-run"],
            cwd=directory, env=env, stdout=f, stderr=subprocess.STDOUT,
        ).returncode
    if status != 0:
        print(f"build of {label} failed; see {log}", file=sys.stderr)
        tail = log.read_text(errors="replace").splitlines()[-20:]
        print("\n".join(tail), file=sys.stderr)
        sys.exit(1)


def failure_message(text):
    """The panic or error of a failed run: (signature that ignores ids and addresses, message)."""
    lines = [ANSI.sub("", line).rstrip() for line in text.splitlines()]
    message = None
    for i, line in enumerate(lines):
        if "panicked at" in line:
            where = line.split("panicked at", 1)[1].strip().rstrip(":")
            body = []
            for follow in lines[i + 1:i + 6]:
                if not follow.strip() or follow.startswith(("note:", "stack backtrace")):
                    break
                body.append(follow.strip())
            message = ("panic: " + " / ".join(body), f"panicked at {where}: " + " / ".join(body))
    if message is None:
        errors = [l.strip() for l in lines if l.strip().startswith(("error", "Error"))]
        tail = errors[-3:] or [l.strip() for l in lines if l.strip()][-3:]
        text = " / ".join(tail) or "no output"
        message = (text, text)
    signature = re.sub(r"0x[0-9a-fA-F]+|\(\d+\)|\d+(\.\d+)?\s?(ms|us|s)\b", "#", message[0])
    return signature[:400], message[1][:600]


def progress_marker(dest, bench):
    """What the result file holds, to tell a run that got further from one that did not."""
    try:
        data = json.loads((dest / f"{bench}.json").read_text(encoding="utf-8"))
        return len(data.get("records", [])), data.get("in_progress")
    except (OSError, ValueError):
        return None


def run_bench(env, label, directory, bench, dest, backend, extra=None):
    """Runs one benchmark; returns False if it failed for good.

    If the process dies, which is what a driver crash looks like, it is run again with
    NAGA_PASSES_RESUME=1, which records the case that was running as a crash and continues after
    it. A run that fails twice in a row with the same message and no new results is
    deterministic (a startup panic, say): it is not retried, and the message is recorded in
    `<bench>.failed.json`, which compare.py reports.
    """
    dest.mkdir(parents=True, exist_ok=True)
    log = dest / f"{bench}.log"
    attempt = 0
    previous = None
    while True:
        run_env = dict(env)
        run_env["NAGA_PASSES_OUT"] = str(dest)
        run_env.pop("NAGA_PASSES_RESUME", None)
        if attempt > 0:
            run_env["NAGA_PASSES_RESUME"] = "1"
        if backend:
            run_env["WGPU_BACKEND"] = backend
        run_env.update(extra or {})
        with open(log, "ab") as f:
            start = f.tell()
            f.flush()
            status = subprocess.run(
                [find_tool("cargo") or "cargo", "bench", "-p", "wgpu-benchmark", "--bench",
                 "naga-passes-driver", "--", f"naga_passes::{bench}"],
                cwd=directory, env=run_env, stdout=f, stderr=subprocess.STDOUT,
            ).returncode
        if status == 0:
            return True
        attempt += 1
        say(f"   {label} {bench} exited with status {status} (attempt {attempt})")
        with open(log, "rb") as f:
            f.seek(start)
            signature, message = failure_message(f.read().decode("utf-8", errors="replace"))
        state = (signature, progress_marker(dest, bench))
        if state == previous:
            reason = f"failed twice in a row with the same error: {message}"
            say(f"   {label} {bench} fails the same way each time; not retrying: {message}")
            record_failure(dest, bench, label, backend, attempt, status, reason, True)
            return False
        previous = state
        if attempt >= MAX_ATTEMPTS:
            reason = f"gave up after {attempt} attempts; last error: {message}"
            say(f"   giving up on {label} {bench}; partial results are kept")
            record_failure(dest, bench, label, backend, attempt, status, reason, False)
            return False


def record_failure(dest, bench, label, backend, attempts, status, error, deterministic):
    data = {
        "bench": bench,
        "label": label,
        "backend": dest.parent.name,
        "wgpu_backend": backend,
        "attempts": attempts,
        "exit_status": status,
        "deterministic": deterministic,
        "error": error,
    }
    (dest / f"{bench}.failed.json").write_text(json.dumps(data, indent=1) + "\n", encoding="utf-8")


def run_benchmarks(args, env, out, base_dir, new_dir, backends):
    def guarded(label, directory, bench, dest, backend, round_number, title, extra=None):
        say(f"== round {round_number}: {title} on {label} ({backend})")
        with loadguard.group(out, f"{label} {dest.parent.name} {bench} r{round_number}",
                             dest / f"{bench}.json", args.max_load, args.load_wait):
            run_bench(env, label, directory, bench, dest, backend, extra)

    for round_number in range(1, args.rounds + 1):
        # The checkout that goes first alternates, so drift in machine speed does not favor one.
        if round_number % 2 == 1:
            order = [(args.base_label, base_dir), (args.new_label, new_dir)]
        else:
            order = [(args.new_label, new_dir), (args.base_label, base_dir)]
        for backend in backends:
            for bench in ("pipeline_creation", "gpu_runtime"):
                for label, directory in order:
                    guarded(label, directory, bench,
                            out / label / backend / f"r{round_number}", backend,
                            round_number, bench)
            if backend == "vulkan":
                # The same shaders with naga's workgroup zero-init polyfill forced on, to compare
                # with the native mode that the run above uses when the device supports it.
                for label, directory in order:
                    guarded(label, directory, "gpu_runtime",
                            out / label / "vulkan-polyfill" / f"r{round_number}", "vulkan",
                            round_number, "gpu_runtime with polyfilled zero-init",
                            {"WGPU_BENCH_ONLY_FORCE_ZERO_INIT_POLYFILL": "1"})


def run_tests(env, out, label, directory):
    """The full test suite on a checkout, to tell new failures from existing ones."""
    tests = out / "tests"
    tests.mkdir(exist_ok=True)
    say(f"== cargo xtask test on {label} (long)")
    log = tests / f"{label}.log"
    test_env = dict(env, CARGO_TERM_COLOR="never", NO_COLOR="1")
    with open(log, "wb") as f:
        status = subprocess.run(
            [find_tool("cargo") or "cargo", "xtask", "test"],
            cwd=directory, env=test_env, stdout=f, stderr=subprocess.STDOUT,
        ).returncode
    say(f"   exit status {status} (see {log})")
    pattern = re.compile(r"^ *(FAIL|SIGABRT|SIGSEGV|SIGBUS|SIGILL|SIGTRAP|TIMEOUT|LEAK|FLAKY)")
    failures = set()
    for line in log.read_text(errors="replace").splitlines():
        line = ANSI.sub("", line)
        if pattern.match(line):
            line = re.sub(r"\[[^\]]*\]", "", line, count=1).lstrip()
            failures.add(re.sub(r" +", " ", line))
    path = tests / f"{label}-failures.txt"
    path.write_text("".join(f"{line}\n" for line in sorted(failures)), encoding="utf-8")
    say(f"   {len(failures)} distinct failing tests in {path}")
    return failures


def run_all_tests(args, env, out, base_dir, new_dir):
    base = run_tests(env, out, args.base_label, base_dir)
    new = run_tests(env, out, args.new_label, new_dir)
    lines = []
    for title, items in (
        (f"Failing on {args.new_label} but not on {args.base_label} (new failures):", new - base),
        (f"Failing on {args.base_label} but not on {args.new_label} (fixed):", base - new),
        ("Failing on both:", base & new),
    ):
        lines += [title, *sorted(items), ""]
    text = "\n".join(lines)
    (out / "tests" / "failure-diff.txt").write_text(text + "\n", encoding="utf-8")
    say(text)


def make_archive(out):
    archive = out.with_name(out.name + ".tar.gz")

    def keep(info):
        if fnmatch.fnmatch(os.path.basename(info.name), "build-*.log"):
            return None
        info.uid = info.gid = 0
        info.uname = info.gname = ""
        return info

    try:
        with tarfile.open(archive, "w:gz") as tar:
            tar.add(out, arcname=out.name, filter=keep)
    except (OSError, tarfile.TarError) as e:
        print(f"could not write {archive}: {e}", file=sys.stderr)
        return None
    return archive


def parse_args():
    parser = argparse.ArgumentParser(
        description="Build and run the driver benchmarks on two checkouts and compare them.",
        usage="run.py [options] [BASE_CHECKOUT NEW_CHECKOUT]",
        epilog="BASE_CHECKOUT and NEW_CHECKOUT are the trunk-bench and naga-passes-bench "
        "branches. Without them, worktrees of both are made in "
        "<repo>/target/naga-passes-worktrees.",
    )
    parser.add_argument("checkouts", nargs="*", metavar="CHECKOUT", help=argparse.SUPPRESS)
    parser.add_argument("--out", help="Where to write results (default: "
                        "./naga-passes-results/<date>-<host>).")
    parser.add_argument("--backends", default="",
                        help="Comma separated wgpu backends for the device benchmarks (default: "
                        "metal on macOS, vulkan on Linux, dx12,vulkan on Windows). Add `gl` to "
                        "include the GLES backend.")
    parser.add_argument("--rounds", type=int, default=None,
                        help="Run each benchmark N times per checkout, alternating which "
                        f"checkout goes first (default {DEFAULT_ROUNDS}, or 1 with --quick). "
                        f"Fewer than {MIN_ROUNDS} rounds is not enough for a comparison.")
    parser.add_argument("--quick", action="store_true",
                        help="A smoke check of the setup: 2 samples per pipeline, small "
                        "dispatches. INVALID FOR COMPARISONS; the results are marked as such.")
    parser.add_argument("--max-load", type=float, default=None, metavar="CORES",
                        help="Mark a benchmark group as high_load if the load (busy cores, "
                        "from the 1 minute load average) at its start or end is above this "
                        "(default: 1 + 0.25 * the number of cores).")
    parser.add_argument("--load-wait", type=int, default=loadguard.DEFAULT_SETTLE_SECONDS,
                        metavar="SECONDS",
                        help="Before each group, wait up to this long for the load to fall "
                        f"under the limit (default {loadguard.DEFAULT_SETTLE_SECONDS}; 0 "
                        "does not wait).")
    parser.add_argument("--no-info", action="store_true",
                        help="Skip collecting adapter information with wgpu-info.")
    parser.add_argument("--full-tests", action="store_true",
                        help="Also run `cargo xtask test` in both checkouts and compare the "
                        "failures. Long (see the README); needs cargo-nextest and, on most "
                        "OSes, the Vulkan SDK.")
    parser.add_argument("--no-bench", action="store_true",
                        help="Skip the benchmarks (use with --full-tests to run only the tests).")
    parser.add_argument("--labels", default="trunk,combined", metavar="A,B",
                        help="Names for the two checkouts in the report (default trunk,combined).")
    args = parser.parse_args()
    if len(args.checkouts) not in (0, 2):
        parser.error("pass either no checkouts or both BASE_CHECKOUT and NEW_CHECKOUT")
    if args.rounds is None:
        args.rounds = 1 if args.quick else DEFAULT_ROUNDS
    if args.rounds < 1:
        parser.error("--rounds must be at least 1")
    if args.max_load is None:
        args.max_load = loadguard.default_limit()
    labels = [x.strip() for x in args.labels.split(",")]
    if len(labels) != 2 or not all(labels):
        parser.error("--labels takes two names, A,B")
    args.base_label, args.new_label = labels
    return args


def main():
    for stream in (sys.stdout, sys.stderr):
        if hasattr(stream, "reconfigure"):
            stream.reconfigure(errors="replace")
    args = parse_args()

    backends = [b.strip() for b in (args.backends or default_backends()).split(",") if b.strip()]
    if args.out:
        out = Path(args.out)
    else:
        host = socket.gethostname().split(".")[0] or "host"
        stamp = datetime.datetime.now().strftime("%Y%m%d-%H%M%S")
        out = Path.cwd() / "naga-passes-results" / f"{stamp}-{host}"
    out.mkdir(parents=True, exist_ok=True)
    out = out.resolve()

    if find_tool("cargo") is None:
        sys.exit("cargo not found; install the Rust toolchain from https://rustup.rs")
    if args.checkouts:
        base_dir, new_dir = (Path(c).resolve() for c in args.checkouts)
    else:
        if shutil.which("git") is None:
            sys.exit("git not found")
        base_dir, new_dir = make_worktrees()
    for directory in (base_dir, new_dir):
        if not (directory / BENCH_MAIN).is_file():
            sys.exit(f"{directory} does not contain the naga pass benchmarks")

    # Disk caches of compiled shaders would hide the compile time of repeated pipelines.
    env = dict(os.environ)
    env.update(
        MESA_SHADER_CACHE_DISABLE="true",
        MESA_GLSL_CACHE_DISABLE="true",
        __GL_SHADER_DISK_CACHE="0",
        RUST_BACKTRACE="1",
    )
    if args.quick:
        env.update(
            NAGA_PASSES_PIPELINE_ITERS="2",
            NAGA_PASSES_GPU_GROUPS="256",
            NAGA_PASSES_GPU_RUNS="5",
            NAGA_PASSES_GPU_WARMUP="1",
        )

    if args.quick:
        say("!" * 78)
        say(f"!! {QUICK_WARNING}")
        say("!" * 78)
    elif args.rounds < MIN_ROUNDS and not args.no_bench:
        say(f"warning: {args.rounds} round(s); use at least {MIN_ROUNDS} for a comparison")
    say(f"base ({args.base_label}): {base_dir} ({rev(base_dir, short=True)})")
    say(f"new  ({args.new_label}): {new_dir} ({rev(new_dir, short=True)})")
    say(f"results: {out}")
    write_run_info(out, args, base_dir, new_dir, ",".join(backends))

    run_bench_step = not args.no_bench
    if run_bench_step:
        build(out, args.base_label, base_dir, env)
        build(out, args.new_label, new_dir, env)

    if not args.no_info:
        say("== collecting adapter information")
        with open(out / "wgpu-info.txt", "wb") as f:
            status = subprocess.run(
                [find_tool("cargo") or "cargo", "run", "--release", "-p", "wgpu-info"],
                cwd=new_dir, env=env, stdout=f, stderr=subprocess.STDOUT,
            ).returncode
        if status != 0:
            say(f"wgpu-info failed; see {out / 'wgpu-info.txt'}")

    if run_bench_step:
        run_benchmarks(args, env, out, base_dir, new_dir, backends)

    if args.full_tests:
        run_all_tests(args, env, out, base_dir, new_dir)

    say("== comparing")
    if run_bench_step:
        result = subprocess.run(
            [sys.executable, str(COMPARE), str(out), "--base", args.base_label,
             "--new", args.new_label],
            capture_output=True, text=True, errors="replace",
        )
        sys.stderr.write(result.stderr)
        say(result.stdout)
        (out / "comparison.txt").write_text(result.stdout, encoding="utf-8")
    say("")
    if args.quick:
        say(QUICK_WARNING)
    say(f"Results and logs are in {out}")
    say("Send back: the directory (without the build logs if large), or the .tar.gz below.")
    archive = make_archive(out)
    if archive:
        say(f"Archive: {archive}")


if __name__ == "__main__":
    main()
