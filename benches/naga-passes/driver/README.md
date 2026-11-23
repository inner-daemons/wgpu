# Driver benchmarks for the naga passes

These two benchmarks compare two builds of wgpu, `trunk` and a branch of naga IR passes, on a
real GPU and driver: how long it takes to create pipelines, and how fast the compiled shaders
run. Results depend a lot on the driver, so we would like to see them on as many OS, GPU and
driver combinations as possible. Thank you for running them.

You do not need to know wgpu. You need a Rust toolchain ([rustup](https://rustup.rs)), git, bash
and python3. On Windows use Git Bash.

## Run it

```sh
git clone <the fork URL> wgpu && cd wgpu
git fetch origin trunk-bench naga-passes-bench
benches/naga-passes/driver/run.sh
```

That is the whole thing. It creates two git worktrees of the two branches, builds both (the
first build takes about 5 to 15 minutes), runs both benchmarks on each, and writes a comparison.
Expected run time after the build: roughly 5 to 20 minutes per backend (an estimate; it depends
on how slow the driver's shader compiler is). Do not use the machine for other work meanwhile,
and keep a laptop on mains power.

Useful options:

```sh
benches/naga-passes/driver/run.sh --backends vulkan,gl   # which wgpu backends to run
benches/naga-passes/driver/run.sh --rounds 3             # repeat, alternating order (less noise)
benches/naga-passes/driver/run.sh --quick                # about a minute, for checking setup only
benches/naga-passes/driver/run.sh --out DIR              # where the results go
benches/naga-passes/driver/run.sh /path/to/trunk-bench /path/to/passes-bench   # existing checkouts
```

## Which backends run

The default is one backend per OS. Pass `--backends` to add more; each is a separate run and
exercises a different shader language and driver compiler.

| OS | default | also useful | shader language that reaches the driver |
|---|---|---|---|
| macOS | `metal` | `vulkan` if you have MoltenVK or KosmicKrisp | MSL (Metal compiler) |
| Linux | `vulkan` | `gl` | SPIR-V (Vulkan driver), GLSL (GL driver) |
| Windows | `dx12,vulkan` | `gl` | HLSL (DXC/FXC and the D3D driver), SPIR-V, GLSL |

To pick a specific GPU when there are several, set `WGPU_ADAPTER_NAME=<part of its name>`
(and `WGPU_POWER_PREF=high` or `low`). `wgpu-info.txt` in the results lists the adapters.

## What it measures

**Pipeline creation** (`pipeline_creation`): for each shader and each pipeline in it,
`Device::create_shader_module` and then `create_compute_pipeline` or `create_render_pipeline`,
timed separately, in milliseconds. This includes wgpu's parsing and validation, naga's backend
and the driver's compiler. Pipelines use automatic layouts, and render pipelines get vertex
buffers and color targets derived from the shader's inputs and outputs. The shaders are
`examples/features/src/**/*.wgsl` and the benchmark shaders below. No pipeline cache is used.

Drivers cache compiled shaders, which would make repeated creation of the same pipeline look
free, so every iteration compiles a slightly different shader. The benchmark shaders have an
`override BENCH_SALT` that is set to a different value each time. Other shaders get extra digits
on the first float literal in each entry point's body, and the entry points are renamed. The
`variation` field of a result says which was used. The first iteration of every pipeline is not
part of the statistics, but is reported as `first_ms`. The runner also turns off Mesa's and
NVIDIA's on-disk shader caches. If a driver keeps a cache that this does not defeat, times for
`variation: rename` shaders could be too small, on both checkouts.

**GPU runtime** (`gpu_runtime`): one dispatch of each benchmark shader (4096 workgroups of 64 or
256 invocations), 5 warm-up dispatches and then 40 timed ones, reporting median and p90. The time
comes from timestamp queries on the compute pass where the device supports them
(`timing_source: timestamp`) and from wall time around submit and poll otherwise (`wall`, which
includes some overhead). The shaders:

- `zeroinit_{1k,16k,max}_wg{64,256}`: a workgroup array of 1 KB, 16 KB, and the device's
  `max_compute_workgroup_storage_size` minus 1 KB. The shader writes the array, reads it back in
  a different order and writes a result, so the workgroup memory cannot be dropped;
- `zeroinit_2d_*`: a nested 2D workgroup array; `zeroinit_atomic_*`: workgroup atomics;
- `loops_*`: nested loops with trip counts that are only known at run time;
- `helpers_*`: many small functions with pointer parameters, nested several calls deep;
- `matcx2_*`: reads of `mat2x2`, `mat3x2` and `mat4x2` in a uniform buffer.

The output of the first dispatch is hashed (`checksum`), and the comparison reports `MISMATCH`
if the two builds compute different results.

## What to send back

Send the archive `naga-passes-results/<date>-<host>.tar.gz` that the script prints at the end,
or the directory it was made from. It contains:

- `comparison.md` and `comparison.csv`: the comparison table with absolute and % change;
- `<checkout>/<backend>/r<N>/*.json` and `*.log`: the raw results and the output of each run;
- `wgpu-info.txt`: the adapters and their limits and features (from `cargo run -p wgpu-info`);
- `run-info.txt`: date, `uname`, OS version, Rust version, the two commits.

Please add, in your message, what the files cannot say:

- the GPU model and the **driver version** as the vendor shows it (Windows: the NVIDIA, AMD or
  Intel driver version; Linux: `vulkaninfo --summary` and the Mesa or proprietary driver version;
  macOS: the chip and the macOS version);
- whether it is a laptop, on mains or battery, and any other load on the machine;
- anything that looked wrong while it ran.

## Reading the results: slowdown, error or crash

Every measurement has a status in the JSON, in `comparison.md` (last section) and in
`comparison.csv`:

- `ok`: measured. A slowdown shows up as a larger `median` or `p90`, with `status: ok` in both
  checkouts.
- `error`: wgpu or the driver reported an error for that shader. The message is in the `error`
  field. This is a validation error or a driver compile error, not a timing; it often means
  the device lacks a feature the shader needs (ray queries, mesh shaders, cooperative matrices),
  which is expected and shows up in both checkouts. If a case is `error` in only one checkout,
  that is a real difference and is the interesting one.
- `panic`: a Rust panic, caught. Please send the log.
- `crash`: the process died while measuring that case, which is what a driver crash or a GPU reset
  looks like. The runner starts the benchmark again, marks that case as `crash` and carries on
  with the next one, so one bad shader does not lose the run. The log
  (`*.log` next to the JSON) has the last output before the crash; on Windows also check for
  "device lost" or TDR messages, and on Linux `dmesg` for GPU resets.
- `skipped`: not attempted (mesh pipelines are not benchmarked).

Also look at:

- `first_ms` against the median: a large first time with a normal median is a driver cache or
  lazy compilation, not a slowdown;
- `n` and the spread between `median` and `p90`: a large spread means a noisy machine, so run
  again with `--rounds 3`;
- for `gpu_runtime`, `timing_source` should be the same in both checkouts; the comparison shows
  `ts` or `wall`.

## Settings

All optional environment variables: `NAGA_PASSES_PIPELINE_ITERS` (default 8),
`NAGA_PASSES_GPU_GROUPS` (4096), `NAGA_PASSES_GPU_RUNS` (40), `NAGA_PASSES_GPU_WARMUP` (5),
`NAGA_PASSES_GPU_CAP_MS` (8000, stop timing a shader after that long), `NAGA_PASSES_GPU_WALL`
(set it to use wall time even when timestamps exist), `NAGA_PASSES_FILTER` (comma separated
substrings of shader names), `NAGA_PASSES_SKIP` (substrings of cases to skip, for example one that
crashes the driver), `NAGA_PASSES_OUT` (where the JSON goes). To run one benchmark by hand:

```sh
WGPU_BACKEND=vulkan cargo bench -p wgpu-benchmark --bench naga-passes-driver -- naga_passes::gpu_runtime
```
