//! Benchmarks for deciding where wgpu should use host copies.
//!
//! See `README.md` for what each benchmark measures and how to run it.

use std::{
    fmt::Write as _,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use wgpu::util::DeviceExt as _;

const MB: u64 = 1 << 20;

struct Options {
    quick: bool,
    csv: Option<String>,
    adapter_filter: Option<String>,
    include_cpu: bool,
    skip: Vec<String>,
}

impl Options {
    fn parse() -> Self {
        let mut options = Options {
            quick: false,
            csv: None,
            adapter_filter: None,
            include_cpu: false,
            skip: Vec::new(),
        };
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--quick" => options.quick = true,
                "--csv" => options.csv = args.next(),
                "--adapter" => options.adapter_filter = args.next().map(|s| s.to_lowercase()),
                "--include-cpu" => options.include_cpu = true,
                "--skip" => options.skip.extend(args.next()),
                "--help" | "-h" => {
                    println!(
                        "usage: host-copy-bench [--quick] [--csv FILE] [--adapter NAME] \
                         [--include-cpu] [--skip textures|buffers|barrier]..."
                    );
                    std::process::exit(0);
                }
                other => {
                    eprintln!("unknown argument {other:?}, see --help");
                    std::process::exit(2);
                }
            }
        }
        options
    }

    fn runs(&self, bench: &str) -> bool {
        !self.skip.iter().any(|s| s == bench)
    }
}

struct Row {
    adapter: String,
    backend: String,
    bench: &'static str,
    case: String,
    variant: &'static str,
    metric: &'static str,
    value: f64,
    unit: &'static str,
}

struct Ctx {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    timestamps: bool,
    quick: bool,
    errors: Arc<AtomicBool>,
    adapter_name: String,
    backend: String,
    rows: Vec<Row>,
}

impl Ctx {
    fn new(adapter: &wgpu::Adapter, quick: bool) -> Option<Self> {
        let wanted = wgpu::Features::HOST_TEXTURE_COPY
            | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS
            | wgpu::Features::TIMESTAMP_QUERY
            | wgpu::Features::TEXTURE_COMPRESSION_BC
            | wgpu::Features::TEXTURE_COMPRESSION_ETC2;
        let (device, queue) = request_device(adapter, adapter.features() & wanted)?;
        let errors = Arc::new(AtomicBool::new(false));
        let flag = errors.clone();
        device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
            eprintln!("  wgpu error: {error}");
            flag.store(true, Ordering::Relaxed);
        }));
        let info = adapter.get_info();
        Some(Ctx {
            adapter: adapter.clone(),
            timestamps: device.features().contains(wgpu::Features::TIMESTAMP_QUERY),
            device,
            queue,
            quick,
            errors,
            adapter_name: info.name.clone(),
            backend: format!("{:?}", info.backend),
            rows: Vec::new(),
        })
    }

    fn record(
        &mut self,
        bench: &'static str,
        case: &str,
        variant: &'static str,
        metric: &'static str,
        value: f64,
        unit: &'static str,
    ) {
        self.rows.push(Row {
            adapter: self.adapter_name.clone(),
            backend: self.backend.clone(),
            bench,
            case: case.to_owned(),
            variant,
            metric,
            value,
            unit,
        });
    }

    fn wait(&self) {
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("poll failed");
    }

    fn submit_empty_and_wait(&self) {
        self.queue.submit(None);
        self.wait();
    }

    /// Runs a compute pass and returns its GPU time in milliseconds, or its wall
    /// time if timestamps are unavailable.
    fn time_compute(&self, record: impl FnOnce(&mut wgpu::ComputePass<'_>)) -> f64 {
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let queries = self.timestamps.then(|| {
            let set = self.device.create_query_set(&wgpu::QuerySetDescriptor {
                label: None,
                ty: wgpu::QueryType::Timestamp,
                count: 2,
            });
            let resolve = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: 16,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: 16,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            (set, resolve, readback)
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: queries.as_ref().map(|(set, ..)| {
                    wgpu::ComputePassTimestampWrites {
                        query_set: set,
                        beginning_of_pass_write_index: Some(0),
                        end_of_pass_write_index: Some(1),
                    }
                }),
            });
            record(&mut pass);
        }
        if let Some((set, resolve, readback)) = &queries {
            encoder.resolve_query_set(set, 0..2, resolve, 0);
            encoder.copy_buffer_to_buffer(resolve, 0, readback, 0, 16);
        }
        let start = Instant::now();
        self.queue.submit(Some(encoder.finish()));
        self.wait();
        let wall = start.elapsed().as_secs_f64() * 1e3;
        let Some((_, _, readback)) = queries else {
            return wall;
        };
        readback.map_async(wgpu::MapMode::Read, .., |r| r.unwrap());
        self.wait();
        let ticks: [u64; 2] = {
            let view = readback.get_mapped_range(..).unwrap();
            bytemuck::pod_read_unaligned(&view[..16])
        };
        readback.unmap();
        let gpu =
            ticks[1].wrapping_sub(ticks[0]) as f64 * self.queue.get_timestamp_period() as f64 / 1e6;
        // Some drivers report zero or garbage for pass timestamps.
        if gpu > 0.0 && gpu < wall * 10.0 {
            gpu
        } else {
            wall
        }
    }

    fn uniform(&self, params: [u32; 4]) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            })
    }

    fn sink(&self) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 16,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        })
    }

    fn took_error(&self) -> bool {
        self.errors.swap(false, Ordering::Relaxed)
    }
}

