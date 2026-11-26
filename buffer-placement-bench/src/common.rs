//! Devices, timing, data checks and result rows shared by every group.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use wgpu::util::DeviceExt as _;

pub const KIB: u64 = 1 << 10;
pub const MIB: u64 = 1 << 20;

/// Read by wgpu-hal when a device is opened, on the `buffer-placement-bench`
/// branch only.
const LEGACY_ENV: &str = "WGPU_BENCH_LEGACY_BUFFER_PLACEMENT";

/// Index 0 is the new placement, index 1 trunk's.
pub const VARIANTS: [&str; 2] = ["new", "legacy"];
pub const NEW: usize = 0;
pub const LEGACY: usize = 1;

pub fn size_name(bytes: u64) -> String {
    if bytes >= MIB && bytes.is_multiple_of(MIB) {
        format!("{} MiB", bytes / MIB)
    } else {
        format!("{} KiB", bytes / KIB)
    }
}

pub fn ms_since(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1e3
}

pub fn mb_per_s(bytes: u64, ms: f64) -> f64 {
    bytes as f64 / (ms * 1e3)
}

pub fn gb_per_s(bytes: u64, ms: f64) -> f64 {
    bytes as f64 / (ms * 1e6)
}

pub fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// The median of the ratios between measurements taken in the same iteration.
pub fn paired_ratio(values: &[f64], base: &[f64]) -> f64 {
    median(values.iter().zip(base).map(|(v, b)| v / b).collect())
}

/// Uses the GPU time unless it can't be right. Some drivers, such as Apple's
/// Metal driver, sometimes report pass timestamps far too close together. The
/// GPU time can't exceed the wall time around the submission, and for long
/// enough work it is most of it.
pub fn plausible_gpu_time(gpu: f64, wall: f64) -> f64 {
    if gpu <= wall && (gpu >= wall * 0.5 || wall < 2.0) && gpu > 0.0 {
        gpu
    } else {
        wall
    }
}

pub fn noise(len: usize, seed: u64) -> Vec<u8> {
    let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
    let mut out = Vec::with_capacity(len + 8);
    while out.len() < len {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        out.extend_from_slice(&state.to_le_bytes());
    }
    out.truncate(len);
    out
}

/// The wrapping sum of every little-endian u32, as `read.wgsl` computes it.
pub fn sum_words(bytes: &[u8]) -> u32 {
    match bytemuck::try_cast_slice::<u8, u32>(bytes) {
        Ok(words) => words.iter().fold(0u32, |acc, &w| acc.wrapping_add(w)),
        Err(_) => bytes.chunks_exact(4).fold(0u32, |acc, w| {
            acc.wrapping_add(u32::from_le_bytes(w.try_into().unwrap()))
        }),
    }
}

/// The word `fill.wgsl` writes at `index`.
pub fn pattern(index: u32, seed: u32) -> u32 {
    index.wrapping_mul(0x9e37_79b1) ^ seed
}

pub fn pattern_sum(words: u32, seed: u32) -> u32 {
    (0..words).fold(0u32, |acc, i| acc.wrapping_add(pattern(i, seed)))
}

// ---------------------------------------------------------------------------
// Rows
// ---------------------------------------------------------------------------

pub struct Row {
    pub adapter: String,
    pub backend: String,
    pub bench: &'static str,
    pub case: String,
    pub variant: String,
    pub metric: String,
    pub value: f64,
    pub unit: &'static str,
}

/// Collects the rows for one adapter.
pub struct Rec {
    adapter: String,
    backend: String,
    pub rows: Vec<Row>,
    pub failures: Vec<String>,
}

impl Rec {
    pub fn new(info: &wgpu::AdapterInfo) -> Self {
        Rec {
            adapter: info.name.clone(),
            backend: format!("{:?}", info.backend),
            rows: Vec::new(),
            failures: Vec::new(),
        }
    }

    pub fn record(
        &mut self,
        bench: &'static str,
        case: &str,
        variant: &str,
        metric: &str,
        value: f64,
        unit: &'static str,
    ) {
        self.rows.push(Row {
            adapter: self.adapter.clone(),
            backend: self.backend.clone(),
            bench,
            case: case.to_owned(),
            variant: variant.to_owned(),
            metric: metric.to_owned(),
            value,
            unit,
        });
    }

