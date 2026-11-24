# Driver benchmarks for the naga passes

These two benchmarks compare two builds of wgpu, `trunk` and a branch of naga IR passes, on a
real GPU and driver: how long it takes to create pipelines, and how fast the compiled shaders
run. Results depend a lot on the driver, so we would like to see them on as many OS, GPU and
driver combinations as possible. Thank you for running them.

You do not need to know wgpu. You need a Rust toolchain ([rustup](https://rustup.rs)), git and
Python 3.9 or newer (standard library only). The Vulkan SDK is needed only for `--full-tests`.

- **Windows**: install Python from [python.org](https://www.python.org/downloads/) or with
  `winget install Python.Python.3.12`, [Git for Windows](https://gitforwindows.org) and rustup
  (with the MSVC build tools). Run the commands below as `py benches\naga-passes\driver\run.py`
  in PowerShell or `cmd`. The default backends are `dx12,vulkan`. If git complains about long
  paths, run `git config --global core.longpaths true`.
- **macOS**: Xcode command line tools (`xcode-select --install`), then `python3`. The default
  backend is `metal`.
- **Linux**: a C toolchain, `python3` and the GPU driver (Vulkan). The default backend is `vulkan`.

## Run it

```sh
git clone <the fork URL> wgpu && cd wgpu
git fetch origin trunk-bench naga-passes-bench
python3 benches/naga-passes/driver/run.py        # Windows: py benches\naga-passes\driver\run.py
```

That is the whole thing. It creates two git worktrees of the two branches, builds both (the
first build takes about 5 to 15 minutes), runs both benchmarks on each, and writes a comparison.
Expected run time after the build: roughly 5 to 20 minutes per backend (an estimate; it depends
on how slow the driver's shader compiler is). Do not use the machine for other work meanwhile,
and keep a laptop on mains power.

Useful options:

```sh
python3 benches/naga-passes/driver/run.py --backends vulkan,gl   # which wgpu backends to run
python3 benches/naga-passes/driver/run.py --rounds 3             # repeat, alternating order (less noise)
python3 benches/naga-passes/driver/run.py --quick                # about a minute, for checking setup only
python3 benches/naga-passes/driver/run.py --out DIR              # where the results go
python3 benches/naga-passes/driver/run.py --full-tests           # also run the test suite, see below
python3 benches/naga-passes/driver/run.py /path/to/trunk-bench /path/to/passes-bench   # existing checkouts
```

## Also run the full test suite (optional, long)

```sh
python3 benches/naga-passes/driver/run.py --full-tests           # benchmarks, then the tests
python3 benches/naga-passes/driver/run.py --full-tests --no-bench   # only the tests
```

This runs `cargo xtask test` in both checkouts, one after the other. It needs
[cargo-nextest](https://nexte.st) (`cargo install cargo-nextest --locked`) and, for most tests,
the [Vulkan SDK](https://vulkan.lunarg.com/sdk/home) on `PATH` (see `docs/testing.md`). It builds
the whole workspace twice and runs every GPU test on your default adapter, so expect roughly 20
to 60 minutes per checkout after the build, and more on slow machines (an estimate; it has not
been timed). The tests run on trunk as well so that failures that already exist there can be told
from new ones.

Please send back `tests/trunk.log` and `tests/combined.log` (or at least the two
`tests/*-failures.txt` lists), and `tests/failure-diff.txt`, which lists the tests that fail only
on the combined branch, only on trunk, and on both. A test that fails on both is probably a
problem of the device or the setup, not of the passes. The suite also runs the benchmarks once
each in test mode, so a benchmark that cannot run on your device can show up as a test failure;
say so if it does.

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

**Zero-init, native or polyfilled (Vulkan only).** wgpu-hal's Vulkan backend zero-initializes
workgroup memory natively (`VK_KHR_zero_initialize_workgroup_memory`, core in Vulkan 1.3) when
the device reports support for it, and otherwise naga polyfills it with stores and a barrier. On
the combined branch the polyfill is the shared zero-init pass, and on trunk it is the SPIR-V
writer's own code. Most desktop drivers support the native mode, so the polyfill would not be
measured on them. On Vulkan the runner therefore runs `gpu_runtime` a second time into
`<checkout>/vulkan-polyfill/`, with the environment variable
`WGPU_BENCH_ONLY_FORCE_ZERO_INIT_POLYFILL=1`, which makes the Vulkan backend behave as if the
device lacked the native feature. That variable exists only on the two benchmark branches, in
`wgpu-hal/src/vulkan/adapter.rs`, and does nothing unless it is set. The comparison has a
section "Zero-init on Vulkan" with native and polyfilled times side by side for each checkout.
If the device has no native support, the two runs are the same. On other backends this variant is
reported as `skipped` with the reason "N/A".

The output of the first dispatch is hashed (`checksum`), and the comparison reports `MISMATCH`
if the two builds compute different results.

## What to send back

Send the archive `naga-passes-results/<date>-<host>.tar.gz` that the script prints at the end,
or the directory it was made from. It contains:

- `comparison.md` and `comparison.csv`: the comparison table with absolute and % change;
- `<checkout>/<backend>/r<N>/*.json` and `*.log`: the raw results and the output of each run;
- `tests/` (only with `--full-tests`): the logs, failure lists and `failure-diff.txt` of the test
  runs;
- `wgpu-info.txt`: the adapters and their limits and features (from `cargo run -p wgpu-info`);
- `run-info.txt`: date, `uname`, OS version, CPU and GPU, Rust version, the two commits.

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
- `skipped`: not attempted: mesh pipelines are not benchmarked, or the shader needs a feature the
  device lacks (the reason says which). The device is created with only timestamp queries and the
  features the example shaders use, so a driver that mis-reports an unrelated feature still works.

A benchmark process that dies is started again and carries on after the case that killed it. If it
fails twice in a row with the same message and no new results (a startup panic, say), the runner
stops, writes the message to `<bench>.failed.json` next to the log, and `comparison.md` lists it
under "Benchmarks that produced no results". A missing file for one checkout is listed there too.

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
(set it to use wall time even when timestamps exist), `WGPU_BENCH_ONLY_FORCE_ZERO_INIT_POLYFILL` (see above), `NAGA_PASSES_FILTER` (comma separated
substrings of shader names), `NAGA_PASSES_SKIP` (substrings of cases to skip, for example one that
crashes the driver), `NAGA_PASSES_OUT` (where the JSON goes). `NAGA_PASSES_SALT_BASE` (0 to 89; by default a random offset is added to the variation per process, so that repeated runs and `--rounds` do not hit an on-disk shader cache). To run one benchmark by hand:

```sh
WGPU_BACKEND=vulkan cargo bench -p wgpu-benchmark --bench naga-passes-driver -- naga_passes::gpu_runtime
```
