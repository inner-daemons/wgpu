#!/usr/bin/env bash
# Build and run the driver benchmarks (pipeline creation and shader runtime) on two checkouts
# and compare them. The CPU-only naga benchmark is separate: see benches/naga-passes/cpu.sh.
#
#   benches/naga-passes/driver/run.sh [options] [BASE_CHECKOUT NEW_CHECKOUT]
#
# BASE_CHECKOUT and NEW_CHECKOUT are checkouts of the `trunk-bench` and `naga-passes-bench`
# branches. If they are left out, the script makes git worktrees of those two branches (local or
# on `origin`) in <repo>/target/naga-passes-worktrees. See benches/naga-passes/driver/README.md.
set -o pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"

out=""
backends=""
rounds=1
run_info=1
quick=0
full_tests=0
run_bench_step=1
base_label="trunk"
new_label="combined"

usage() {
    sed -n '2,9p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
    cat <<'USAGE'

Options:
  --out DIR          Where to write results (default: ./naga-passes-results/<date>-<host>).
  --backends LIST    Comma separated wgpu backends for the device benchmarks
                     (default: metal on macOS, vulkan on Linux, dx12,vulkan on Windows).
                     Add `gl` to include the GLES backend.
  --rounds N         Run each benchmark N times per checkout, alternating order (default 1).
  --quick            A fast smoke run: few iterations, small dispatches. Not for comparisons.
  --no-info          Skip collecting adapter information with wgpu-info.
  --full-tests       Also run `cargo xtask test` in both checkouts and compare the failures.
                     Long (see the README); needs cargo-nextest and, on most OSes, the Vulkan SDK.
  --no-bench         Skip the benchmarks (use with --full-tests to run only the tests).
  --labels A,B       Names for the two checkouts in the report (default trunk,combined).
  -h, --help         Show this help.
USAGE
}

while [ $# -gt 0 ]; do
    case "$1" in
        --out) out="$2"; shift 2 ;;
        --backends) backends="$2"; shift 2 ;;
        --rounds) rounds="$2"; shift 2 ;;
        --quick) quick=1; shift ;;
        --no-info) run_info=0; shift ;;
        --full-tests) full_tests=1; shift ;;
        --no-bench) run_bench_step=0; shift ;;
        --labels) base_label="${2%%,*}"; new_label="${2##*,}"; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        --) shift; break ;;
        -*) echo "unknown option: $1" >&2; usage >&2; exit 2 ;;
        *) break ;;
    esac
done

if [ -z "$backends" ]; then
    case "$(uname -s)" in
        Darwin) backends="metal" ;;
        MINGW*|MSYS*|CYGWIN*) backends="dx12,vulkan" ;;
        *) backends="vulkan" ;;
    esac
fi
[ -n "$out" ] || out="$PWD/naga-passes-results/$(date +%Y%m%d-%H%M%S)-$(hostname -s 2>/dev/null || echo host)"
mkdir -p "$out"
out="$(cd "$out" && pwd)"

if [ $# -ge 2 ]; then
    base_dir="$(cd "$1" && pwd)"
    new_dir="$(cd "$2" && pwd)"
elif [ $# -eq 0 ]; then
    wt="$repo/target/naga-passes-worktrees"
    mkdir -p "$wt"
    for pair in "base:trunk-bench" "new:naga-passes-bench"; do
        kind="${pair%%:*}"
        branch="${pair##*:}"
        dir="$wt/$branch"
        if [ ! -d "$dir" ]; then
            ref=""
            for candidate in "$branch" "origin/$branch"; do
                if git -C "$repo" rev-parse --verify --quiet "$candidate^{commit}" >/dev/null; then
                    ref="$candidate"
                    break
                fi
            done
            if [ -z "$ref" ]; then
                echo "branch $branch not found; fetch it or pass the two checkouts as arguments" >&2
                exit 2
            fi
            git -C "$repo" worktree add --detach "$dir" "$ref" || exit 1
        fi
        if [ "$kind" = base ]; then base_dir="$dir"; else new_dir="$dir"; fi
    done
else
    usage >&2
    exit 2
fi

for dir in "$base_dir" "$new_dir"; do
    [ -f "$dir/benches/benches/naga-passes-driver/main.rs" ] || {
        echo "$dir does not contain the naga pass benchmarks" >&2
        exit 2
    }
done

# Disk caches of compiled shaders would hide the compile time of repeated pipelines.
export MESA_SHADER_CACHE_DISABLE=true
export MESA_GLSL_CACHE_DISABLE=true
export __GL_SHADER_DISK_CACHE=0
export RUST_BACKTRACE=1

if [ "$quick" = 1 ]; then
    export NAGA_PASSES_PIPELINE_ITERS=2
    export NAGA_PASSES_GPU_GROUPS=256 NAGA_PASSES_GPU_RUNS=5 NAGA_PASSES_GPU_WARMUP=1
fi

echo "base ($base_label): $base_dir ($(git -C "$base_dir" rev-parse --short HEAD))"
echo "new  ($new_label): $new_dir ($(git -C "$new_dir" rev-parse --short HEAD))"
echo "results: $out"

{
    echo "date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "uname: $(uname -a)"
    echo "rustc: $(rustc -V 2>&1)"
    echo "base: $base_label $(git -C "$base_dir" rev-parse HEAD)"
    echo "new: $new_label $(git -C "$new_dir" rev-parse HEAD)"
    echo "backends: $backends"
    echo "rounds: $rounds"
    echo "quick: $quick"
    case "$(uname -s)" in
        Darwin) sw_vers 2>&1 ;;
        Linux) cat /etc/os-release 2>&1 | head -5 ;;
    esac
} > "$out/run-info.txt"

