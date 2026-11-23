#!/usr/bin/env python3
"""Compare the results of two checkouts of the naga pass benchmarks.

    compare.py RESULTS_DIR [--base trunk] [--new combined]

RESULTS_DIR holds <label>/<backend>/r<round>/<benchmark>.json files, as written by run.sh.
Writes comparison.md and comparison.csv into RESULTS_DIR and prints the markdown.
"""

import argparse
import csv
import glob
import os
import statistics
import sys
from collections import defaultdict

import json

CPU_METRICS = ["parse", "validate", "overrides", "spv", "spv_polyfill", "hlsl", "msl", "glsl", "total"]
CPU_BACKENDS = ["spv", "hlsl", "msl", "glsl"]


def load(root, label):
    """Returns {(bench, backend, case, metric): [record per round]} and per-run metadata."""
    records = defaultdict(list)
    meta = {}
    for path in sorted(glob.glob(os.path.join(root, label, "*", "r*", "*.json"))):
        backend = path.split(os.sep)[-3]
        try:
            with open(path) as f:
                data = json.load(f)
        except (OSError, ValueError) as e:
            print(f"warning: cannot read {path}: {e}", file=sys.stderr)
            continue
        bench = data.get("bench", os.path.basename(path)[:-5])
        meta[(bench, backend)] = data.get("meta", {})
        for record in data.get("records", []):
            records[(bench, backend, record["case"], record["metric"])].append(record)
    return records, meta


def merge(recs):
    """One record from the rounds of a measurement: medians of medians if all rounds are ok."""
    bad = [r for r in recs if r["status"] != "ok"]
    if bad:
        return dict(bad[0])
    out = dict(recs[0])
    for key in ("median", "mean", "p90", "min", "max"):
        out[key] = statistics.median(r[key] for r in recs)
    out["n"] = sum(r["n"] for r in recs)
    return out


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
    def __init__(self, base, new, base_label, new_label):
        self.base = base
        self.new = new
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
        row["base_error"] = (b or {}).get("error") or ""
        row["new_error"] = (n or {}).get("error") or ""
        self.csv_rows.append(row)

    # -- sections --

    def cpu(self):
        bench = "cpu_compile"
        backends = self.backends(bench)
        if not backends:
            return
        self.add("## CPU compile pipeline (naga only)\n")
        self.add(
            "One iteration is parse, validate, process_overrides and one backend write for one "
            "entry point. Times are microseconds, medians; sums are over the cases that "
            "succeeded in both checkouts. `spv_polyfill` is the SPIR-V writer with the "
            "workgroup zero-init polyfill instead of the native mode, and is not part of "
            "`total`.\n"
        )
        for backend in backends:
            rows_by_group = defaultdict(lambda: defaultdict(lambda: [0.0, 0.0, 0]))
            sizes = defaultdict(lambda: [0, 0])
            for case, metric, b, n in self.pairs(bench, backend):
                self.csv_row(bench, backend, case, metric, b, n)
                if b and n and b["status"] == "ok" and n["status"] == "ok":
                    for group in (b.get("group", "?"), "all"):
                        acc = rows_by_group[group][metric]
                        acc[0] += b["median"]
                        acc[1] += n["median"]
                        acc[2] += 1
                    if metric in CPU_BACKENDS + ["spv_polyfill"] and "output_bytes" in b["aux"] and "output_bytes" in n["aux"]:
                        sizes[metric][0] += int(b["aux"]["output_bytes"])
                        sizes[metric][1] += int(n["aux"]["output_bytes"])
            for group in ["all"] + sorted(g for g in rows_by_group if g != "all"):
                self.add(f"### Totals: {group} cases\n")
                rows = []
                for metric in CPU_METRICS:
                    if metric in rows_by_group[group]:
                        b, n, count = rows_by_group[group][metric]
                        rows.append(
                            [metric, count, f"{b:,.0f}", f"{n:,.0f}", f"{n - b:+,.0f}", pct(b, n)]
                        )
                self.add(
                    table(
                        ["metric", "cases", f"{self.base_label} us", f"{self.new_label} us", "abs", "%"],
                        rows,
                    )
                )
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
        self.add("Each cell is `base -> new (%)` in microseconds; `ERR` marks a failed backend.\n")
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
                changes.append((n["median"] - b["median"], case, metric, b["median"], n["median"]))
        if not changes:
            return
        changes.sort()
        self.add(f"### Largest changes ({backend})\n")
        rows = []
        head = changes[:limit]
        tail = [c for c in changes[-limit:][::-1] if c not in head]
        for delta, case, metric, b, n in head + tail:
            rows.append([case, metric, num(b, unit), num(n, unit), f"{delta:+,.3f}".rstrip("0").rstrip("."), pct(b, n)])
        self.add("Top: most improved first, then most regressed.\n")
        self.add(table(["case", "metric", self.base_label, self.new_label, "abs", "%"], rows))

    def pipeline(self):
        bench = "pipeline_creation"
        backends = self.backends(bench)
        if not backends:
            return
        self.add("## Pipeline creation through wgpu (includes the driver compiler)\n")
        self.add(
            "`create_shader_module` plus `create_*_pipeline`, in milliseconds, medians of "
            "iterations that each compile a varied shader (see the README). Sums are over the "
            "cases that succeeded in both checkouts.\n"
        )
        for backend in backends:
            self.add(f"### Backend: {backend}\n")
            sums = defaultdict(lambda: defaultdict(lambda: [0.0, 0.0, 0]))
            for case, metric, b, n in self.pairs(bench, backend):
                self.csv_row(bench, backend, case, metric, b, n)
                if b and n and b["status"] == "ok" and n["status"] == "ok":
                    for group in (b.get("group", "?"), "all"):
                        acc = sums[group][metric]
                        acc[0] += b["median"]
                        acc[1] += n["median"]
                        acc[2] += 1
            for group in ["all"] + sorted(g for g in sums if g != "all"):
                rows = []
                for metric in ("create_shader_module", "create_pipeline", "total"):
                    if metric in sums[group]:
                        b, n, count = sums[group][metric]
                        rows.append([metric, count, f"{b:,.1f}", f"{n:,.1f}", f"{n - b:+,.1f}", pct(b, n)])
                self.add(f"Totals: {group} cases\n")
                self.add(
                    table(
                        ["metric", "cases", f"{self.base_label} ms", f"{self.new_label} ms", "abs", "%"],
                        rows,
                    )
                )
            rows = []
            for case, metric, b, n in self.pairs(bench, backend):
                rec = b or n
                if metric == "total" and rec.get("group") == "bench":
                    rows.append([case.replace("::main", ""), cell(b, n, "ms")])
            if rows:
                self.add("Benchmark shaders (total per pipeline):\n")
                self.add(table(["shader", f"{self.base_label} -> {self.new_label}"], rows))
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
            for case, metric, b, n in self.pairs(bench, backend):
                self.csv_row(bench, backend, case, metric, b, n)
                if metric != "gpu_time":
                    continue
                ok = b and n and b["status"] == "ok" and n["status"] == "ok"
                if not ok:
                    rows.append([case, cell(b, n, "ms"), "", "", "", "", ""])
                    continue
                cb, cn = b["aux"].get("checksum"), n["aux"].get("checksum")
                check = "ok" if cb == cn else "MISMATCH"
                src = {"timestamp": "ts", "wall": "wall"}
                src_b = src.get(b["aux"].get("timing_source"), "?")
                src_n = src.get(n["aux"].get("timing_source"), "?")
                rows.append(
                    [
                        case,
                        f"{b['median']:.3f} -> {n['median']:.3f}",
                        pct(b["median"], n["median"]),
                        f"{b['p90']:.3f} -> {n['p90']:.3f}",
                        pct(b["p90"], n["p90"]),
                        check,
                        src_b if src_b == src_n else f"{src_b}/{src_n}",
                    ]
                )
            self.add(table(["shader", "median", "%", "p90", "%", "check", "src"], rows))

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
            "the base checkout, both polyfilled. Milliseconds, medians.\n"
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
                    row += [f"{native['median']:.3f}", f"{poly['median']:.3f}", pct(native["median"], poly["median"])]
                else:
                    row += ["N/A", "N/A", ""]
            if polyfilled[0] and polyfilled[1]:
                row.append(pct(polyfilled[0]["median"], polyfilled[1]["median"]))
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