fn request_device(
    adapter: &wgpu::Adapter,
    features: wgpu::Features,
) -> Option<(wgpu::Device, wgpu::Queue)> {
    let result = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: features,
        required_limits: adapter.limits(),
        default_queue: wgpu::QueueDescriptor { label: None },
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }));
    match result {
        Ok(device) => Some(device),
        Err(e) => {
            eprintln!("  failed to create a device: {e}");
            None
        }
    }
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

fn wait_for_map(ctx: &Ctx, done: &AtomicBool) {
    while !done.load(Ordering::Acquire) {
        ctx.wait();
    }
}

// ---------------------------------------------------------------------------
// Device information
// ---------------------------------------------------------------------------

fn describe(adapter: &wgpu::Adapter, out: &mut String) {
    let info = adapter.get_info();
    let _ = writeln!(out, "## {} ({:?})", info.name, info.backend);
    let _ = writeln!(out);
    let _ = writeln!(out, "- device type: {:?}", info.device_type);
    let _ = writeln!(out, "- driver: {} {}", info.driver, info.driver_info);
    let _ = writeln!(
        out,
        "- vendor/device: {:#x}/{:#x}",
        info.vendor, info.device
    );
    let features = adapter.features();
    for (name, feature) in [
        ("HOST_TEXTURE_COPY", wgpu::Features::HOST_TEXTURE_COPY),
        (
            "MAPPABLE_PRIMARY_BUFFERS",
            wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
        ),
        ("TIMESTAMP_QUERY", wgpu::Features::TIMESTAMP_QUERY),
    ] {
        let _ = writeln!(out, "- {name}: {}", features.contains(feature));
    }
    for (format, _) in TEXTURE_FORMATS {
        let flags = adapter.get_texture_format_features(*format);
        let _ = writeln!(
            out,
            "- {format:?}: HOST_VISIBLE {}, HOST_VISIBLE_OPTIMAL {}",
            flags
                .allowed_usages
                .contains(wgpu::TextureUsages::HOST_VISIBLE),
            flags
                .flags
                .contains(wgpu::TextureFormatFeatureFlags::HOST_VISIBLE_OPTIMAL),
        );
    }
    describe_vulkan(adapter, out);
    describe_metal(adapter, out);
    let _ = writeln!(out);
}

#[cfg(not(target_arch = "wasm32"))]
fn describe_vulkan(adapter: &wgpu::Adapter, out: &mut String) {
    use ash::vk;
    let Some(hal) = (unsafe { adapter.as_hal::<wgpu::hal::api::Vulkan>() }) else {
        return;
    };
    let instance = hal.shared_instance().raw_instance();
    let props =
        unsafe { instance.get_physical_device_memory_properties(hal.raw_physical_device()) };
    let heaps = &props.memory_heaps[..props.memory_heap_count as usize];
    for (i, heap) in heaps.iter().enumerate() {
        let _ = writeln!(
            out,
            "- memory heap {i}: {} MiB, {:?}",
            heap.size / MB,
            heap.flags
        );
    }
    let types = &props.memory_types[..props.memory_type_count as usize];
    for (i, ty) in types.iter().enumerate() {
        let _ = writeln!(
            out,
            "- memory type {i}: heap {}, {:?}",
            ty.heap_index, ty.property_flags
        );
    }
    let all_device_local_visible = types
        .iter()
        .filter(|t| {
            t.property_flags
                .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
        })
        .all(|t| {
            t.property_flags
                .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
        });
    let _ = writeln!(
        out,
        "- every DEVICE_LOCAL memory type is HOST_VISIBLE: {all_device_local_visible}"
    );
}