    /// Records a `check` row: 1 if the data was right, 0 if not.
    pub fn check(
        &mut self,
        bench: &'static str,
        case: &str,
        variant: &str,
        result: &Result<(), String>,
    ) {
        if let Err(e) = result {
            let message = format!("{bench} {case} {variant}: {e}");
            eprintln!("  CHECK FAILED: {message}");
            self.failures.push(message);
        }
        self.record(
            bench,
            case,
            variant,
            "check",
            result.is_ok() as u32 as f64,
            "bool",
        );
    }

    /// Records the median of each metric per variant, the paired ratio
    /// `new / legacy` as `<metric>_vs_legacy`, and a check row per variant.
    pub fn finish(&mut self, bench: &'static str, case: &str, run: Run) {
        for (metric, unit, values) in &run.series.metrics {
            for (v, name) in VARIANTS.iter().enumerate() {
                if !values[v].is_empty() {
                    self.record(bench, case, name, metric, median(values[v].clone()), unit);
                }
            }
            if !values[NEW].is_empty() && values[NEW].len() == values[LEGACY].len() {
                self.record(
                    bench,
                    case,
                    VARIANTS[NEW],
                    &format!("{metric}_vs_legacy"),
                    paired_ratio(&values[NEW], &values[LEGACY]),
                    "x",
                );
            }
        }
        for (v, result) in run.results.iter().enumerate() {
            self.check(bench, case, VARIANTS[v], result);
        }
    }
}

// ---------------------------------------------------------------------------
// Interleaved measurements
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct Series {
    metrics: Vec<(&'static str, &'static str, [Vec<f64>; 2])>,
}

impl Series {
    fn push(&mut self, variant: usize, metric: &'static str, unit: &'static str, value: f64) {
        let index = match self.metrics.iter().position(|(m, ..)| *m == metric) {
            Some(i) => i,
            None => {
                self.metrics.push((metric, unit, [Vec::new(), Vec::new()]));
                self.metrics.len() - 1
            }
        };
        self.metrics[index].2[variant].push(value);
    }
}

/// One iteration's measurements for one variant.
#[derive(Default)]
pub struct Sample(Vec<(&'static str, &'static str, f64)>);

impl Sample {
    pub fn add(&mut self, metric: &'static str, unit: &'static str, value: f64) {
        self.0.push((metric, unit, value));
    }
}

pub struct Run {
    series: Series,
    pub results: [Result<(), String>; 2],
}

impl Run {
    /// Marks `variant` as failed unless it already failed.
    pub fn fail(&mut self, variant: usize, error: String) {
        if self.results[variant].is_ok() {
            self.results[variant] = Err(error);
        }
    }
}

/// Runs `step(variant, iteration, sample)` for both variants, alternating which
/// goes first, `iterations + 1` times. The first iteration is a warm-up and is
/// discarded. A variant stops at its first error.
pub fn interleave(
    iterations: usize,
    mut step: impl FnMut(usize, usize, &mut Sample) -> Result<(), String>,
) -> Run {
    let mut run = Run {
        series: Series::default(),
        results: [Ok(()), Ok(())],
    };
    for i in 0..=iterations {
        let order = if i % 2 == 0 {
            [NEW, LEGACY]
        } else {
            [LEGACY, NEW]
        };
        for v in order {
            if run.results[v].is_err() {
                continue;
            }
            let mut sample = Sample::default();
            match step(v, i, &mut sample) {
                Ok(()) if i > 0 => {
                    for (metric, unit, value) in sample.0 {
                        run.series.push(v, metric, unit, value);
                    }
                }
                Ok(()) => {}
                Err(e) => run.results[v] = Err(e),
            }
        }
    }
    run
}

pub fn iterations(quick: bool, bytes: u64) -> usize {
    if quick {
        3
    } else if bytes <= MIB {
        100
    } else if bytes <= 16 * MIB {
        40
    } else {
        20
    }
}

// ---------------------------------------------------------------------------
// Devices
// ---------------------------------------------------------------------------

struct Kernels {
    read: wgpu::ComputePipeline,
    fill: wgpu::ComputePipeline,
    uniform_read: wgpu::ComputePipeline,
}

