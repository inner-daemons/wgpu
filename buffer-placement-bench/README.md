# buffer-placement-bench

Measures what wgpu's new placement of mappable buffers costs or gains against
the placement it replaces, and records each GPU's memory layout. Results are
collected from many devices before the change goes upstream.

What changed:

| Usage | Before, on Vulkan | Now |
|---|---|---|
| `MAP_WRITE` without `MAP_READ` (staging, `write_buffer`, `write_texture`, `MAP_WRITE` with GPU usages) | CPU-visible VRAM if the GPU has it, else host memory | host memory, preferring memory that isn't `DEVICE_LOCAL` |
| `MAP_READ \| MAP_WRITE` | uncached, possibly VRAM | host, cached |
| `MAP_READ` only | host, cached | unchanged |
| no map usage | device-local | unchanged |

On DX12 and Metal only `MAP_READ | MAP_WRITE` changed: it used to be
write-combined and is now cached.

This crate lives on the `buffer-placement-bench` branch only. That branch adds
a switch to wgpu-hal: `WGPU_BENCH_LEGACY_BUFFER_PLACEMENT=1`, read when a
device is opened, restores the old placement for that device. The benchmark
opens two devices on every GPU, `new` and `legacy`, and runs the same work on
both, alternating between them every iteration.

## Running it

You need a Rust toolchain (<https://rustup.rs>) and Python 3.8 or later. From
the repository root:

```sh
python3 buffer-placement-bench/run.py
```

(On Windows, use `py` or `python` instead of `python3`.) The machine model,
CPU, GPUs, OS and whether it's on battery are detected automatically. To add
anything else worth knowing, such as whether resizable BAR is on, pass
`--note "..."`.

The script builds the benchmark, runs every benchmark group three times, one
after another, and writes a `buffer-placement-bench-<date>.tar.gz` file to the
current directory. Please send that file back. It contains the results, the
benchmark's output, each GPU's memory layout (`memory-properties.json`), the
machine's model, CPU, GPUs, memory size, OS version and power source, and the
git commit. It doesn't contain your username, hostname, file paths or any
serial numbers. It takes a few minutes on a recent machine.

Each benchmark group runs in its own process, so if a driver crashes, the other
groups' results are still collected, along with the crash log.

Arguments after `--` go to the benchmark itself, for example
`python3 buffer-placement-bench/run.py -- --adapter radeon`. To run the
benchmark directly:

```sh
cargo run --release -p buffer-placement-bench -- --csv results.csv
```

Benchmark options:

- `--quick`: one size per benchmark and 3 iterations, for a fast check that it
  works.
- `--adapter NAME`: only GPUs whose name contains `NAME` (case-insensitive).
- `--include-cpu`: also run software renderers such as llvmpipe and WARP.
- `--skip GROUP`: skip a group (`info`, `upload`, `write_texture`,
  `staging_belt`, `mappable_write`, `mappable_readwrite`, `controls`,
  `bar_pressure`). Can be repeated.
- `--memory-json FILE`: write the memory layouts as JSON.

The benchmark exits with 2 if no GPU was benchmarked and 3 if a data check
failed.

Close other GPU-heavy programs while it runs, and keep laptops plugged in.

On macOS, Vulkan devices (MoltenVK or KosmicKrisp) are included if the
Vulkan loader can be found. If it is installed in `/usr/local/lib`, run with
`DYLD_FALLBACK_LIBRARY_PATH=/usr/local/lib`.

## Output

Every row of the CSV has `adapter, backend, bench, case, variant, metric,
value, unit`. Variants are `new` and `legacy`. Each metric is the median over
the iterations, after a discarded warm-up. `<metric>_vs_legacy` rows (unit
`x`) are the median of `new / legacy` ratios from the same iteration: below 1
is better for times, above 1 for rates. Every case has a `check` row per
variant, 1 if the data came out right and 0 if not.

Without timestamp queries, GPU times are the wall time of the submission.
GPU times that can't be right (more than the wall time, or under half of it
for long work) are also replaced by the wall time.

## What it measures

Vulkan, DX12 and Metal, on every GPU. OpenGL is skipped.

### info