#[cfg(target_arch = "wasm32")]
fn describe_vulkan(_: &wgpu::Adapter, _: &mut String) {}

#[cfg(target_vendor = "apple")]
fn describe_metal(adapter: &wgpu::Adapter, out: &mut String) {
    use objc2_metal::MTLDevice as _;
    let Some(hal) = (unsafe { adapter.as_hal::<wgpu::hal::api::Metal>() }) else {
        return;
    };
    let device = hal.raw_device();
    let _ = writeln!(out, "- hasUnifiedMemory: {}", device.hasUnifiedMemory());
    let _ = writeln!(
        out,
        "- recommendedMaxWorkingSetSize: {} MiB",
        device.recommendedMaxWorkingSetSize() / MB
    );
}

#[cfg(not(target_vendor = "apple"))]
fn describe_metal(_: &wgpu::Adapter, _: &mut String) {}

// ---------------------------------------------------------------------------
// Textures
// ---------------------------------------------------------------------------

const TEXTURE_FORMATS: &[(wgpu::TextureFormat, wgpu::Features)] = &[
    (wgpu::TextureFormat::Rgba8Unorm, wgpu::Features::empty()),
    (wgpu::TextureFormat::Rgba16Float, wgpu::Features::empty()),
    (
        wgpu::TextureFormat::Bc1RgbaUnorm,
        wgpu::Features::TEXTURE_COMPRESSION_BC,
    ),
    (
        wgpu::TextureFormat::Etc2Rgb8Unorm,
        wgpu::Features::TEXTURE_COMPRESSION_ETC2,
    ),
];

#[derive(Clone, Copy, PartialEq)]
enum Content {
    /// Smooth gradients, or repeated blocks for compressed formats. Lossless
    /// framebuffer-style compression should help.
    Smooth,
    /// Random bytes. Lossless compression can't help.
    Noise,
}

impl Content {
    fn name(self) -> &'static str {
        match self {
            Content::Smooth => "smooth",
            Content::Noise => "noise",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum TextureVariant {
    /// `COPY_DST` texture written with `Queue::write_texture`, which goes
    /// through a staging buffer and a copy on the GPU.
    Staging,
    /// `HOST_VISIBLE` texture written with `copy_from_memory`.
    Host,
    /// Metal only: like `Host`, followed by `optimizeContentsForGPUAccess`.
    HostOptimized,
}

impl TextureVariant {
    fn name(self) -> &'static str {
        match self {
            TextureVariant::Staging => "staging",
            TextureVariant::Host => "host",
            TextureVariant::HostOptimized => "host+optimize",
        }
    }
}

struct TextureCase {
    format: wgpu::TextureFormat,
    size: u32,
    content: Content,
    data: Vec<u8>,
    layout: wgpu::TexelCopyBufferLayout,
}

impl TextureCase {
    fn new(format: wgpu::TextureFormat, size: u32, content: Content) -> Self {
        let (block_w, block_h) = format.block_dimensions();
        let block_bytes = format.block_copy_size(None).unwrap();
        let blocks_x = size / block_w;
        let blocks_y = size / block_h;
        let bytes_per_row = blocks_x * block_bytes;
        let len = (bytes_per_row * blocks_y) as usize;
        let data = match content {
            Content::Noise => noise(len, size as u64),
            Content::Smooth => smooth(format, size, len, block_bytes as usize),
        };
        TextureCase {
            format,
            size,
            content,
            data,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(blocks_y),
            },
        }
    }

    fn name(&self) -> String {
        format!(
            "{:?} {}x{} {}",
            self.format,
            self.size,
            self.size,
            self.content.name()
        )
    }