impl Kernels {
    fn new(device: &wgpu::Device) -> Self {
        let pipeline = |label, module: wgpu::ShaderModuleDescriptor<'_>| {
            let module = device.create_shader_module(module);
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(label),
                layout: None,
                module: &module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        Kernels {
            read: pipeline("read", wgpu::include_wgsl!("shaders/read.wgsl")),
            fill: pipeline("fill", wgpu::include_wgsl!("shaders/fill.wgsl")),
            uniform_read: pipeline(
                "uniform_read",
                wgpu::include_wgsl!("shaders/uniform_read.wgsl"),
            ),
        }
    }
}

struct Timestamps {
    set: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    readback: wgpu::Buffer,
}

/// A 16-byte device-only buffer that shaders add a checksum into, and a buffer
/// to read it back from.
pub struct Sink {
    pub buffer: wgpu::Buffer,
    readback: wgpu::Buffer,
}

impl Sink {
    pub fn clear(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.clear_buffer(&self.buffer, 0, None);
    }

    pub fn copy_out(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_buffer_to_buffer(&self.buffer, 0, &self.readback, 0, 16);
    }

    pub fn read(&self, ctx: &Ctx) -> u32 {
        ctx.map(&self.readback, wgpu::MapMode::Read);
        let value = {
            let view = self.readback.get_mapped_range(..).unwrap();
            u32::from_le_bytes(view[..4].try_into().unwrap())
        };
        self.readback.unmap();
        value
    }
}

pub struct Ctx {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub quick: bool,
    errors: Arc<AtomicBool>,
    kernels: Kernels,
    timestamps: Option<Timestamps>,
}

