#!/usr/bin/env python3
"""Compare the results of two checkouts of the naga pass benchmarks.

    compare.py RESULTS_DIR [--base trunk] [--new combined]

RESULTS_DIR holds <label>/<backend>/r<round>/<benchmark>.json files, as written by run.py and
cpu.py, and <benchmark>.failed.json files for benchmarks that could not run. Writes
comparison.md and comparison.csv into RESULTS_DIR and prints the markdown.

Statistics: each case's number is its minimum over all samples and rounds (the median of the
per-round medians is shown for reference). Summaries are geometric means of the per-case
new/base ratios, not ratios of sums.
"""

import argparse
import csv
import glob
import math
import os
import statistics
import sys
from collections import defaultdict

import json

CPU_METRICS = ["parse", "validate", "overrides", "spv", "spv_polyfill", "hlsl", "msl", "glsl", "total"]
CPU_BACKENDS = ["spv", "hlsl", "msl", "glsl"]


def load(root, label):
    """Returns {(bench, backend, case, metric): [record per round]}, per-run metadata, the
    failures recorded for benchmarks that could not run and the load guard fields of every
    benchmark group."""
    records = defaultdict(list)
    meta = {}
    failures = []
    guards = []
    for path in sorted(glob.glob(os.path.join(root, label, "*", "r*", "*.json"))):
        backend = path.split(os.sep)[-3]
        if path.endswith(".failed.json"):
            try:
                with open(path) as f:
                    failure = json.load(f)
            except (OSError, ValueError) as e:
                print(f"warning: cannot read {path}: {e}", file=sys.stderr)
                continue
            failure["label"] = label
            failure["backend"] = backend
            failure["round"] = path.split(os.sep)[-2]
            failures.append(failure)
            continue
        try:
            with open(path) as f:
                data = json.load(f)
        except (OSError, ValueError) as e:
            print(f"warning: cannot read {path}: {e}", file=sys.stderr)
            continue
        bench = data.get("bench", os.path.basename(path)[:-5])
        meta[(bench, backend)] = data.get("meta", {})
        if "high_load" in data.get("meta", {}):
            guards.append(dict(data["meta"], label=label, backend=backend, bench=bench,
                               round=path.split(os.sep)[-2]))
        for record in data.get("records", []):
            records[(bench, backend, record["case"], record["metric"])].append(record)
    return records, meta, failures, guards


def run_info(root):
    """The `key: value` lines of run-info.txt (the first of each key)."""
    info = {}
    try:
        with open(os.path.join(root, "run-info.txt"), encoding="utf-8", errors="replace") as f:
            for line in f:
                key, sep, value = line.partition(":")
                if sep and key.strip() and key.strip() not in info:
                    info[key.strip()] = value.strip()
                if sep and key.strip() == "gpu":
                    info.setdefault("gpus", []).append(value.strip())
    except OSError:
        pass
    return info


def driver_text(meta, backend, info):
    """The driver of an adapter. Metal and DX12 adapters report none, so use the OS and GPU
    details that run.py recorded."""
    driver = " ".join(x for x in (meta.get("driver", ""), meta.get("driver_info", "")) if x)
    if driver:
        return driver
    name = meta.get("backend", backend).lower()
    if name == "metal" or backend == "metal":
        parts = []
        if info.get("ProductVersion"):
            parts.append(f"macOS {info['ProductVersion']} ({info.get('BuildVersion', '?')})")
        if info.get("metal"):
            parts.append(info["metal"])
        return ", ".join(parts) or "n/a"
    for gpu in info.get("gpus", []):
        gpu_name, _, version = gpu.partition(", driver ")
        if version and gpu_name and gpu_name.lower() in meta.get("adapter_name", "").lower():
            return version
    return "n/a"