    fn extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.size,
            height: self.size,
            depth_or_array_layers: 1,
        }
    }

    fn create(&self, ctx: &Ctx, variant: TextureVariant, mapped: bool) -> wgpu::Texture {
        let usage = match variant {
            TextureVariant::Staging => {
                wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING
            }
            TextureVariant::Host | TextureVariant::HostOptimized => {
                wgpu::TextureUsages::HOST_VISIBLE | wgpu::TextureUsages::TEXTURE_BINDING
            }
        };
        ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: self.extent(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage,
            view_formats: &[],
            texture_binding_view_dimension: None,
            mapped_at_creation: mapped,
        })
    }

    fn host_write(&self, texture: &wgpu::Texture) {
        texture.get_mapped_texture_mut().unwrap().copy_from_memory(
            wgpu::TexelCopyTextureInfoBase {
                texture: (),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.data,
            self.layout,
            self.extent(),
        );
    }

    /// Creates a texture and fills it, returning it and the time taken until
    /// the GPU could use it.
    fn upload_fresh(&self, ctx: &Ctx, variant: TextureVariant) -> (wgpu::Texture, f64) {
        let start = Instant::now();
        let texture = match variant {
            TextureVariant::Staging => {
                let texture = self.create(ctx, variant, false);
                ctx.queue.write_texture(
                    texture.as_image_copy(),
                    &self.data,
                    self.layout,
                    self.extent(),
                );
                ctx.submit_empty_and_wait();
                texture
            }
            TextureVariant::Host | TextureVariant::HostOptimized => {
                let texture = self.create(ctx, variant, true);
                self.host_write(&texture);
                texture.unmap();
                if variant == TextureVariant::HostOptimized {
                    metal_optimize(ctx, &texture);
                }
                texture
            }
        };
        (texture, start.elapsed().as_secs_f64() * 1e3)
    }

    /// Overwrites a texture the GPU has used, returning the time taken until the
    /// GPU could use it again.
    fn upload_again(&self, ctx: &Ctx, texture: &wgpu::Texture, variant: TextureVariant) -> f64 {
        let start = Instant::now();
        match variant {
            TextureVariant::Staging => {
                ctx.queue.write_texture(
                    texture.as_image_copy(),
                    &self.data,
                    self.layout,
                    self.extent(),
                );
                ctx.submit_empty_and_wait();
            }
            TextureVariant::Host | TextureVariant::HostOptimized => {
                let done = Arc::new(AtomicBool::new(false));
                let flag = done.clone();
                texture.map_async(wgpu::MapMode::Write, move |r| {
                    r.unwrap();
                    flag.store(true, Ordering::Release);
                });
                wait_for_map(ctx, &done);
                self.host_write(texture);
                texture.unmap();
                if variant == TextureVariant::HostOptimized {
                    metal_optimize(ctx, texture);
                }
            }
        }
        start.elapsed().as_secs_f64() * 1e3
    }
}