build() {
    echo "== building $1"
    (cd "$2" && cargo bench -p wgpu-benchmark --bench naga-passes-driver --no-run) > "$out/build-$1.log" 2>&1 || {
        echo "build of $1 failed; see $out/build-$1.log" >&2
        tail -20 "$out/build-$1.log" >&2
        exit 1
    }
}
if [ "$run_bench_step" = 1 ]; then
    build "$base_label" "$base_dir"
    build "$new_label" "$new_dir"
fi

if [ "$run_info" = 1 ]; then
    echo "== collecting adapter information"
    (cd "$new_dir" && cargo run --release -p wgpu-info) > "$out/wgpu-info.txt" 2>&1 \
        || echo "wgpu-info failed; see $out/wgpu-info.txt"
fi

# run_bench LABEL CHECKOUT BENCH_NAME RESULT_DIR [WGPU_BACKEND [EXTRA_ENV]]
#
# If the benchmark process dies, which is what a driver crash looks like, run it again with
# NAGA_PASSES_RESUME=1. It then records the case that was running as a crash and continues
# after it.
run_bench() {
    local label="$1" dir="$2" bench="$3" dest="$4" backend="${5:-}" extra="${6:-}"
    mkdir -p "$dest"
    local attempt=0 status
    while :; do
        local resume=()
        [ "$attempt" -gt 0 ] && resume=(NAGA_PASSES_RESUME=1)
        local backend_env=()
        [ -n "$backend" ] && backend_env=(WGPU_BACKEND="$backend")
        [ -n "$extra" ] && backend_env+=("$extra")
        (cd "$dir" && env NAGA_PASSES_OUT="$dest" "${resume[@]}" "${backend_env[@]}" \
            cargo bench -p wgpu-benchmark --bench naga-passes-driver -- "naga_passes::$bench") \
            >> "$dest/$bench.log" 2>&1
        status=$?
        [ "$status" -eq 0 ] && break
        attempt=$((attempt + 1))
        echo "   $label $bench exited with status $status (attempt $attempt)"
        if [ "$attempt" -ge 10 ]; then
            echo "   giving up on $label $bench; partial results are kept"
            break
        fi
    done
}

if [ "$run_bench_step" = 1 ]; then
for round in $(seq 1 "$rounds"); do
    if [ $((round % 2)) -eq 1 ]; then
        order=("$base_label:$base_dir" "$new_label:$new_dir")
    else
        order=("$new_label:$new_dir" "$base_label:$base_dir")
    fi

    IFS=',' read -r -a backend_list <<< "$backends"
    for backend in "${backend_list[@]}"; do
        for bench in pipeline_creation gpu_runtime; do
            for entry in "${order[@]}"; do
                label="${entry%%:*}"; dir="${entry#*:}"
                echo "== round $round: $bench on $label ($backend)"
                run_bench "$label" "$dir" "$bench" "$out/$label/$backend/r$round" "$backend"
            done
        done
        if [ "$backend" = vulkan ]; then
            # The same shaders with naga's workgroup zero-init polyfill forced on, to compare
            # with the native mode that the run above uses when the device supports it.
            for entry in "${order[@]}"; do
                label="${entry%%:*}"; dir="${entry#*:}"
                echo "== round $round: gpu_runtime with polyfilled zero-init on $label (vulkan)"
                run_bench "$label" "$dir" gpu_runtime "$out/$label/vulkan-polyfill/r$round" vulkan \
                    WGPU_BENCH_ONLY_FORCE_ZERO_INIT_POLYFILL=1
            done
        fi
    done
done
fi

# The full test suite on each checkout, to tell new failures from existing ones.
run_tests() {
    local label="$1" dir="$2"
    mkdir -p "$out/tests"
    echo "== cargo xtask test on $label (long)"
    (cd "$dir" && CARGO_TERM_COLOR=never NO_COLOR=1 cargo xtask test) > "$out/tests/$label.log" 2>&1
    echo "   exit status $? (see $out/tests/$label.log)"
    sed $'s/\033\\[[0-9;]*m//g' "$out/tests/$label.log" \
        | grep -E '^ *(FAIL|SIGABRT|SIGSEGV|SIGBUS|SIGILL|SIGTRAP|TIMEOUT|LEAK|FLAKY)' \
        | sed -E 's/\[[^]]*\]//; s/^ +//; s/ +/ /g' | sort -u > "$out/tests/$label-failures.txt"
    echo "   $(wc -l < "$out/tests/$label-failures.txt" | tr -d ' ') distinct failing tests in $out/tests/$label-failures.txt"
}

if [ "$full_tests" = 1 ]; then
    run_tests "$base_label" "$base_dir"
    run_tests "$new_label" "$new_dir"
    {
        echo "Failing on $new_label but not on $base_label (new failures):"
        comm -13 "$out/tests/$base_label-failures.txt" "$out/tests/$new_label-failures.txt"
        echo
        echo "Failing on $base_label but not on $new_label (fixed):"
        comm -23 "$out/tests/$base_label-failures.txt" "$out/tests/$new_label-failures.txt"
        echo
        echo "Failing on both:"
        comm -12 "$out/tests/$base_label-failures.txt" "$out/tests/$new_label-failures.txt"
    } | tee "$out/tests/failure-diff.txt"
fi

echo "== comparing"
if [ "$run_bench_step" = 1 ]; then
    python3 "$here/../compare.py" "$out" --base "$base_label" --new "$new_label" | tee "$out/comparison.txt"
fi
echo
echo "Results and logs are in $out"
echo "Send back: $out (without the build logs if large), or the .tar.gz below."
tar -czf "$out.tar.gz" -C "$(dirname "$out")" --exclude='build-*.log' "$(basename "$out")" 2>/dev/null \
    && echo "Archive: $out.tar.gz"