def merge(recs):
    """One record from the rounds of a measurement if all rounds are ok: the minimum of the
    minimums, the maximum of the maximums, and the median of the other statistics."""
    bad = [r for r in recs if r["status"] != "ok"]
    if bad:
        return dict(bad[0])
    out = dict(recs[0])
    for key in ("median", "mean", "p90"):
        out[key] = statistics.median(r[key] for r in recs)
    out["min"] = min(r["min"] for r in recs)
    out["max"] = max(r["max"] for r in recs)
    out["n"] = sum(r["n"] for r in recs)
    out["rounds"] = len(recs)
    return out


def geomean(values):
    values = list(values)
    return math.exp(sum(math.log(v) for v in values) / len(values)) if values else None


def ratio_pct(ratio):
    return "n/a" if ratio is None else f"{(ratio - 1) * 100:+.1f}%"


class Ratios:
    """Per-case (base, new) pairs of the minimum and the median, for geometric means."""

    def __init__(self):
        self.fields = {"min": [], "median": []}

    def add(self, b, n):
        for field, pairs in self.fields.items():
            if b[field] > 0 and n[field] > 0:
                pairs.append((b[field], n[field]))

    def count(self):
        return len(self.fields["min"])

    def change(self, field):
        pairs = self.fields[field]
        return geomean(n / b for b, n in pairs)

    def base_new(self, field):
        pairs = self.fields[field]
        return geomean(b for b, _ in pairs), geomean(n for _, n in pairs)


def pct(base, new):
    if base == 0:
        return "n/a" if new == 0 else "inf"
    return f"{(new - base) / base * 100:+.1f}%"


def num(value, unit):
    if unit == "us":
        return f"{value:,.0f}"
    return f"{value:,.3f}"


def table(headers, rows):
    if not rows:
        return "_no data_\n"
    lines = ["| " + " | ".join(headers) + " |", "|" + "|".join(["---"] * len(headers)) + "|"]
    for row in rows:
        lines.append("| " + " | ".join(str(c) for c in row) + " |")
    return "\n".join(lines) + "\n"