fn noise(len: usize, seed: u64) -> Vec<u8> {
    let mut state = 0x9E37_79B9_7F4A_7C15u64 ^ seed;
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

fn smooth(format: wgpu::TextureFormat, size: u32, len: usize, block_bytes: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(len);
    match format {
        wgpu::TextureFormat::Rgba8Unorm => {
            for y in 0..size {
                for x in 0..size {
                    out.extend_from_slice(&[
                        (x * 255 / size) as u8,
                        (y * 255 / size) as u8,
                        128,
                        255,
                    ]);
                }
            }
        }
        wgpu::TextureFormat::Rgba16Float => {
            for y in 0..size {
                for x in 0..size {
                    for v in [x as f32 / size as f32, y as f32 / size as f32, 0.5, 1.0] {
                        out.extend_from_slice(&f16(v).to_le_bytes());
                    }
                }
            }
        }
        _ => {
            // One repeated block. Any bit pattern is a valid BC1 or ETC2 block.
            let block = [0x84u8, 0x42, 0x21, 0x02, 0x10, 0x32, 0x54, 0x76];
            while out.len() < len {
                out.extend_from_slice(&block[..block_bytes.min(8)]);
            }
        }
    }
    out.truncate(len);
    out
}

/// Converts a value in `0.0..=1.0` to half precision.
fn f16(v: f32) -> u16 {
    if v <= 0.0 {
        return 0;
    }
    let bits = v.to_bits();
    let exp = ((bits >> 23) & 0xff) as i32 - 127 + 15;
    if exp <= 0 {
        return 0;
    }
    ((exp as u32) << 10 | ((bits >> 13) & 0x3ff)) as u16
}

#[cfg(target_vendor = "apple")]
fn metal_optimize(ctx: &Ctx, texture: &wgpu::Texture) {
    use objc2_metal::{
        MTLBlitCommandEncoder as _, MTLCommandBuffer as _, MTLCommandEncoder as _,
        MTLCommandQueue as _,
    };
    unsafe {
        let texture = texture
            .as_hal::<wgpu::hal::api::Metal>()
            .expect("not a Metal texture");
        let queue = ctx
            .queue
            .as_hal::<wgpu::hal::api::Metal>()
            .expect("not a Metal queue");
        let command_buffer = queue.as_raw().commandBuffer().unwrap();
        let blit = command_buffer.blitCommandEncoder().unwrap();
        blit.optimizeContentsForGPUAccess(texture.raw_handle());
        blit.endEncoding();
        command_buffer.commit();
        command_buffer.waitUntilCompleted();
    }
}

#[cfg(not(target_vendor = "apple"))]
fn metal_optimize(_: &Ctx, _: &wgpu::Texture) {
    unreachable!("host+optimize only runs on Metal")
}

#[cfg(target_vendor = "apple")]
fn allows_gpu_optimized_contents(texture: &wgpu::Texture) -> Option<bool> {
    use objc2_metal::MTLTexture as _;
    let texture = unsafe { texture.as_hal::<wgpu::hal::api::Metal>() }?;
    Some(texture.raw_handle().allowGPUOptimizedContents())
}

#[cfg(not(target_vendor = "apple"))]
fn allows_gpu_optimized_contents(_: &wgpu::Texture) -> Option<bool> {
    None
}

struct Sampler {
    pipeline: wgpu::ComputePipeline,
    sink: wgpu::Buffer,
}

impl Sampler {
    fn new(ctx: &Ctx) -> Self {
        let module = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("sample_texture.wgsl"));
        let pipeline = ctx
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("sample_texture"),
                layout: None,
                module: &module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });
        Sampler {
            pipeline,
            sink: ctx.sink(),
        }
    }

    /// Samples every texel `reps` times and returns the time in milliseconds.
    fn run(&self, ctx: &Ctx, texture: &wgpu::Texture, reps: u32) -> f64 {
        let size = texture.size();
        let params = ctx.uniform([size.width, size.height, reps, 0]);
        let view = texture.create_view(&Default::default());
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.sink.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params.as_entire_binding(),
                },
            ],
        });
        ctx.time_compute(|pass| {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(size.width.div_ceil(8), size.height.div_ceil(8), 1);
        })
    }
}

