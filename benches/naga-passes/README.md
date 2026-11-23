# naga pass benchmarks

These benchmarks compare two checkouts of wgpu, `trunk` and a branch of naga IR passes, on the
shader compile and shader runtime paths. They live in the `wgpu-benchmark` package, as two
separate bench targets so each can run on its own:

| target | measures | needs a GPU | runner |
|---|---|---|---|
| `naga-passes-cpu` | the naga half of compiling a pipeline | no | `cpu.sh` (this file) |
| `naga-passes-driver` | pipeline creation including the driver compiler, and shader runtime | yes | [`driver/run.sh`](driver/README.md) |

The two checkouts are the branches `trunk-bench` and `naga-passes-bench`. They carry identical
benchmark code, except `benches/benches/naga-passes-cpu/naga_adapter.rs`: the naga backend
writers have different signatures on the two branches, and that file is the only place that
calls them.

## CPU compile benchmark (`naga-passes-cpu`)

For each shader and each entry point, one iteration does what wgpu does to create one pipeline,
and times each step:

1. parse the WGSL (`naga::front::wgsl`, with all capabilities);
2. validate (`ValidationFlags::all()`, `Capabilities::all()`, all subgroup stages and operations);
3. `naga::back::pipeline_constants::process_overrides` for that entry point, with overrides that
   have no default set to 1;
4. write SPIR-V, HLSL, MSL and GLSL, with options like wgpu-hal's: workgroup zero
   initialization on, loop bounding on, `Restrict` bounds checks (except GLSL, which wgpu-hal
   leaves `Unchecked`), SPIR-V 1.6, HLSL shader model 6.6, MSL 3.1, GLES 3.1.

Reported per shader and entry point: `parse`, `validate`, `overrides`, `spv`, `hlsl`, `msl`,
`glsl`, `total` (the sum of those, which counts each backend once), plus `spv_polyfill` (the
SPIR-V writer with the zero-init polyfill instead of the native mode, which wgpu uses on devices
without `VK_KHR_zero_initialize_workgroup_memory`). The size of each output is recorded too. Each
case runs for at least 10 iterations and 150 ms after one warm-up iteration, and the median is
reported.

Differences from wgpu-hal: bindings are faked rather than taken from a pipeline layout, the SPIR-V
capability set is not restricted, and MSL vertex pulling is off.

The shaders (`NAGA_PASSES_CORPUS`, comma separated):

- `bench`: the shaders in `benches/benches/naga-passes-common/shaders/`, instantiated with
  workgroup sizes 64 and 256: `zero_init_array` (1 KB, 16 KB and about 32 KB of workgroup
  memory), `zero_init_2d`, `zero_init_atomic`, `loops`, `helpers`, `matcx2`;
- `examples`: `examples/features/src/**/*.wgsl`;
- `snapshots` (off by default): the naga snapshot test inputs in `naga/tests/in/wgsl`. These
  differ between the branches, so only shaders present in both are compared.

Shaders that cannot be written by a backend (GLSL has no mesh shaders, for example) are recorded
as errors for that backend, and a panic in a writer is caught and recorded.

### Running it

```sh
# From any checkout that has benches/naga-passes (about 5 to 15 minutes the first time, which is
# mostly building two copies of wgpu; the benchmark itself is estimated at 1 to 3 minutes per
# checkout and round, not measured yet).
benches/naga-passes/cpu.sh ../wgpu-trunk-bench ../wgpu-passes-bench [OUT_DIR [ROUNDS]]
```

It builds both, runs the benchmark `ROUNDS` times (default 3) per checkout, alternating which
checkout goes first, and prints and saves a comparison: `OUT_DIR/comparison.md` and
`comparison.csv`. Negative percentages mean the second checkout, the passes branch, is faster.
Close other programs while it runs; the CPU is the only thing measured.

To run one checkout by hand, and use the harness' own output:

```sh
cargo bench -p wgpu-benchmark --bench naga-passes-cpu
# Narrow the shaders, for example to the benchmark shaders only:
NAGA_PASSES_CORPUS=bench cargo bench -p wgpu-benchmark --bench naga-passes-cpu
NAGA_PASSES_FILTER=zeroinit,loops cargo bench -p wgpu-benchmark --bench naga-passes-cpu
```

Detailed results are written to `target/bench/naga_passes/cpu_compile.json` (or
`$NAGA_PASSES_OUT`). To compare two such files, put them under
`OUT/<label>/cpu/r1/cpu_compile.json` and run `compare.py OUT --base <label> --new <label>`.

Settings: `NAGA_PASSES_CPU_MIN_ITERS`, `NAGA_PASSES_CPU_MAX_ITERS`, `NAGA_PASSES_CPU_BUDGET_MS`.

## Notes on the combined branch

On the combined branch the naga backends run passes (inlining, loop bounding, zero
initialization, matrix layout) and validate the module again inside `write`, so the backend
times there include that work; on trunk the same work, where it exists, happens in the backend
writers themselves. That is what the comparison is for.