class Report:
    def __init__(self, base, new, base_label, new_label, base_meta=None, new_meta=None,
                 base_failures=(), new_failures=(), guards=()):
        self.guards = list(guards)
        self.base = base
        self.new = new
        self.base_meta = base_meta or {}
        self.new_meta = new_meta or {}
        self.base_failures = list(base_failures)
        self.new_failures = list(new_failures)
        self.base_label = base_label
        self.new_label = new_label
        self.out = []
        self.csv_rows = []

    def add(self, text=""):
        self.out.append(text)

    def pairs(self, bench, backend):
        """The (case, metric) keys with a measurement in both, merged over rounds."""
        keys = sorted(
            k[2:]
            for k in set(self.base) | set(self.new)
            if k[0] == bench and k[1] == backend
        )
        for case, metric in keys:
            b = self.base.get((bench, backend, case, metric))
            n = self.new.get((bench, backend, case, metric))
            yield case, metric, (merge(b) if b else None), (merge(n) if n else None)

    def backends(self, bench):
        found = {k[1] for k in list(self.base) + list(self.new) if k[0] == bench}
        return sorted(found)

    def csv_row(self, bench, backend, case, metric, b, n):
        rec = b or n
        row = {
            "bench": bench,
            "backend": backend,
            "group": rec.get("group", ""),
            "case": case,
            "metric": metric,
            "unit": rec.get("unit", ""),
            "base_status": b["status"] if b else "missing",
            "new_status": n["status"] if n else "missing",
            "base_median": b["median"] if b and b["status"] == "ok" else "",
            "new_median": n["median"] if n and n["status"] == "ok" else "",
            "base_p90": b["p90"] if b and b["status"] == "ok" else "",
            "new_p90": n["p90"] if n and n["status"] == "ok" else "",
            "base_n": b["n"] if b else "",
            "new_n": n["n"] if n else "",
        }
        if b and n and b["status"] == "ok" and n["status"] == "ok":
            row["abs_change"] = n["median"] - b["median"]
            row["pct_change"] = (
                (n["median"] - b["median"]) / b["median"] * 100 if b["median"] else ""
            )
        else:
            row["abs_change"] = row["pct_change"] = ""
        ok = b and n and b["status"] == "ok" and n["status"] == "ok"
        row["base_min"] = b["min"] if b and b["status"] == "ok" else ""
        row["new_min"] = n["min"] if n and n["status"] == "ok" else ""
        row["pct_change_min"] = (n["min"] - b["min"]) / b["min"] * 100 if ok and b["min"] else ""
        row["base_error"] = (b or {}).get("error") or ""
        row["new_error"] = (n or {}).get("error") or ""
        self.csv_rows.append(row)

    # -- sections --

    def summary(self, acc, metrics, unit):
        """Rows of geometric means. `acc` maps a metric to its Ratios."""
        rows = []
        for metric in metrics:
            if metric in acc and acc[metric].count():
                r = acc[metric]
                base_min, new_min = r.base_new("min")
                rows.append([metric, r.count(), num(base_min, unit), num(new_min, unit),
                             ratio_pct(r.change("min")), ratio_pct(r.change("median"))])
        return table(["metric", "cases", f"{self.base_label} min", f"{self.new_label} min",
                      "change (min)", "change (median)"], rows)

    def cpu(self):
        bench = "cpu_compile"
        backends = self.backends(bench)
        if not backends:
            return
        self.add("## CPU compile pipeline (naga only)\n")
        self.add(
            "One iteration is parse, validate, process_overrides and one backend write for one "
            "entry point. Times are microseconds. The summary rows are geometric means over the "
            "cases that succeeded in both checkouts (`min` columns are the geometric mean of the "
            "per-case minimums; the changes are geometric means of per-case ratios). "
            "`spv_polyfill` is the SPIR-V writer with the workgroup zero-init polyfill instead "
            "of the native mode, and is not part of `total`.\n"
        )
        for backend in backends:
            by_group = defaultdict(lambda: defaultdict(Ratios))
            sizes = defaultdict(lambda: [0, 0])
            for case, metric, b, n in self.pairs(bench, backend):
                self.csv_row(bench, backend, case, metric, b, n)
                if b and n and b["status"] == "ok" and n["status"] == "ok":
                    for group in (b.get("group", "?"), "all"):
                        by_group[group][metric].add(b, n)
                    if metric in CPU_BACKENDS + ["spv_polyfill"] and "output_bytes" in b["aux"] and "output_bytes" in n["aux"]:
                        sizes[metric][0] += int(b["aux"]["output_bytes"])
                        sizes[metric][1] += int(n["aux"]["output_bytes"])
            for group in ["all"] + sorted(g for g in by_group if g != "all"):
                self.add(f"### Summary: {group} cases\n")
                self.add(self.summary(by_group[group], CPU_METRICS, "us"))
            if sizes:
                self.add("### Output size (bytes, sum over cases that succeeded in both)\n")
                self.add(
                    table(
                        ["backend", self.base_label, self.new_label, "abs", "%"],
                        [
                            [m, f"{b:,}", f"{n:,}", f"{n - b:+,}", pct(b, n)]
                            for m, (b, n) in sorted(sizes.items())
                        ],
                    )
                )
            self.cpu_bench_shaders(bench, backend)
            self.largest_changes(bench, backend, CPU_BACKENDS + ["total"], "us")

    def cpu_bench_shaders(self, bench, backend):
        cases = defaultdict(dict)
        for case, metric, b, n in self.pairs(bench, backend):
            rec = b or n
            if rec.get("group") == "bench" and metric in CPU_BACKENDS + ["total"]:
                cases[case][metric] = (b, n)
        if not cases:
            return
        self.add("### Benchmark shaders\n")
        self.add("Each cell is the minimum, `base -> new (%)`, in microseconds; `ERR` marks a "
                 "failed backend.\n")
        rows = []
        for case in sorted(cases):
            row = [case.replace("::main", "")]
            for metric in CPU_BACKENDS + ["total"]:
                b, n = cases[case].get(metric, (None, None))
                row.append(cell(b, n, "us"))
            rows.append(row)
        self.add(table(["shader"] + CPU_BACKENDS + ["total"], rows))

    def largest_changes(self, bench, backend, metrics, unit, limit=8):
        changes = []
        for case, metric, b, n in self.pairs(bench, backend):
            if metric in metrics and b and n and b["status"] == "ok" and n["status"] == "ok":
                changes.append((n["min"] - b["min"], case, metric, b, n))
        if not changes:
            return
        changes.sort(key=lambda c: c[:3])
        self.add(f"### Largest changes ({backend})\n")
        rows = []
        head = changes[:limit]
        tail = [c for c in changes[-limit:][::-1] if c not in head]
        for delta, case, metric, b, n in head + tail:
            rows.append([case, metric, num(b["min"], unit), num(n["min"], unit),
                         pct(b["min"], n["min"]), num(b["median"], unit),
                         num(n["median"], unit), pct(b["median"], n["median"])])
        self.add("Ordered by the change in the minimum: most improved first, then most "
                 "regressed.\n")
        self.add(table(["case", "metric", f"{self.base_label} min", f"{self.new_label} min",
                        "% (min)", f"{self.base_label} median", f"{self.new_label} median",
                        "% (median)"], rows))

    def pipeline(self):
        bench = "pipeline_creation"
        backends = self.backends(bench)
        if not backends:
            return
        self.add("## Pipeline creation through wgpu (includes the driver compiler)\n")
        self.add(
            "`create_shader_module` plus `create_*_pipeline`, in milliseconds, for iterations "
            "that each compile a differently salted shader (see the README). The summary rows "
            "are geometric means over the cases that succeeded in both checkouts.\n"
        )
        for backend in backends:
            self.add(f"### Backend: {backend}\n")
            by_group = defaultdict(lambda: defaultdict(Ratios))
            weak = set()
            for case, metric, b, n in self.pairs(bench, backend):
                self.csv_row(bench, backend, case, metric, b, n)
                if metric == "total":
                    for rec in (b, n):
                        if rec and rec.get("aux", {}).get("variation") == "rename":
                            weak.add(case)
                if b and n and b["status"] == "ok" and n["status"] == "ok":
                    for group in (b.get("group", "?"), "all"):
                        by_group[group][metric].add(b, n)
            for group in ["all"] + sorted(g for g in by_group if g != "all"):
                self.add(f"Summary: {group} cases\n")
                self.add(self.summary(by_group[group],
                                      ("create_shader_module", "create_pipeline", "total"), "ms"))
            if weak:
                self.add(
                    f"**{len(weak)} case(s) were varied only by renaming their entry points "
                    "(`variation: rename` in the JSON), which some drivers ignore, so a driver "
                    "cache may have served them:** " + ", ".join(f"`{c}`" for c in sorted(weak)[:10])
                    + (", ..." if len(weak) > 10 else "") + "\n"
                )
            rows = []
            for case, metric, b, n in self.pairs(bench, backend):
                rec = b or n
                if metric == "total" and rec.get("group") == "bench":
                    rows.append([case.replace("::main", ""), cell(b, n, "ms"), cell(b, n, "ms", "median")])
            if rows:
                self.add("Benchmark shaders (total per pipeline):\n")
                self.add(table(["shader", f"min, {self.base_label} -> {self.new_label}",
                                f"median, {self.base_label} -> {self.new_label}"], rows))
            self.largest_changes(bench, backend, {"total"}, "ms")

    def runtime(self):
        bench = "gpu_runtime"
        backends = self.backends(bench)
        if not backends:
            return
        self.add("## GPU runtime of the benchmark shaders\n")
        self.add(
            "One dispatch, in milliseconds. `src` is the timing source: `ts` timestamp queries, "
            "`wall` wall time around submit and poll. `check` compares the hash of the shader's "
            "output between the two checkouts; `MISMATCH` means the shader computed something "
            "different.\n"
        )
        for backend in backends:
            self.add(f"### Backend: {backend}\n")
            rows = []
            ratios = Ratios()
            for case, metric, b, n in self.pairs(bench, backend):
                self.csv_row(bench, backend, case, metric, b, n)
                if metric != "gpu_time":
                    continue
                ok = b and n and b["status"] == "ok" and n["status"] == "ok"
                if not ok:
                    rows.append([case, cell(b, n, "ms"), "", "", "", "", ""])
                    continue
                ratios.add(b, n)
                cb, cn = b["aux"].get("checksum"), n["aux"].get("checksum")
                check = "ok" if cb == cn else "MISMATCH"
                src = {"timestamp": "ts", "wall": "wall"}
                src_b = src.get(b["aux"].get("timing_source"), "?")
                src_n = src.get(n["aux"].get("timing_source"), "?")
                rows.append(
                    [
                        case,
                        f"{b['min']:.3f} -> {n['min']:.3f}",
                        pct(b["min"], n["min"]),
                        f"{b['median']:.3f} -> {n['median']:.3f}",
                        pct(b["median"], n["median"]),
                        check,
                        src_b if src_b == src_n else f"{src_b}/{src_n}",
                    ]
                )
            if ratios.count():
                self.add(f"Geometric mean of the per-shader ratios over {ratios.count()} shaders: "
                         f"min {ratio_pct(ratios.change('min'))}, "
                         f"median {ratio_pct(ratios.change('median'))}.\n")
            self.add(table(["shader", "min", "%", "median", "%", "check", "src"], rows))

    def get(self, side, backend, case, metric="gpu_time"):
        recs = side.get(("gpu_runtime", backend, case, metric))
        rec = merge(recs) if recs else None
        return rec if rec and rec["status"] == "ok" else None

    def zero_init(self):
        backends = self.backends("gpu_runtime")
        if "vulkan" not in backends:
            return
        self.add("## Zero-init on Vulkan: native mode vs naga's polyfill\n")
        if "vulkan-polyfill" not in backends:
            self.add("N/A: no polyfill run (the `vulkan-polyfill` results are missing).\n")
            return
        self.add(
            "The `vulkan` run uses `VK_KHR_zero_initialize_workgroup_memory` when the device "
            "supports it. The `vulkan-polyfill` run forces naga's polyfill (the shared zero-init "
            "pass on the combined branch, the SPIR-V writer's own code on trunk). If the device "
            "has no native support both runs polyfill and the columns agree. `overhead` is "
            "polyfill vs native for one checkout; the last column is the passes checkout vs "
            "the base checkout, both polyfilled. Milliseconds, minimums.\n"
        )
        cases = sorted(
            {k[2] for k in list(self.base) + list(self.new) if k[0] == "gpu_runtime" and k[2].startswith("zeroinit_")}
        )
        rows = []
        for case in cases:
            row = [case]
            polyfilled = []
            for side in (self.base, self.new):
                native = self.get(side, "vulkan", case)
                poly = self.get(side, "vulkan-polyfill", case)
                polyfilled.append(poly)
                if native and poly:
                    row += [f"{native['min']:.3f}", f"{poly['min']:.3f}", pct(native["min"], poly["min"])]
                else:
                    row += ["N/A", "N/A", ""]
            if polyfilled[0] and polyfilled[1]:
                row.append(pct(polyfilled[0]["min"], polyfilled[1]["min"]))
            else:
                row.append("N/A")
            rows.append(row)
        bl, nl = self.base_label, self.new_label
        self.add(
            table(
                ["shader", f"{bl} native", f"{bl} polyfill", f"{bl} overhead",
                 f"{nl} native", f"{nl} polyfill", f"{nl} overhead", "polyfill, new vs base"],
                rows,
            )
        )

    def load(self):
        """The groups that ran on a busy machine, from the load guard fields."""
        if not self.guards:
            return
        flagged = [g for g in self.guards if g.get("high_load") == "true"]
        unknown = [g for g in self.guards if g.get("high_load") == "unknown"]
        self.add("## Machine load\n")
        self.add(
            "The runner reads the load (busy cores) before and after every benchmark group. A "
            "group is `high_load` if either reading is above the limit (by default 1 + 0.25 * "
            "cores). Timings from flagged groups are unreliable.\n"
        )
        if not flagged:
            self.add(f"No group of {len(self.guards)} was above the limit"
                     + (f" ({len(unknown)} could not be read)" if unknown else "") + ".\n")
            return
        self.add(f"**{len(flagged)} of {len(self.guards)} groups ran under high load.**\n")
        rows = [[g["label"], g["backend"], g["bench"], g["round"], g.get("load_before", "?"),
                 g.get("load_after", "?"), g.get("load_limit", "?"), g.get("started_utc", "?")]
                for g in flagged]
        self.add(table(["checkout", "backend", "bench", "round", "load before", "load after",
                        "limit", "started (UTC)"], rows))

    def failed_runs(self):
        """Benchmarks that did not produce results, or produced them on one side only."""
        seen = {}
        for label, meta, failures in (
            (self.base_label, self.base_meta, self.base_failures),
            (self.new_label, self.new_meta, self.new_failures),
        ):
            for (bench, backend) in meta:
                seen.setdefault((backend, bench), {}).setdefault(label, [])
            for failure in failures:
                seen.setdefault((failure["backend"], failure["bench"]), {}).setdefault(
                    label, []
                ).append(failure)
        rows = []
        for (backend, bench), sides in sorted(seen.items()):
            for label in (self.base_label, self.new_label):
                failures = sides.get(label)
                if failures is None:
                    rows.append([label, backend, bench, "-", "no results", "not run or no result file"])
                    continue
                for failure in failures:
                    kind = "failed, not retried" if failure.get("deterministic") else "gave up"
                    rows.append(
                        [label, backend, bench, failure.get("round", ""), kind,
                         str(failure.get("error", "")).replace("\n", " ").replace("|", "/")[:300]]
                    )
        if not rows:
            return
        self.add("## Benchmarks that produced no results\n")
        self.add(
            "A benchmark process that fails the same way twice in a row, with no new results, "
            "is not retried (a startup panic, for example). Partial results of a run that gave "
            "up after repeated crashes are kept and compared. `no results` means that side has "
            "no result file for this benchmark.\n"
        )
        self.add(table(["checkout", "backend", "bench", "round", "outcome", "message"], rows))

    def problems(self):
        self.add("## Errors, crashes and skipped cases\n")
        self.add(
            "`error` is a wgpu validation or backend error, `panic` a Rust panic that was caught, "
            "`crash` means the process died while the case ran (usually a driver crash), and "
            "`skipped` was not attempted. Cases that fail the same way in both checkouts are "
            "expected for shaders that need features the device lacks.\n"
        )
        rows = []
        counts = defaultdict(lambda: defaultdict(int))
        for label, data in ((self.base_label, self.base), (self.new_label, self.new)):
            for (bench, backend, case, metric), recs in sorted(data.items()):
                rec = merge(recs)
                if rec["status"] == "ok":
                    continue
                counts[(bench, backend, label)][rec["status"]] += 1
                other = (self.new if label == self.base_label else self.base).get(
                    (bench, backend, case, metric)
                )
                same = other and merge(other)["status"] == rec["status"]
                rows.append(
                    [
                        bench,
                        backend,
                        label,
                        case,
                        metric,
                        rec["status"] + (" (both)" if same else " (only here)" if other else ""),
                        (rec.get("error") or "").replace("\n", " ").replace("|", "/")[:160],
                    ]
                )
        if not rows:
            self.add("None.\n")
            return
        self.add(
            table(
                ["bench", "backend", "checkout", "status", "count"],
                [
                    [b, be, lab, status, c]
                    for (b, be, lab), d in sorted(counts.items())
                    for status, c in sorted(d.items())
                ],
            )
        )
        only = [r for r in rows if "only here" in r[5] or "(both)" not in r[5]]
        self.add("Cases whose status differs between the checkouts (or exist in only one):\n")
        self.add(table(["bench", "backend", "checkout", "case", "metric", "status", "error"], only[:80]))
        if len(only) > 80:
            self.add(f"... and {len(only) - 80} more, see comparison.csv.\n")


