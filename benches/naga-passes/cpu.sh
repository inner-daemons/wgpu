#!/usr/bin/env bash
# Run the CPU-only naga benchmark (WGSL parse, validate, process_overrides and the SPIR-V, HLSL,
# MSL and GLSL writers) on two checkouts and compare them. No GPU is used.
#
#   benches/naga-passes/cpu.sh BASE_CHECKOUT NEW_CHECKOUT [OUT_DIR [ROUNDS]]
#
# The checkouts are the `trunk-bench` and `naga-passes-bench` branches. See README.md.
set -o pipefail

[ $# -ge 2 ] || { sed -n '2,7p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 2; }
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
base="$(cd "$1" && pwd)"
new="$(cd "$2" && pwd)"
out="${3:-$PWD/naga-passes-cpu-results/$(date +%Y%m%d-%H%M%S)}"
rounds="${4:-3}"
mkdir -p "$out" && out="$(cd "$out" && pwd)"

for dir in "$base" "$new"; do
    (cd "$dir" && cargo bench -p wgpu-benchmark --bench naga-passes-cpu --no-run) || exit 1
done

# Alternate the order each round so drift in machine speed does not favor one checkout.
for round in $(seq 1 "$rounds"); do
    if [ $((round % 2)) -eq 1 ]; then pair=("trunk:$base" "combined:$new"); else pair=("combined:$new" "trunk:$base"); fi
    for entry in "${pair[@]}"; do
        label="${entry%%:*}"; dir="${entry#*:}"
        echo "== round $round: $label"
        mkdir -p "$out/$label/cpu/r$round"
        (cd "$dir" && NAGA_PASSES_OUT="$out/$label/cpu/r$round" \
            cargo bench -p wgpu-benchmark --bench naga-passes-cpu -- naga_passes::cpu_compile) \
            > "$out/$label/cpu/r$round/cpu_compile.log" 2>&1 \
            || echo "   $label failed; see $out/$label/cpu/r$round/cpu_compile.log"
    done
done

python3 "$here/compare.py" "$out" --base trunk --new combined
echo "Results: $out"