fn bench_textures(ctx: &mut Ctx) {
    let sampler = Sampler::new(ctx);
    let sizes: &[u32] = if ctx.quick {
        &[1024]
    } else {
        &[256, 1024, 4096]
    };
    let iterations = if ctx.quick { 3 } else { 5 };
    let host_feature = ctx
        .device
        .features()
        .contains(wgpu::Features::HOST_TEXTURE_COPY);
    let is_metal = ctx.adapter.get_info().backend == wgpu::Backend::Metal;

    for &(format, feature) in TEXTURE_FORMATS {
        if !ctx.device.features().contains(feature) {
            continue;
        }
        let host_visible = host_feature
            && ctx
                .adapter
                .get_texture_format_features(format)
                .allowed_usages
                .contains(wgpu::TextureUsages::HOST_VISIBLE);
        let mut variants = vec![TextureVariant::Staging];
        if host_visible {
            variants.push(TextureVariant::Host);
            if is_metal {
                variants.push(TextureVariant::HostOptimized);
            }
        }
        let contents: &[Content] = if ctx.quick {
            &[Content::Smooth]
        } else {
            &[Content::Smooth, Content::Noise]
        };
        for &size in sizes {
            for &content in contents {
                let case = TextureCase::new(format, size, content);
                let name = case.name();
                println!("  textures: {name}");
                // Enough reads per dispatch that small textures aren't dominated by
                // dispatch overhead.
                let reps = ((64 << 20) / (size * size)).clamp(1, 4096);
                let texel_reads = (size as f64) * (size as f64) * reps as f64;
                for &variant in &variants {
                    let mut fresh = Vec::new();
                    let mut first_use = Vec::new();
                    let mut texture = None;
                    for _ in 0..iterations {
                        let (t, ms) = case.upload_fresh(ctx, variant);
                        fresh.push(ms);
                        first_use.push(sampler.run(ctx, &t, reps));
                        texture = Some(t);
                    }
                    let texture = texture.unwrap();
                    let mut again = Vec::new();
                    for _ in 0..iterations {
                        again.push(case.upload_again(ctx, &texture, variant));
                        sampler.run(ctx, &texture, 1);
                    }
                    let steady: Vec<f64> = (0..iterations * 2)
                        .map(|_| sampler.run(ctx, &texture, reps))
                        .collect();
                    if ctx.took_error() {
                        eprintln!("  {name} {}: skipped after an error", variant.name());
                        continue;
                    }
                    let v = variant.name();
                    ctx.record("texture", &name, v, "upload_fresh", median(fresh), "ms");
                    ctx.record("texture", &name, v, "upload_again", median(again), "ms");
                    ctx.record("texture", &name, v, "first_sample", median(first_use), "ms");
                    let steady = median(steady);
                    ctx.record("texture", &name, v, "sample", steady, "ms");
                    ctx.record(
                        "texture",
                        &name,
                        v,
                        "sample_rate",
                        texel_reads / (steady * 1e6),
                        "Gtexel/s",
                    );
                    if variant != TextureVariant::Staging
                        && let Some(allowed) = allows_gpu_optimized_contents(&texture)
                    {
                        ctx.record(
                            "texture",
                            &name,
                            v,
                            "allowGPUOptimizedContents",
                            allowed as u32 as f64,
                            "bool",
                        );
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Buffers
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum BufferVariant {
    /// `STORAGE | COPY_DST`, in GPU-only memory.
    Gpu,
    /// `STORAGE | MAP_WRITE`, in CPU-visible memory.
    Mappable,
}

impl BufferVariant {
    fn name(self) -> &'static str {
        match self {
            BufferVariant::Gpu => "gpu-only",
            BufferVariant::Mappable => "mappable",
        }
    }

    fn usage(self) -> wgpu::BufferUsages {
        match self {
            BufferVariant::Gpu => wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            BufferVariant::Mappable => {
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::MAP_WRITE
                    | wgpu::BufferUsages::COPY_DST
            }
        }
    }
}

fn bench_buffers(ctx: &mut Ctx) {
    let module = ctx
        .device
        .create_shader_module(wgpu::include_wgsl!("read_buffer.wgsl"));
    let pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("read_buffer"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
    let sink = ctx.sink();
    let max = ctx
        .device
        .limits()
        .max_storage_buffer_binding_size
        .min(ctx.device.limits().max_buffer_size);
    let sizes: Vec<u64> = if ctx.quick {
        vec![16 * MB]
    } else {
        vec![MB, 16 * MB, 64 * MB]
    }
    .into_iter()
    .filter(|&s| s <= max)
    .collect();
    let iterations = if ctx.quick { 3 } else { 5 };
    let mut variants = vec![BufferVariant::Gpu];
    if ctx
        .device
        .features()
        .contains(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS)
    {
        variants.push(BufferVariant::Mappable);
    }

    for size in sizes {
        let name = format!("{} MiB", size / MB);
        println!("  buffers: {name}");
        let data = noise(size as usize, size);
        let reps = ((256 * MB) / size).max(1) as u32;
        for &variant in &variants {
            let mut fresh = Vec::new();
            let mut buffer = None;
            for _ in 0..iterations {
                let start = Instant::now();
                let b = ctx.device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size,
                    usage: variant.usage(),
                    mapped_at_creation: true,
                });
                b.get_mapped_range_mut(..).unwrap().copy_from_slice(&data);
                b.unmap();
                ctx.submit_empty_and_wait();
                fresh.push(start.elapsed().as_secs_f64() * 1e3);
                buffer = Some(b);
            }
            let buffer = buffer.unwrap();
            let params = ctx.uniform([0, 0, 1, (size / 16) as u32]);
            let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: sink.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: params.as_entire_binding(),
                    },
                ],
            });
            let read = |ctx: &Ctx| {
                ctx.time_compute(|pass| {
                    pass.set_pipeline(&pipeline);
                    pass.set_bind_group(0, &bind_group, &[]);
                    // One dispatch per pass over the buffer, so the reads can't be
                    // hoisted out of a loop.
                    for _ in 0..reps {
                        pass.dispatch_workgroups(1024, 1, 1);
                    }
                })
            };
            let mut again = Vec::new();
            for _ in 0..iterations {
                read(ctx);
                let start = Instant::now();
                match variant {
                    BufferVariant::Gpu => {
                        ctx.queue.write_buffer(&buffer, 0, &data);
                        ctx.submit_empty_and_wait();
                    }
                    BufferVariant::Mappable => {
                        let done = Arc::new(AtomicBool::new(false));
                        let flag = done.clone();
                        buffer.map_async(wgpu::MapMode::Write, .., move |r| {
                            r.unwrap();
                            flag.store(true, Ordering::Release);
                        });
                        wait_for_map(ctx, &done);
                        buffer
                            .get_mapped_range_mut(..)
                            .unwrap()
                            .copy_from_slice(&data);
                        buffer.unmap();
                    }
                }
                again.push(start.elapsed().as_secs_f64() * 1e3);
            }
            let reads: Vec<f64> = (0..iterations * 2).map(|_| read(ctx)).collect();
            if ctx.took_error() {
                eprintln!("  {name} {}: skipped after an error", variant.name());
                continue;
            }
            let v = variant.name();
            ctx.record("buffer", &name, v, "upload_fresh", median(fresh), "ms");
            ctx.record("buffer", &name, v, "upload_again", median(again), "ms");
            let read = median(reads);
            ctx.record("buffer", &name, v, "read", read, "ms");
            ctx.record(
                "buffer",
                &name,
                v,
                "read_rate",
                (size * reps as u64) as f64 / (read * 1e6),
                "GB/s",
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Vulkan end-of-command-buffer host barrier
// ---------------------------------------------------------------------------

/// Runs submit-heavy workloads with and without a global device-write to host
/// barrier at the end of every command buffer. The hal reads
/// `WGPU_BENCH_VK_HOST_BARRIER` when it opens a device.
fn bench_barrier(adapter: &wgpu::Adapter, rows: &mut Vec<Row>, quick: bool) {
    let info = adapter.get_info();
    if info.backend != wgpu::Backend::Vulkan {
        return;
    }
    let rounds = if quick { 1 } else { 2 };
    for round in 0..rounds {
        for (on, variant) in [(false, "barrier off"), (true, "barrier on")] {
            // SAFETY: no other threads are running.
            unsafe { std::env::set_var("WGPU_BENCH_VK_HOST_BARRIER", if on { "1" } else { "0" }) };
            let Some(mut ctx) = Ctx::new(adapter, quick) else {
                return;
            };
            println!("  barrier: round {} {variant}", round + 1);
            let case = format!("round {}", round + 1);
            let (throughput, latency) = tiny_submits(&ctx, quick);
            ctx.record(
                "barrier",
                &case,
                variant,
                "submit_throughput",
                throughput,
                "us/submit",
            );
            ctx.record(
                "barrier",
                &case,
                variant,
                "submit_latency",
                latency,
                "us/submit",
            );
            let frame = frames(&ctx, quick);
            ctx.record("barrier", &case, variant, "frame", frame, "ms/frame");
            rows.append(&mut ctx.rows);
        }
    }
    unsafe { std::env::remove_var("WGPU_BENCH_VK_HOST_BARRIER") };
}

fn tiny_submits(ctx: &Ctx, quick: bool) -> (f64, f64) {
    let module = ctx
        .device
        .create_shader_module(wgpu::include_wgsl!("read_buffer.wgsl"));
    let pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("tiny"),
            layout: None,
            module: &module,
            entry_point: Some("tiny"),
            compilation_options: Default::default(),
            cache: None,
        });
    let sink = ctx.sink();
    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 1,
            resource: sink.as_entire_binding(),
        }],
    });
    let submit = || {
        let mut encoder = ctx.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        ctx.queue.submit(Some(encoder.finish()))
    };
    for _ in 0..50 {
        submit();
    }
    ctx.wait();

    let count = if quick { 300 } else { 2000 };
    let start = Instant::now();
    for i in 0..count {
        let index = submit();
        // Bound the work in flight, like a renderer would.
        if i % 8 == 7 {
            ctx.device
                .poll(wgpu::PollType::Wait {
                    submission_index: index,
                    timeout: None,
                })
                .unwrap();
        }
    }
    ctx.wait();
    let throughput = start.elapsed().as_secs_f64() * 1e6 / count as f64;

    let count = if quick { 100 } else { 400 };
    let start = Instant::now();
    for _ in 0..count {
        submit();
        ctx.wait();
    }
    let latency = start.elapsed().as_secs_f64() * 1e6 / count as f64;
    (throughput, latency)
}