def cell(b, n, unit, field="min"):
    if b is None or n is None:
        return "missing in " + ("base" if b is None else "new")
    if b["status"] != "ok" or n["status"] != "ok":
        return f"{'ERR' if b['status'] != 'ok' else num(b[field], unit)} -> {'ERR' if n['status'] != 'ok' else num(n[field], unit)}"
    return f"{num(b[field], unit)} -> {num(n[field], unit)} ({pct(b[field], n[field])})"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("results")
    parser.add_argument("--base", default="trunk")
    parser.add_argument("--new", default="combined")
    args = parser.parse_args()

    base, base_meta, base_failures, base_guards = load(args.results, args.base)
    new, new_meta, new_failures, new_guards = load(args.results, args.new)

    report = Report(base, new, args.base, args.new, base_meta, new_meta, base_failures,
                    new_failures, base_guards + new_guards)
    info = run_info(args.results)
    quick = info.get("quick") == "1"
    report.add(f"# naga passes: {args.base} vs {args.new}\n")
    if quick:
        report.add("**QUICK RUN: a smoke check of the setup (2 samples per pipeline, small "
                   "dispatches). THESE NUMBERS ARE INVALID FOR COMPARISONS.**\n")
    status = 0
    if not base or not new:
        status = 1
        missing = [label for label, data in ((args.base, base), (args.new, new)) if not data]
        report.add(
            "**No results for " + " and ".join(f"`{m}`" for m in missing) + f" in "
            f"`{args.results}`**: every benchmark run failed or nothing was written. "
            "There is nothing to compare; see the failures below and the `*.log` files.\n"
        )
    else:
        report.add("Negative % is faster for the second checkout. Per-case numbers are the "
                   "minimum over all samples and rounds, with the median of the per-round "
                   "medians for reference. Summary changes are geometric means of per-case "
                   "new/base ratios, not ratios of sums.\n")
        rounds = max((len(v) for data in (base, new) for v in data.values()), default=0)
        if rounds < 3 and not quick:
            report.add(f"**Only {rounds} round(s) of results; at least 3 are needed to tell "
                       "noise from a difference.**\n")
        flagged = sum(1 for g in base_guards + new_guards if g.get("high_load") == "true")
        if flagged:
            report.add(f"**{flagged} benchmark group(s) ran on a busy machine; see \"Machine "
                       "load\" below.**\n")
    failed = len(base_failures) + len(new_failures)
    if failed and base and new:
        report.add(f"**{failed} benchmark run(s) failed to complete; see \"Benchmarks that "
                   "produced no results\" below.**\n")
    adapters = {}
    for (bench, backend), meta in list(new_meta.items()) + list(base_meta.items()):
        if "adapter_name" in meta:
            adapters[backend] = (
                f"{meta['adapter_name']} ({meta.get('backend', backend)}, {meta.get('device_type', '?')}), "
                f"driver {driver_text(meta, backend, info)}"
            )
    for backend, text in sorted(adapters.items()):
        report.add(f"- `{backend}`: {text}")
    report.add("")
    if status == 0:
        report.cpu()
        report.pipeline()
        report.runtime()
        report.zero_init()
        report.load()
    report.failed_runs()
    if status == 0:
        report.problems()

    text = "\n".join(report.out)
    with open(os.path.join(args.results, "comparison.md"), "w", encoding="utf-8") as f:
        f.write(text + "\n")
    if report.csv_rows:
        with open(os.path.join(args.results, "comparison.csv"), "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=list(report.csv_rows[0].keys()))
            writer.writeheader()
            writer.writerows(report.csv_rows)
    print(text)
    return status


if __name__ == "__main__":
    sys.exit(main())