The adapter's memory: on Vulkan, the device type, every memory heap and
type, and which memory type each placement picks for a device-only,
`MAP_WRITE`, `MAP_READ` and `MAP_READ | MAP_WRITE` buffer with the
`memoryTypeBits` of a real `STORAGE | TRANSFER_SRC | TRANSFER_DST` buffer. The
new pick is a copy of wgpu-hal's `select_memory_type`; the legacy pick is
gpu-allocator's rule (the first type with the preferred flags, else the first
with the required ones). gpu-allocator also falls back from VRAM to host
memory when a `CpuToGpu` allocation fails, which a static pick can't show.
`memory-properties.json` has the same layout in the shape of wgpu-hal's
unit-test fixtures.

On DX12: `UMA` and `CacheCoherentUMA` from `D3D12_FEATURE_ARCHITECTURE1`, and
`GPUUploadHeapSupported` from `OPTIONS16`. On Metal: `hasUnifiedMemory`.

On DX12 and Metal, it also checks that the switch works: a `MAP_READ |
MAP_WRITE` buffer must be write-combined on the legacy device only (Metal
`cpuCacheMode`; DX12 heap type or CPU page property), and a `MAP_WRITE` buffer
on both. Vulkan has no query for a buffer's memory type.

### upload

`queue.write_buffer` into a device-only `STORAGE | COPY_DST` buffer: one write
of 4 KiB, 64 KiB, 1 MiB, 16 MiB or 64 MiB per submit, and 256 writes of 4 KiB
per submit.

- `cpu`: the `write_buffer` calls and the submit.
- `total`: until `poll` has waited for the GPU. `throughput` is the same as a
  rate.
- `gpu_copy`: wgpu-core's own copy from its staging buffer can't be timed, so
  this times the same copies from a `MAP_WRITE | COPY_SRC` buffer, the usages
  wgpu-core's staging buffers have, in a separate submission.

### write_texture

`queue.write_texture` of `Rgba8Unorm` textures of 256, 1024 and 4096 pixels
square, through wgpu-core's staging buffers. Same metrics as `upload`.

### staging_belt

`wgpu::util::StagingBelt` with 1 MiB chunks, writing 64 MiB per frame into a
device-only buffer. Same metrics; `gpu_copy` times the belt's own copies.

### mappable_write

Needs `MAPPABLE_PRIMARY_BUFFERS`. A `MAP_WRITE | STORAGE` buffer of 64 KiB,
1 MiB or 16 MiB used as a per-frame dynamic buffer: map it, write all of it
from the CPU, unmap, and run a shader that reads all of it once or 16 times
and checks a checksum. Also a 64 KiB `MAP_WRITE | UNIFORM` buffer, which each
of 4096 invocations reads in full.

- `cpu_map_write`: mapping and writing. `cpu_write_rate` is the write alone.
- `gpu_read`, `gpu_read_rate`: the shader.

### mappable_readwrite

Needs `MAPPABLE_PRIMARY_BUFFERS`. A `MAP_READ | MAP_WRITE | STORAGE` buffer of
64 KiB, 1 MiB or 16 MiB that a shader fills and the CPU then maps and sums.
`MAP_READ | STORAGE` buffers, whose placement didn't change, are the control.

- `gpu_write`, `gpu_write_rate`: the shader.
- `cpu_read`, `cpu_read_rate`: summing the mapped buffer.

### controls

Placement didn't change for these, so `new / legacy` should be about 1. They
show the noise.

- `readback 16 MiB`: copy into a `MAP_READ | COPY_DST` buffer, then map it and
  sum it on the CPU.
- `device storage 16 MiB, 16 reads`: a shader reading a device-only buffer.

### bar_pressure

Vulkan with `VK_EXT_memory_budget` only. 64 `queue.write_buffer` calls of
4 MiB (8 with `--quick`) pending before one submit. Records this process's usage of the heaps
that either placement takes staging memory from, before, while the writes are
pending and after they complete, and the allocator report's totals. With the
legacy placement on a GPU with a small CPU-visible VRAM window, this shows how
much of the window staging takes. Heap usage is for the whole process, which
has both devices open, so the deltas are what matters.
