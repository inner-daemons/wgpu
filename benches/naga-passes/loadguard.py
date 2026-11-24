"""Records how busy the machine is around each benchmark group, for run.py and cpu.py.

Other work on the machine skews timings, so before and after every group (one benchmark process
on one checkout) the guard reads the load and the time. "Load" is the average number of busy
cores: on Linux and macOS the 1 minute load average (`os.getloadavg()`), on Windows the CPU
load that PowerShell reports, times the number of cores. A group is marked `high_load` when the
load at the start or the end is above the limit, which by default is 1 + 0.25 * cores: the one
is the benchmark itself, and a quarter of the cores busy with other work is where timings stop
being comparable. Both ends of a group are checked because the load average lags: the end reading
still contains the benchmark and anything that ran during it.

The result goes into the `meta` of the group's result file (all values are strings) and into
`load-guard.txt` in the results directory, and a high load prints a warning. compare.py lists the
flagged groups.

If the load cannot be read (no `os.getloadavg`, no PowerShell), `high_load` is `unknown`.
"""

import contextlib
import datetime
import json
import os
import shutil
import subprocess
import sys
import time

DEFAULT_SETTLE_SECONDS = 30
POLL_SECONDS = 5


def cores():
    return os.cpu_count() or 1


def default_limit():
    return 1.0 + 0.25 * cores()


def _windows_busy_cores():
    powershell = shutil.which("powershell") or shutil.which("pwsh")
    if not powershell:
        return None
    try:
        result = subprocess.run(
            [powershell, "-NoProfile", "-Command",
             "(Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage "
             "-Average).Average"],
            capture_output=True, text=True, errors="replace", timeout=30,
        )
        return float(result.stdout.strip()) / 100.0 * cores()
    except (OSError, ValueError, subprocess.SubprocessError):
        return None


def read_load():
    """(busy cores, source) or (None, 'unavailable')."""
    if hasattr(os, "getloadavg"):
        try:
            return os.getloadavg()[0], "loadavg1"
        except OSError:
            pass
    if os.name == "nt":
        value = _windows_busy_cores()
        if value is not None:
            return value, "powershell"
    return None, "unavailable"


def now():
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def snapshot():
    load, source = read_load()
    return {"time": now(), "load": load, "source": source}


def _text(load):
    return "n/a" if load is None else f"{load:.2f}"


@contextlib.contextmanager
def group(out_dir, name, result_file, limit=None, settle=DEFAULT_SETTLE_SECONDS):
    """Wraps one benchmark group. `name` labels it in messages; `result_file` is the JSON file
    the group writes, whose meta gets the load fields afterwards.

    Before the group it waits up to `settle` seconds for the load to fall under the limit (a
    build or the previous group leaves the load average high for a minute or so)."""
    limit = default_limit() if limit is None else limit
    before = snapshot()
    waited = 0
    while (settle > 0 and before["load"] is not None and before["load"] > limit
           and waited < settle):
        if waited == 0:
            print(f"   load {_text(before['load'])} is above {limit:.2f}; waiting up to "
                  f"{settle} s for it to settle", flush=True)
        time.sleep(POLL_SECONDS)
        waited += POLL_SECONDS
        before = snapshot()
    yield
    after = snapshot()

    loads = [x["load"] for x in (before, after) if x["load"] is not None]
    if loads:
        high = "true" if max(loads) > limit else "false"
    else:
        high = "unknown"
    meta = {
        "started_utc": before["time"],
        "finished_utc": after["time"],
        "load_before": _text(before["load"]),
        "load_after": _text(after["load"]),
        "load_limit": f"{limit:.2f}",
        "load_cores": str(cores()),
        "load_source": before["source"],
        "high_load": high,
    }
    annotate(result_file, meta)
    line = (f"{name}: load {meta['load_before']} -> {meta['load_after']} "
            f"(limit {meta['load_limit']}, {meta['load_cores']} cores), "
            f"{meta['started_utc']} to {meta['finished_utc']}, high_load={high}")
    try:
        with open(os.path.join(out_dir, "load-guard.txt"), "a", encoding="utf-8") as f:
            f.write(line + "\n")
    except OSError:
        pass
    if high == "true":
        bar = "!" * 78
        print(f"{bar}\n!! HIGH LOAD during {name}: {meta['load_before']} -> "
              f"{meta['load_after']} busy cores (limit {meta['load_limit']}).\n"
              f"!! These timings are unreliable; the group is marked high_load in the results.\n"
              f"{bar}", file=sys.stderr, flush=True)
    elif high == "unknown":
        print(f"   note: cannot read the system load on this machine ({name})", flush=True)


def annotate(result_file, meta):
    """Add `meta` to the `meta` of a result file, if the file exists and is readable."""
    try:
        with open(result_file, encoding="utf-8") as f:
            data = json.load(f)
        data.setdefault("meta", {}).update(meta)
        tmp = str(result_file) + ".tmp"
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump(data, f)
        os.replace(tmp, result_file)
    except (OSError, ValueError):
        pass