impl Ctx {
    /// Opens a device with the new placement, or with trunk's if `legacy`.
    pub fn new(adapter: &wgpu::Adapter, legacy: bool, quick: bool) -> Result<Self, String> {
        let wanted = wgpu::Features::MAPPABLE_PRIMARY_BUFFERS | wgpu::Features::TIMESTAMP_QUERY;
        // SAFETY: no other threads exist yet, and wgpu-hal reads the variable
        // only while the device is opened.
        unsafe {
            if legacy {
                std::env::set_var(LEGACY_ENV, "1");
            } else {
                std::env::remove_var(LEGACY_ENV);
            }
        }
        let result = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some(VARIANTS[legacy as usize]),
            required_features: adapter.features() & wanted,
            required_limits: adapter.limits(),
            default_queue: wgpu::QueueDescriptor { label: None },
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }));
        unsafe { std::env::remove_var(LEGACY_ENV) };
        let (device, queue) = result.map_err(|e| format!("failed to create a device: {e}"))?;

        let errors = Arc::new(AtomicBool::new(false));
        let flag = errors.clone();
        device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
            eprintln!("  wgpu error: {error}");
            flag.store(true, Ordering::Relaxed);
        }));
        let timestamps = device
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY)
            .then(|| Timestamps {
                set: device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: None,
                    ty: wgpu::QueryType::Timestamp,
                    count: 2,
                }),
                resolve: device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: 16,
                    usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }),
                readback: device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: 16,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            });
        Ok(Ctx {
            kernels: Kernels::new(&device),
            device,
            queue,
            quick,
            errors,
            timestamps,
        })
    }

    pub fn mappable_primary_buffers(&self) -> bool {
        self.device
            .features()
            .contains(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS)
    }

    pub fn wait(&self) {
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("poll failed");
    }

    /// Whether a wgpu error was reported since the last call.
    pub fn took_error(&self) -> bool {
        self.errors.swap(false, Ordering::Relaxed)
    }

    pub fn buffer(&self, size: u64, usage: wgpu::BufferUsages) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage,
            mapped_at_creation: false,
        })
    }

    pub fn buffer_with(&self, contents: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents,
                usage,
            })
    }

    pub fn sink(&self) -> Sink {
        Sink {
            buffer: self.buffer(
                16,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
            ),
            readback: self.buffer(
                16,
                wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            ),
        }
    }

    pub fn map(&self, buffer: &wgpu::Buffer, mode: wgpu::MapMode) {
        let done = Arc::new(AtomicBool::new(false));
        let flag = done.clone();
        buffer.map_async(mode, .., move |r| {
            r.unwrap();
            flag.store(true, Ordering::Release);
        });
        while !done.load(Ordering::Acquire) {
            self.wait();
        }
    }

    pub fn submit_and_wait(&self, encoder: wgpu::CommandEncoder) -> f64 {
        let start = Instant::now();
        self.queue.submit(Some(encoder.finish()));
        self.wait();
        ms_since(start)
    }

    pub fn encoder(&self) -> wgpu::CommandEncoder {
        self.device.create_command_encoder(&Default::default())
    }

    /// Timestamp writes at the beginning and end of a compute pass.
    pub fn pass_timestamps(&self) -> Option<wgpu::ComputePassTimestampWrites<'_>> {
        self.timestamps
            .as_ref()
            .map(|t| wgpu::ComputePassTimestampWrites {
                query_set: &t.set,
                beginning_of_pass_write_index: Some(0),
                end_of_pass_write_index: Some(1),
            })
    }

    /// Writes timestamp `index` with an empty compute pass. End-of-pass
    /// timestamps wait for all earlier work, so two marks bracket the
    /// commands recorded between them, such as copies.
    pub fn mark(&self, encoder: &mut wgpu::CommandEncoder, index: u32) {
        let Some(t) = &self.timestamps else {
            return;
        };
        encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                query_set: &t.set,
                beginning_of_pass_write_index: None,
                end_of_pass_write_index: Some(index),
            }),
        });
    }

    pub fn resolve_timestamps(&self, encoder: &mut wgpu::CommandEncoder) {
        if let Some(t) = &self.timestamps {
            encoder.resolve_query_set(&t.set, 0..2, &t.resolve, 0);
            encoder.copy_buffer_to_buffer(&t.resolve, 0, &t.readback, 0, 16);
        }
    }

    /// The GPU time between timestamps 0 and 1 of the last submission, or
    /// `wall` if timestamps aren't supported or can't be right.
    pub fn gpu_ms(&self, wall: f64) -> f64 {
        let Some(t) = &self.timestamps else {
            return wall;
        };
        self.map(&t.readback, wgpu::MapMode::Read);
        let ticks: [u64; 2] = {
            let view = t.readback.get_mapped_range(..).unwrap();
            bytemuck::pod_read_unaligned(&view[..16])
        };
        t.readback.unmap();
        let gpu =
            ticks[1].wrapping_sub(ticks[0]) as f64 * self.queue.get_timestamp_period() as f64 / 1e6;
        plausible_gpu_time(gpu, wall)
    }

    fn workgroups(count: u64) -> u32 {
        count.div_ceil(256).clamp(1, 1024) as u32
    }

    fn params(&self, words: [u32; 4]) -> wgpu::Buffer {
        self.buffer_with(bytemuck::cast_slice(&words), wgpu::BufferUsages::UNIFORM)
    }

    /// A bind group for `read.wgsl` over `size` bytes of `buffer` from `offset`.
    pub fn read_bind_group(
        &self,
        buffer: &wgpu::Buffer,
        offset: u64,
        size: u64,
        sink: &Sink,
    ) -> (wgpu::BindGroup, u32) {
        let count = size / 16;
        let params = self.params([count as u32, 0, 0, 0]);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.kernels.read.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer,
                        offset,
                        size: wgpu::BufferSize::new(size),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: sink.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params.as_entire_binding(),
                },
            ],
        });
        (bind_group, Self::workgroups(count))
    }

    /// Records `dispatches` passes of `read.wgsl` over a bind group from
    /// [`Ctx::read_bind_group`]. Separate dispatches keep the compiler from
    /// hoisting the reads out of a loop.
    pub fn dispatch_read(
        &self,
        pass: &mut wgpu::ComputePass<'_>,
        bind_group: &wgpu::BindGroup,
        workgroups: u32,
        dispatches: u32,
    ) {
        pass.set_pipeline(&self.kernels.read);
        pass.set_bind_group(0, bind_group, &[]);
        for _ in 0..dispatches {
            pass.dispatch_workgroups(workgroups, 1, 1);
        }
    }

    pub fn uniform_bind_group(&self, buffer: &wgpu::Buffer, sink: &Sink) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.kernels.uniform_read.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: sink.buffer.as_entire_binding(),
                },
            ],
        })
    }

    pub fn dispatch_uniform_read(
        &self,
        pass: &mut wgpu::ComputePass<'_>,
        bind_group: &wgpu::BindGroup,
        workgroups: u32,
        dispatches: u32,
    ) {
        pass.set_pipeline(&self.kernels.uniform_read);
        pass.set_bind_group(0, bind_group, &[]);
        for _ in 0..dispatches {
            pass.dispatch_workgroups(workgroups, 1, 1);
        }
    }

    /// A bind group for `fill.wgsl` over the whole of `buffer`.
    pub fn fill_bind_group(&self, buffer: &wgpu::Buffer, seed: u32) -> (wgpu::BindGroup, u32) {
        let count = buffer.size() / 16;
        let params = self.params([count as u32, seed, 0, 0]);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.kernels.fill.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: params.as_entire_binding(),
                },
            ],
        });
        (bind_group, Self::workgroups(count))
    }

    pub fn dispatch_fill(
        &self,
        pass: &mut wgpu::ComputePass<'_>,
        bind_group: &wgpu::BindGroup,
        workgroups: u32,
    ) {
        pass.set_pipeline(&self.kernels.fill);
        pass.set_bind_group(0, bind_group, &[]);
        pass.dispatch_workgroups(workgroups, 1, 1);
    }

    /// Fills a `STORAGE` buffer with `fill.wgsl` and waits.
    pub fn fill(&self, buffer: &wgpu::Buffer, seed: u32) {
        let (bind_group, workgroups) = self.fill_bind_group(buffer, seed);
        let mut encoder = self.encoder();
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            self.dispatch_fill(&mut pass, &bind_group, workgroups);
        }
        self.submit_and_wait(encoder);
    }

    /// The wrapping sum of every u32 of a `STORAGE` buffer, computed on the GPU.
    pub fn checksum(&self, buffer: &wgpu::Buffer, size: u64) -> u32 {
        let limits = self.device.limits();
        let chunk = (64 * MIB)
            .min(limits.max_storage_buffer_binding_size / MIB * MIB)
            .max(MIB);
        let sink = self.sink();
        let mut encoder = self.encoder();
        let bind_groups: Vec<_> = (0..size)
            .step_by(chunk as usize)
            .map(|offset| self.read_bind_group(buffer, offset, chunk.min(size - offset), &sink))
            .collect();
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            for (bind_group, workgroups) in &bind_groups {
                self.dispatch_read(&mut pass, bind_group, *workgroups, 1);
            }
        }
        sink.copy_out(&mut encoder);
        self.submit_and_wait(encoder);
        sink.read(self)
    }

    /// Checks that a `STORAGE` buffer holds `expected`, by checksum.
    pub fn check_contents(&self, buffer: &wgpu::Buffer, expected: &[u8]) -> Result<(), String> {
        let got = self.checksum(buffer, expected.len() as u64);
        let want = sum_words(expected);
        if got == want {
            Ok(())
        } else {
            Err(format!("checksum {got:#010x}, expected {want:#010x}"))
        }
    }

    /// Turns a wgpu error reported since the last call into an error.
    pub fn error_check(&self) -> Result<(), String> {
        if self.took_error() {
            Err("wgpu reported an error".into())
        } else {
            Ok(())
        }
    }
}

/// The two devices on one adapter, indexed by [`NEW`] and [`LEGACY`].
pub struct Pair {
    pub ctxs: [Ctx; 2],
}

impl Pair {
    pub fn new(adapter: &wgpu::Adapter, quick: bool) -> Result<Self, String> {
        let new = Ctx::new(adapter, false, quick)?;
        let legacy = Ctx::new(adapter, true, quick)?;
        Ok(Pair {
            ctxs: [new, legacy],
        })
    }

    pub fn quick(&self) -> bool {
        self.ctxs[NEW].quick
    }

    /// Marks any variant whose device reported a wgpu error as failed.
    pub fn collect_errors(&self, run: &mut Run) {
        for (v, ctx) in self.ctxs.iter().enumerate() {
            if let Err(e) = ctx.error_check() {
                run.fail(v, e);
            }
        }
    }
}