def cell(b, n, unit):
    if b is None or n is None:
        return "missing in " + ("base" if b is None else "new")
    if b["status"] != "ok" or n["status"] != "ok":
        return f"{'ERR' if b['status'] != 'ok' else num(b['median'], unit)} -> {'ERR' if n['status'] != 'ok' else num(n['median'], unit)}"
    return f"{num(b['median'], unit)} -> {num(n['median'], unit)} ({pct(b['median'], n['median'])})"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("results")
    parser.add_argument("--base", default="trunk")
    parser.add_argument("--new", default="combined")
    args = parser.parse_args()

    base, base_meta = load(args.results, args.base)
    new, new_meta = load(args.results, args.new)
    if not base or not new:
        print(f"no results for {args.base if not base else args.new} in {args.results}", file=sys.stderr)
        return 1

    report = Report(base, new, args.base, args.new)
    report.add(f"# naga passes: {args.base} vs {args.new}\n")
    report.add("Negative % is faster for the second checkout. `abs` is new minus base.\n")
    adapters = {}
    for (bench, backend), meta in list(new_meta.items()) + list(base_meta.items()):
        if "adapter_name" in meta:
            adapters[backend] = (
                f"{meta['adapter_name']} ({meta.get('backend', backend)}, {meta.get('device_type', '?')}), "
                f"driver {meta.get('driver', '?')} {meta.get('driver_info', '')}".strip()
            )
    for backend, text in sorted(adapters.items()):
        report.add(f"- `{backend}`: {text}")
    report.add("")
    report.cpu()
    report.pipeline()
    report.runtime()
    report.zero_init()
    report.problems()

    text = "\n".join(report.out)
    with open(os.path.join(args.results, "comparison.md"), "w") as f:
        f.write(text + "\n")
    if report.csv_rows:
        with open(os.path.join(args.results, "comparison.csv"), "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=list(report.csv_rows[0].keys()))
            writer.writeheader()
            writer.writerows(report.csv_rows)
    print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