fn frames(ctx: &Ctx, quick: bool) -> f64 {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: 1920,
            height: 1080,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
        texture_binding_view_dimension: None,
        mapped_at_creation: false,
    });
    let target_view = target.create_view(&Default::default());
    let case = TextureCase::new(format, 1024, Content::Noise);
    let (source, _) = case.upload_fresh(ctx, TextureVariant::Staging);
    let source_view = source.create_view(&Default::default());
    let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let module = ctx
        .device
        .create_shader_module(wgpu::include_wgsl!("frame.wgsl"));
    let pipeline = ctx
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("frame"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(format.into())],
            }),
            multiview_mask: None,
            cache: None,
        });
    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&source_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let frame = || {
        let mut encoder = ctx.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        ctx.queue.submit(Some(encoder.finish()))
    };
    for _ in 0..10 {
        frame();
    }
    ctx.wait();
    let count = if quick { 60 } else { 300 };
    let mut in_flight = std::collections::VecDeque::new();
    let start = Instant::now();
    for _ in 0..count {
        in_flight.push_back(frame());
        if in_flight.len() > 2 {
            ctx.device
                .poll(wgpu::PollType::Wait {
                    submission_index: in_flight.pop_front().flatten(),
                    timeout: None,
                })
                .unwrap();
        }
    }
    ctx.wait();
    start.elapsed().as_secs_f64() * 1e3 / count as f64
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

