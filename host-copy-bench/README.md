# host-copy-bench

Measures whether wgpu should write textures and buffers directly from the CPU
instead of going through a staging buffer and a copy on the GPU. Results are
collected from many devices to decide which backends and GPUs get these
optimizations.

## Running it

You need a Rust toolchain (<https://rustup.rs>) and Python 3.8 or later. From
the repository root:

```sh
python3 host-copy-bench/run.py --name "MacBook Air M2, macOS 15.3"
```

(On Windows, use `py` or `python` instead of `python3`.) Describe the machine
in `--name`: its model, GPU and OS.

The script builds the benchmark, runs every benchmark group three times, one
after another, and writes a `host-copy-bench-<date>.tar.gz` file to the current
directory. Please send that file back. It contains the results, the
benchmark's output, and the OS version, CPU model, memory size and git commit.
It doesn't contain your username, hostname or file paths. It takes a few minutes
on recent hardware.

Each benchmark group runs in its own process, so if a driver crashes, the other
groups' results are still collected, along with the crash log.

Arguments after `--` go to the benchmark itself, for example
`python3 host-copy-bench/run.py --name "..." -- --adapter radeon`. To run the
benchmark directly:

```sh
cargo run --release -p host-copy-bench -- --csv results.csv
```

Benchmark options:

- `--quick`: one size per benchmark, for a fast check that it works.
- `--adapter NAME`: only GPUs whose name contains `NAME` (case-insensitive).
- `--include-cpu`: also run software renderers such as llvmpipe and WARP.
- `--skip textures|buffers|scenarios|barrier`: skip a benchmark. Can be
  repeated.

Close other GPU-heavy programs while it runs, and keep laptops plugged in.

On macOS, Vulkan devices (MoltenVK or KosmicKrisp) are included if the
Vulkan loader can be found. If it is installed in `/usr/local/lib`, run with
`DYLD_FALLBACK_LIBRARY_PATH=/usr/local/lib`.

Android and iOS need extra setup that isn't covered here yet.

## What it measures

Each GPU is run on every backend it supports: Vulkan, Metal or DX12. OpenGL
is skipped.

### Textures

For `Rgba8Unorm`, `Rgba16Float`, `Bc1RgbaUnorm` and `Etc2Rgb8Unorm` (where
supported), at 256, 1024 and 4096 pixels square, with smooth content (which
lossless compression helps) and random content (which it can't help):

- `staging`: a normal texture written with `Queue::write_texture`.
- `host`: a `HOST_VISIBLE` texture written from the CPU with
  `copy_from_memory`. On Metal this is a `Shared` texture, on Vulkan an image
  with `HOST_TRANSFER` usage, and on DX12 a texture in a CPU-visible heap.
- `host+optimize` (Metal only): like `host`, followed by
  `optimizeContentsForGPUAccess`.

Metrics:

- `upload_fresh`: create a texture and fill it, until the GPU can use it.
- `upload_again`: overwrite a texture the GPU has already used.
  `upload_again_cpu` is the part spent copying on the CPU, and
  `upload_again_wait` the part spent waiting for the GPU (the copy, or the
  map).
- `readback`: read the whole texture back into memory, through a buffer or
  from the host.
- `first_sample` and `sample`: time for a compute shader to read every texel
  many times, the first time and afterwards. `sample_rate` is the same as a
  rate. This shows whether CPU-writable textures are slower for the GPU.

### Buffers

At 1, 16 and 64 MiB:

- `gpu-only`: a normal `STORAGE | COPY_DST` buffer, written through staging.
- `mappable`: a `STORAGE | MAP_WRITE` buffer (`MAPPABLE_PRIMARY_BUFFERS`),
  written directly.

Metrics: upload times as for textures, and `read_rate`, the GPU's read
bandwidth from the buffer. `buffer_readback` compares reading a buffer back
through a copy into a `MAP_READ` buffer with mapping a `STORAGE | MAP_READ`
buffer directly.

### Scenarios

- `render_target`: rendering into a `HOST_VISIBLE` texture vs a normal one,
  and sampling the result. Host copy usage can turn off framebuffer
  compression.
- `atlas`: 1000 small writes into a 4096x4096 atlas that the GPU samples after
  every batch, like a glyph atlas.
- `mips`: uploading a 2048x2048 texture with a full mip chain.
- `threads`: 1 to 8 threads uploading their own textures at once.
- `busy`: frame times of a GPU-heavy render loop that uploads and samples a new
  texture every frame, against the same loop without uploads.

### Vulkan host barrier

Vulkan only. Compares submit throughput, submit latency and the time per
frame of a simple 1080p render loop, with and without a memory barrier from
all GPU writes to the host at the end of every command buffer. Two rounds of
each, to show the noise.