fn print_rows(rows: &[Row], out: &mut String) {
    let _ = writeln!(out, "| bench | case | variant | metric | value |");
    let _ = writeln!(out, "|---|---|---|---|---|");
    for row in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {:.3} {} |",
            row.bench, row.case, row.variant, row.metric, row.value, row.unit
        );
    }
    let _ = writeln!(out);
}

fn write_csv(path: &str, rows: &[Row]) {
    let mut csv = String::from("adapter,backend,bench,case,variant,metric,value,unit\n");
    for r in rows {
        let _ = writeln!(
            csv,
            "\"{}\",{},{},\"{}\",{},{},{},{}",
            r.adapter, r.backend, r.bench, r.case, r.variant, r.metric, r.value, r.unit
        );
    }
    std::fs::write(path, csv).expect("failed to write the CSV file");
}

fn main() {
    env_logger::init();
    let options = Options::parse();
    if cfg!(debug_assertions) {
        eprintln!(
            "warning: this is a debug build; run with `cargo run --release` for real numbers"
        );
    }

    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::VULKAN | wgpu::Backends::METAL | wgpu::Backends::DX12;
    descriptor.flags = wgpu::InstanceFlags::empty();
    let instance = wgpu::Instance::new(descriptor.with_env());
    let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));

    let mut report = String::from("# host-copy-bench\n\n");
    let mut all_rows = Vec::new();
    for adapter in adapters {
        let info = adapter.get_info();
        if info.backend == wgpu::Backend::Gl || info.backend == wgpu::Backend::BrowserWebGpu {
            continue;
        }
        if info.device_type == wgpu::DeviceType::Cpu && !options.include_cpu {
            continue;
        }
        if let Some(filter) = &options.adapter_filter
            && !info.name.to_lowercase().contains(filter)
        {
            continue;
        }
        println!("{} ({:?})", info.name, info.backend);
        let mut section = String::new();
        describe(&adapter, &mut section);

        let mut rows = Vec::new();
        if let Some(mut ctx) = Ctx::new(&adapter, options.quick) {
            if options.runs("textures") {
                bench_textures(&mut ctx);
            }
            if options.runs("buffers") {
                bench_buffers(&mut ctx);
            }
            rows.append(&mut ctx.rows);
        }
        if options.runs("barrier") {
            bench_barrier(&adapter, &mut rows, options.quick);
        }
        print_rows(&rows, &mut section);
        report.push_str(&section);
        all_rows.extend(rows);
    }

    println!();
    print!("{report}");
    if let Some(path) = &options.csv {
        write_csv(path, &all_rows);
        println!("wrote {path}");
    }
}
