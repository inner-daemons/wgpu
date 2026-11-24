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

mod scenarios;

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
                         [--include-cpu] [--skip textures|buffers|scenarios|barrier]..."
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
    variant: String,
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
            | wgpu::Features::TEXTURE_COMPRESSION_ETC2
            | wgpu::Features::TEXTURE_COMPRESSION_ASTC;
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
            variant: variant.to_owned(),
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
        plausible_gpu_time(gpu, wall)
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

/// Uses the GPU time unless it can't be right. Some drivers, such as Apple's
/// Metal driver, sometimes report pass timestamps far too close together. The
/// GPU time can't exceed the wall time around the submission, and for long
/// enough work it is most of it.
fn plausible_gpu_time(gpu: f64, wall: f64) -> f64 {
    if gpu <= wall && (gpu >= wall * 0.5 || wall < 2.0) && gpu > 0.0 {
        gpu
    } else {
        wall
    }
}

/// The median of the ratios between measurements taken in the same iteration.
fn paired_ratio(values: &[f64], base: &[f64]) -> f64 {
    median(values.iter().zip(base).map(|(v, b)| v / b).collect())
}

#[derive(Default)]
struct Samples {
    fresh: Vec<f64>,
    first_use: Vec<f64>,
    again: Vec<f64>,
    again_cpu: Vec<f64>,
    again_wait: Vec<f64>,
    steady: Vec<f64>,
    readback: Vec<f64>,
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
// Vulkan host image copy queries
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
const VK_HOST_COPY_FORMATS: &[ash::vk::Format] = {
    use ash::vk::Format as F;
    &[
        F::R8_UNORM,
        F::R8_UINT,
        F::R8G8_UNORM,
        F::R8G8B8A8_UNORM,
        F::R8G8B8A8_SRGB,
        F::B8G8R8A8_UNORM,
        F::B8G8R8A8_SRGB,
        F::A2B10G10R10_UNORM_PACK32,
        F::B10G11R11_UFLOAT_PACK32,
        F::E5B9G9R9_UFLOAT_PACK32,
        F::R16_UNORM,
        F::R16_SFLOAT,
        F::R16G16_SFLOAT,
        F::R16G16B16A16_UNORM,
        F::R16G16B16A16_SFLOAT,
        F::R32_UINT,
        F::R32_SFLOAT,
        F::R32G32B32A32_SFLOAT,
        F::D16_UNORM,
        F::D32_SFLOAT,
        F::D24_UNORM_S8_UINT,
        F::D32_SFLOAT_S8_UINT,
        F::S8_UINT,
        F::BC1_RGBA_UNORM_BLOCK,
        F::BC1_RGBA_SRGB_BLOCK,
        F::BC3_UNORM_BLOCK,
        F::BC4_UNORM_BLOCK,
        F::BC5_UNORM_BLOCK,
        F::BC6H_UFLOAT_BLOCK,
        F::BC7_UNORM_BLOCK,
        F::BC7_SRGB_BLOCK,
        F::ETC2_R8G8B8_UNORM_BLOCK,
        F::ETC2_R8G8B8A8_UNORM_BLOCK,
        F::EAC_R11_UNORM_BLOCK,
        F::ASTC_4X4_UNORM_BLOCK,
        F::ASTC_4X4_SRGB_BLOCK,
        F::ASTC_8X8_UNORM_BLOCK,
    ]
};

/// For each format, asks the driver whether host transfer usage keeps device
/// access optimal and the size limits unchanged, for each combination of
/// creation flags wgpu can use, with all usages the format supports and with
/// only sampling and copies.
#[cfg(not(target_arch = "wasm32"))]
fn describe_vulkan_host_copy(adapter: &wgpu::Adapter, out: &mut String, rows: &mut Vec<Row>) {
    use ash::vk;
    let Some(hal) = (unsafe { adapter.as_hal::<wgpu::hal::api::Vulkan>() }) else {
        return;
    };
    let info = adapter.get_info();
    let instance = hal.shared_instance().raw_instance();
    let physical_device = hal.raw_physical_device();
    let mut record = |case: String, variant: String, metric: &'static str, value: bool| {
        rows.push(Row {
            adapter: info.name.clone(),
            backend: format!("{:?}", info.backend),
            bench: "vk_host_copy",
            case,
            variant,
            metric,
            value: value as u32 as f64,
            unit: "bool",
        });
    };

    let supported = unsafe { instance.enumerate_device_extension_properties(physical_device) }
        .unwrap_or_default()
        .iter()
        .any(|e| e.extension_name_as_c_str() == Ok(ash::ext::host_image_copy::NAME));
    let _ = writeln!(out, "- VK_EXT_host_image_copy: {supported}");
    record("device".into(), "-".into(), "host_image_copy", supported);
    if !supported {
        return;
    }
    let mut host_copy_properties = vk::PhysicalDeviceHostImageCopyPropertiesEXT::default();
    let mut properties =
        vk::PhysicalDeviceProperties2::default().push_next(&mut host_copy_properties);
    unsafe { instance.get_physical_device_properties2(physical_device, &mut properties) };
    let identical = host_copy_properties.identical_memory_type_requirements != vk::FALSE;
    let _ = writeln!(out, "- identicalMemoryTypeRequirements: {identical}");
    // wgpu also treats integrated and CPU devices as having one kind of memory.
    let free_memory = identical
        || matches!(
            info.device_type,
            wgpu::DeviceType::IntegratedGpu | wgpu::DeviceType::Cpu
        );
    record(
        "device".into(),
        "-".into(),
        "identical_memory_type_requirements",
        identical,
    );

    let variants = [
        ("2d", vk::ImageType::TYPE_2D, vk::ImageCreateFlags::empty()),
        (
            "2d-cube",
            vk::ImageType::TYPE_2D,
            vk::ImageCreateFlags::CUBE_COMPATIBLE,
        ),
        (
            "2d-mutable",
            vk::ImageType::TYPE_2D,
            vk::ImageCreateFlags::MUTABLE_FORMAT,
        ),
        ("3d", vk::ImageType::TYPE_3D, vk::ImageCreateFlags::empty()),
        (
            "3d-2d-array",
            vk::ImageType::TYPE_3D,
            vk::ImageCreateFlags::TYPE_2D_ARRAY_COMPATIBLE,
        ),
    ];
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Host transfer per format and creation flags, with all usages / with sampling and copies only. \
         Y: optimal, N: not optimal, L: optimal but lower size limits, -: combination unsupported."
    );
    let _ = writeln!(out);
    let mut header = String::from("| format | host transfer |");
    let mut rule = String::from("|---|---|");
    for (name, _, _) in variants {
        header += &format!(" {name} |");
        rule += "---|";
    }
    let _ = writeln!(out, "{header} wgpu optimal |");
    let _ = writeln!(out, "{rule}---|");

    for &format in VK_HOST_COPY_FORMATS {
        let name = format!("{format:?}");
        let mut properties3 = vk::FormatProperties3::default();
        let mut properties2 = vk::FormatProperties2::default().push_next(&mut properties3);
        unsafe {
            instance.get_physical_device_format_properties2(
                physical_device,
                format,
                &mut properties2,
            )
        };
        let features = properties3.optimal_tiling_features;
        let host_transfer = features.contains(vk::FormatFeatureFlags2::HOST_IMAGE_TRANSFER_EXT);
        record(name.clone(), "-".into(), "host_transfer", host_transfer);
        if !host_transfer {
            let _ = writeln!(out, "| {name} | no |{} |", " |".repeat(variants.len()));
            continue;
        }
        let mut all = vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST;
        for (feature, usage) in [
            (
                vk::FormatFeatureFlags2::SAMPLED_IMAGE,
                vk::ImageUsageFlags::SAMPLED,
            ),
            (
                vk::FormatFeatureFlags2::STORAGE_IMAGE,
                vk::ImageUsageFlags::STORAGE,
            ),
            (
                vk::FormatFeatureFlags2::COLOR_ATTACHMENT,
                vk::ImageUsageFlags::COLOR_ATTACHMENT,
            ),
            (
                vk::FormatFeatureFlags2::DEPTH_STENCIL_ATTACHMENT,
                vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            ),
        ] {
            if features.contains(feature) {
                all |= usage;
            }
        }
        let sampled = vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | if features.contains(vk::FormatFeatureFlags2::SAMPLED_IMAGE) {
                vk::ImageUsageFlags::SAMPLED
            } else {
                vk::ImageUsageFlags::empty()
            };

        let query =
            |ty: vk::ImageType,
             flags: vk::ImageCreateFlags,
             usage: vk::ImageUsageFlags,
             performance: Option<&mut vk::HostImageCopyDevicePerformanceQueryEXT>| {
                let info = vk::PhysicalDeviceImageFormatInfo2::default()
                    .format(format)
                    .ty(ty)
                    .tiling(vk::ImageTiling::OPTIMAL)
                    .usage(usage)
                    .flags(flags);
                let mut properties = vk::ImageFormatProperties2::default();
                if let Some(performance) = performance {
                    properties = properties.push_next(performance);
                }
                unsafe {
                    instance.get_physical_device_image_format_properties2(
                        physical_device,
                        &info,
                        &mut properties,
                    )
                }
                .ok()
                .map(|()| properties.image_format_properties)
            };

        let mut row = format!("| {name} | yes |");
        let mut wgpu_optimal = free_memory;
        let mut any = false;
        for (variant, ty, flags) in variants {
            let mut cell = Vec::new();
            for (usage_name, usage) in [("all", all), ("sampled", sampled)] {
                let key = format!("{variant}/{usage_name}");
                let Some(base) = query(ty, flags, usage, None) else {
                    record(name.clone(), key, "supported", false);
                    cell.push("-");
                    continue;
                };
                record(name.clone(), key.clone(), "supported", true);
                let mut performance = vk::HostImageCopyDevicePerformanceQueryEXT::default();
                let host = query(
                    ty,
                    flags,
                    usage | vk::ImageUsageFlags::HOST_TRANSFER_EXT,
                    Some(&mut performance),
                );
                let optimal = host.is_some() && performance.optimal_device_access != vk::FALSE;
                let identical_layout =
                    host.is_some() && performance.identical_memory_layout != vk::FALSE;
                let limits_kept = host.is_some_and(|host| {
                    host.max_extent.width >= base.max_extent.width
                        && host.max_extent.height >= base.max_extent.height
                        && host.max_extent.depth >= base.max_extent.depth
                        && host.max_mip_levels >= base.max_mip_levels
                        && host.max_array_layers >= base.max_array_layers
                        && host.max_resource_size >= base.max_resource_size
                });
                record(name.clone(), key.clone(), "optimal", optimal);
                record(
                    name.clone(),
                    key.clone(),
                    "identical_layout",
                    identical_layout,
                );
                record(name.clone(), key, "limits_kept", limits_kept);
                if usage_name == "all" {
                    any = true;
                    wgpu_optimal &= optimal && limits_kept;
                }
                cell.push(match (optimal, limits_kept) {
                    (true, true) => "Y",
                    (true, false) => "L",
                    _ => "N",
                });
            }
            row += &format!(" {} |", cell.join("/"));
        }
        let wgpu_optimal = wgpu_optimal && any;
        record(name.clone(), "-".into(), "wgpu_optimal", wgpu_optimal);
        let _ = writeln!(out, "{row} {} |", if wgpu_optimal { "yes" } else { "no" });
    }
    let _ = writeln!(out);
}

#[cfg(target_arch = "wasm32")]
fn describe_vulkan_host_copy(_: &wgpu::Adapter, _: &mut String, _: &mut Vec<Row>) {}

// ---------------------------------------------------------------------------
// Textures
// ---------------------------------------------------------------------------

const ASTC_4X4: wgpu::TextureFormat = wgpu::TextureFormat::Astc {
    block: wgpu::AstcBlock::B4x4,
    channel: wgpu::AstcChannel::Unorm,
};

const TEXTURE_FORMATS: &[(wgpu::TextureFormat, wgpu::Features)] = &[
    (wgpu::TextureFormat::Rgba8Unorm, wgpu::Features::empty()),
    (wgpu::TextureFormat::R8Unorm, wgpu::Features::empty()),
    (wgpu::TextureFormat::Rgba16Float, wgpu::Features::empty()),
    (ASTC_4X4, wgpu::Features::TEXTURE_COMPRESSION_ASTC),
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
    /// Gradients with a little noise, like a photo. Uncompressed formats only.
    Detail,
    /// Random bytes. Lossless compression can't help.
    Noise,
}

impl Content {
    fn name(self) -> &'static str {
        match self {
            Content::Smooth => "smooth",
            Content::Detail => "detail",
            Content::Noise => "noise",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum TextureVariant {
    /// `COPY_DST` texture written with `Queue::write_texture`, which goes
    /// through a staging buffer and a copy on the GPU.
    Staging,
    /// Like `Staging`, on a second device, to show how much two devices doing
    /// the same work differ.
    StagingControl,
    /// Like `Staging`, on a device that lets wgpu add host copy usage and
    /// write from the host where the backend says that's free.
    Implicit,
    /// `HOST_VISIBLE` texture written with `copy_from_memory`.
    Host,
    /// Metal only: like `Host`, followed by `optimizeContentsForGPUAccess`.
    HostOptimized,
}

impl TextureVariant {
    fn name(self) -> &'static str {
        match self {
            TextureVariant::Staging => "staging",
            TextureVariant::StagingControl => "staging-control",
            TextureVariant::Implicit => "implicit",
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
        let data = if format == ASTC_4X4 {
            astc_blocks(len / 16, content == Content::Noise)
        } else if format.is_compressed() {
            match content {
                Content::Noise => noise(len, size as u64),
                _ => repeated_block(len, block_bytes as usize),
            }
        } else {
            match content {
                Content::Noise => noise(len, size as u64),
                Content::Smooth => gradient(format, size, 0),
                Content::Detail => gradient(format, size, 6),
            }
        };
        assert_eq!(data.len(), len);
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
            "{} {}x{} {}",
            format_name(self.format),
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
            TextureVariant::Staging | TextureVariant::StagingControl | TextureVariant::Implicit => {
                wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING
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
            TextureVariant::Staging | TextureVariant::StagingControl | TextureVariant::Implicit => {
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

    /// Overwrites a texture the GPU has used, timing each phase until the GPU
    /// could use it again.
    fn upload_again(&self, ctx: &Ctx, texture: &wgpu::Texture, variant: TextureVariant) -> Phases {
        let start = Instant::now();
        let mut phases = Phases::default();
        match variant {
            TextureVariant::Staging | TextureVariant::StagingControl | TextureVariant::Implicit => {
                ctx.queue.write_texture(
                    texture.as_image_copy(),
                    &self.data,
                    self.layout,
                    self.extent(),
                );
                phases.cpu = ms_since(start);
                let wait = Instant::now();
                ctx.submit_empty_and_wait();
                phases.wait = ms_since(wait);
            }
            TextureVariant::Host | TextureVariant::HostOptimized => {
                map_texture(ctx, texture, wgpu::MapMode::Write);
                phases.wait = ms_since(start);
                let cpu = Instant::now();
                self.host_write(texture);
                texture.unmap();
                phases.cpu = ms_since(cpu);
                if variant == TextureVariant::HostOptimized {
                    let optimize = Instant::now();
                    metal_optimize(ctx, texture);
                    phases.wait += ms_since(optimize);
                }
            }
        }
        phases.total = ms_since(start);
        phases
    }

    /// Reads the whole texture back into memory, through a buffer or from the
    /// host.
    fn read_back(&self, ctx: &Ctx, texture: &wgpu::Texture, variant: TextureVariant) -> f64 {
        let start = Instant::now();
        let mut out = vec![0u8; self.data.len()];
        match variant {
            TextureVariant::Staging | TextureVariant::StagingControl | TextureVariant::Implicit => {
                let row = self.layout.bytes_per_row.unwrap();
                let rows = self.layout.rows_per_image.unwrap();
                let padded = row.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
                let buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: (padded * rows) as u64,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                let mut encoder = ctx.device.create_command_encoder(&Default::default());
                encoder.copy_texture_to_buffer(
                    texture.as_image_copy(),
                    wgpu::TexelCopyBufferInfo {
                        buffer: &buffer,
                        layout: wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(padded),
                            rows_per_image: Some(rows),
                        },
                    },
                    self.extent(),
                );
                ctx.queue.submit(Some(encoder.finish()));
                map_buffer(ctx, &buffer, wgpu::MapMode::Read);
                {
                    let view = buffer.get_mapped_range(..).unwrap();
                    for (dst, src) in out
                        .chunks_exact_mut(row as usize)
                        .zip(view.chunks(padded as usize))
                    {
                        dst.copy_from_slice(&src[..row as usize]);
                    }
                }
                buffer.unmap();
            }
            TextureVariant::Host | TextureVariant::HostOptimized => {
                map_texture(ctx, texture, wgpu::MapMode::Read);
                texture.get_mapped_texture().unwrap().copy_to_memory(
                    wgpu::TexelCopyTextureInfoBase {
                        texture: (),
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &mut out,
                    self.layout,
                    self.extent(),
                );
                texture.unmap();
            }
        }
        std::hint::black_box(&out);
        ms_since(start)
    }
}

#[derive(Default)]
struct Phases {
    total: f64,
    /// Waiting for the GPU: submitting and waiting for the copy, or for the map.
    wait: f64,
    /// Copying on the CPU.
    cpu: f64,
}

fn ms_since(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1e3
}

fn map_texture(ctx: &Ctx, texture: &wgpu::Texture, mode: wgpu::MapMode) {
    let done = Arc::new(AtomicBool::new(false));
    let flag = done.clone();
    texture.map_async(mode, move |r| {
        r.unwrap();
        flag.store(true, Ordering::Release);
    });
    wait_for_map(ctx, &done);
}

fn map_buffer(ctx: &Ctx, buffer: &wgpu::Buffer, mode: wgpu::MapMode) {
    let done = Arc::new(AtomicBool::new(false));
    let flag = done.clone();
    buffer.map_async(mode, .., move |r| {
        r.unwrap();
        flag.store(true, Ordering::Release);
    });
    wait_for_map(ctx, &done);
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

/// Horizontal and vertical gradients, with noise of up to `amplitude` levels
/// out of 255 added to each color channel.
fn gradient(format: wgpu::TextureFormat, size: u32, amplitude: u64) -> Vec<u8> {
    let mut state = 0x2545_F491_4F6C_DD1Du64 ^ size as u64;
    let mut jitter = move || {
        if amplitude == 0 {
            return 0.0;
        }
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        ((state % (2 * amplitude + 1)) as f32 - amplitude as f32) / 255.0
    };
    let unorm8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let mut out = Vec::new();
    for y in 0..size {
        for x in 0..size {
            let u = x as f32 / size as f32;
            let v = y as f32 / size as f32;
            match format {
                wgpu::TextureFormat::R8Unorm => out.push(unorm8((u + v) / 2.0 + jitter())),
                wgpu::TextureFormat::Rgba8Unorm => {
                    for c in [u, v, 0.5] {
                        out.push(unorm8(c + jitter()));
                    }
                    out.push(255);
                }
                wgpu::TextureFormat::Rgba16Float => {
                    for c in [u + jitter(), v + jitter(), 0.5 + jitter(), 1.0] {
                        out.extend_from_slice(&f16(c.clamp(0.0, 1.0)).to_le_bytes());
                    }
                }
                _ => unreachable!("no gradient for {format:?}"),
            }
        }
    }
    out
}

/// One block repeated. Any bit pattern is a valid BC1 or ETC2 block.
fn repeated_block(len: usize, block_bytes: usize) -> Vec<u8> {
    let block = [0x84u8, 0x42, 0x21, 0x02, 0x10, 0x32, 0x54, 0x76];
    block[..block_bytes.min(8)]
        .iter()
        .copied()
        .cycle()
        .take(len)
        .collect()
}

/// The low 17 bits of an ASTC block: block mode 0x042 (a 4x4 grid of 2-bit
/// weights), one partition and direct RGBA endpoints (CEM 12). The rest is
/// 64 bits of 8-bit endpoints and 32 bits of weights, so any value for those
/// bits is a valid block.
const ASTC_HEADER: u128 = 0x042 | (12 << 13);
const ASTC_HEADER_MASK: u128 = (1 << 17) - 1;

fn format_name(format: wgpu::TextureFormat) -> String {
    if format == ASTC_4X4 {
        "Astc4x4Unorm".to_string()
    } else {
        format!("{format:?}")
    }
}

fn astc_block(payload: u128) -> u128 {
    (payload & !ASTC_HEADER_MASK) | ASTC_HEADER
}

/// ASTC 4x4 blocks with random endpoints and weights, or one block repeated.
fn astc_blocks(count: usize, random: bool) -> Vec<u8> {
    let fixed = astc_block(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
    let mut out = Vec::with_capacity(count * 16);
    for chunk in noise(count * 16, 0xA57C).chunks_exact(16) {
        let block = if random {
            astc_block(u128::from_le_bytes(chunk.try_into().unwrap()))
        } else {
            fixed
        };
        out.extend_from_slice(&block.to_le_bytes());
    }
    out
}

/// Decodes a block made by `astc_block` to RGBA8, texels in row order.
fn decode_astc_block(block: u128) -> [[u8; 4]; 16] {
    let v: [u32; 8] = std::array::from_fn(|i| ((block >> (17 + 8 * i)) & 0xff) as u32);
    let mut e0 = [v[0], v[2], v[4], v[6]];
    let mut e1 = [v[1], v[3], v[5], v[7]];
    if v[1] + v[3] + v[5] < v[0] + v[2] + v[4] {
        let contract = |c: [u32; 4]| [(c[0] + c[2]) >> 1, (c[1] + c[2]) >> 1, c[2], c[3]];
        (e0, e1) = (contract(e1), contract(e0));
    }
    // Weights are stored bit-reversed from the top of the block.
    let weights = block.reverse_bits();
    std::array::from_fn(|i| {
        let w = [0, 21, 43, 64][((weights >> (2 * i)) & 3) as usize];
        std::array::from_fn(|c| {
            let value = (e0[c] * 257 * (64 - w) + e1[c] * 257 * w + 32) / 64;
            (value >> 8) as u8
        })
    })
}

/// Checks that the GPU decodes the generated ASTC blocks as `decode_astc_block`
/// does, so the timings aren't of invalid blocks.
fn check_astc(ctx: &Ctx, case: &TextureCase, texture: &wgpu::Texture) -> Result<(), String> {
    let module = ctx
        .device
        .create_shader_module(wgpu::include_wgsl!("astc_check.wgsl"));
    let pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("astc_check"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
    let storage = |size| {
        ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    };
    let first_block = storage(256);
    let error_texels = storage(4);
    let readback = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 260,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let view = texture.create_view(&Default::default());
    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: first_block.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: error_texels.as_entire_binding(),
            },
        ],
    });
    let mut encoder = ctx.device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(case.size.div_ceil(8), case.size.div_ceil(8), 1);
    }
    encoder.copy_buffer_to_buffer(&first_block, 0, &readback, 0, 256);
    encoder.copy_buffer_to_buffer(&error_texels, 0, &readback, 256, 4);
    ctx.queue.submit(Some(encoder.finish()));
    map_buffer(ctx, &readback, wgpu::MapMode::Read);
    let (texels, errors) = {
        let view = readback.get_mapped_range(..).unwrap();
        let texels: Vec<f32> = bytemuck::cast_slice(&view[..256]).to_vec();
        let errors = u32::from_le_bytes(view[256..260].try_into().unwrap());
        (texels, errors)
    };
    readback.unmap();
    if errors > 0 {
        return Err(format!("{errors} texels decoded to the ASTC error color"));
    }
    let expected = decode_astc_block(u128::from_le_bytes(case.data[..16].try_into().unwrap()));
    for (i, texel) in expected.iter().enumerate() {
        for c in 0..4 {
            let got = texels[i * 4 + c] * 255.0;
            if (got - texel[c] as f32).abs() > 2.0 {
                return Err(format!(
                    "texel {i} channel {c} decoded to {got:.1}, expected {}",
                    texel[c]
                ));
            }
        }
    }
    Ok(())
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

/// Texel reads per invocation, so every dispatch reads about 256M texels and
/// runs long enough to be measured at full clocks.
fn sample_reps(size: u32) -> u32 {
    ((256 << 20) / (size * size)).clamp(1, 16384)
}

/// Keeps the GPU busy for a while so clocks ramp up before measuring.
fn warm_up(ctx: &Ctx, sampler: &Sampler) {
    let texture = TextureCase::new(wgpu::TextureFormat::Rgba8Unorm, 1024, Content::Noise)
        .upload_fresh(ctx, TextureVariant::Staging)
        .0;
    let start = Instant::now();
    while start.elapsed().as_millis() < 500 {
        sampler.run(ctx, &texture, sample_reps(1024));
    }
}

fn bench_textures(ctx: &mut Ctx, control: Option<&Ctx>, implicit: Option<&Ctx>) {
    let sampler = Sampler::new(ctx);
    warm_up(ctx, &sampler);
    let control = control.map(|c| (c, Sampler::new(c)));
    let implicit = implicit.map(|c| (c, Sampler::new(c)));
    let sizes: &[u32] = if ctx.quick {
        &[1024]
    } else {
        &[256, 512, 1024, 2048, 4096]
    };
    let iterations = if ctx.quick { 3 } else { 5 };
    let host_feature = ctx
        .device
        .features()
        .contains(wgpu::Features::HOST_TEXTURE_COPY);
    let is_metal = ctx.adapter.get_info().backend == wgpu::Backend::Metal;
    let mut records = Vec::new();
    let main = &*ctx;

    for &(format, feature) in TEXTURE_FORMATS {
        if !main.device.features().contains(feature) {
            continue;
        }
        let host_visible = host_feature
            && main
                .adapter
                .get_texture_format_features(format)
                .allowed_usages
                .contains(wgpu::TextureUsages::HOST_VISIBLE);
        let mut variants: Vec<(TextureVariant, &Ctx, &Sampler)> =
            vec![(TextureVariant::Staging, main, &sampler)];
        if let Some((c, s)) = &control {
            variants.push((TextureVariant::StagingControl, c, s));
        }
        if let Some((c, s)) = &implicit {
            variants.push((TextureVariant::Implicit, c, s));
        }
        if host_visible {
            variants.push((TextureVariant::Host, main, &sampler));
            if is_metal {
                variants.push((TextureVariant::HostOptimized, main, &sampler));
            }
        }
        let contents: &[Content] = if main.quick {
            &[Content::Smooth]
        } else if format.is_compressed() {
            &[Content::Smooth, Content::Noise]
        } else {
            &[Content::Smooth, Content::Detail, Content::Noise]
        };
        for &size in sizes {
            for &content in contents {
                let case = TextureCase::new(format, size, content);
                let name = case.name();
                println!("  textures: {name}");
                // Enough reads per dispatch that small textures aren't dominated by
                // dispatch overhead.
                let reps = sample_reps(size);
                let texel_reads = (size as f64) * (size as f64) * reps as f64;
                // Variants are measured in turn within every iteration, so clock
                // and thermal changes affect them equally.
                let mut samples: Vec<Samples> =
                    variants.iter().map(|_| Samples::default()).collect();
                // Short CPU-side timings are noisy, so small textures get more of them.
                let copies = if main.quick {
                    iterations
                } else {
                    match size {
                        ..=1024 => 21,
                        2048 => 11,
                        _ => 7,
                    }
                };
                // Each device's first texture of a size can pay for allocating memory.
                for &(variant, c, _) in &variants {
                    drop(case.upload_fresh(c, variant));
                }
                let mut textures = Vec::new();
                for iteration in 0..copies {
                    for (i, &(variant, c, s)) in variants.iter().enumerate() {
                        let (texture, ms) = case.upload_fresh(c, variant);
                        samples[i].fresh.push(ms);
                        if iteration == 0 && i == 0 && format == ASTC_4X4 {
                            let ok = check_astc(c, &case, &texture);
                            if let Err(e) = &ok {
                                eprintln!("  {name}: ASTC blocks didn't decode as expected: {e}");
                            }
                            records.push((
                                name.clone(),
                                "staging",
                                "decode_ok",
                                ok.is_ok() as u32 as f64,
                                "bool",
                            ));
                        }
                        if iteration < iterations {
                            samples[i].first_use.push(s.run(c, &texture, reps));
                        }
                        if iteration == copies - 1 {
                            textures.push(texture);
                        }
                    }
                }
                for _ in 0..copies {
                    for (i, &(variant, c, s)) in variants.iter().enumerate() {
                        let phases = case.upload_again(c, &textures[i], variant);
                        samples[i].again.push(phases.total);
                        samples[i].again_cpu.push(phases.cpu);
                        samples[i].again_wait.push(phases.wait);
                        s.run(c, &textures[i], 1);
                    }
                }
                for _ in 0..iterations * 2 {
                    for (i, &(_, c, s)) in variants.iter().enumerate() {
                        samples[i].steady.push(s.run(c, &textures[i], reps));
                    }
                }
                for _ in 0..copies {
                    for (i, &(variant, c, s)) in variants.iter().enumerate() {
                        s.run(c, &textures[i], 1);
                        samples[i]
                            .readback
                            .push(case.read_back(c, &textures[i], variant));
                    }
                }
                if main.took_error()
                    || control.as_ref().is_some_and(|(c, _)| c.took_error())
                    || implicit.as_ref().is_some_and(|(c, _)| c.took_error())
                {
                    eprintln!("  {name}: skipped after an error");
                    continue;
                }
                for (i, &(variant, _, _)) in variants.iter().enumerate() {
                    let s = &samples[i];
                    let v = variant.name();
                    let steady = median(s.steady.clone());
                    for (metric, values) in [
                        ("upload_fresh", &s.fresh),
                        ("upload_again", &s.again),
                        ("upload_again_cpu", &s.again_cpu),
                        ("upload_again_wait", &s.again_wait),
                        ("first_sample", &s.first_use),
                        ("readback", &s.readback),
                        ("sample", &s.steady),
                    ] {
                        records.push((name.clone(), v, metric, median(values.clone()), "ms"));
                    }
                    records.push((
                        name.clone(),
                        v,
                        "sample_rate",
                        texel_reads / (steady * 1e6),
                        "Gtexel/s",
                    ));
                    if i > 0 {
                        let base = &samples[0];
                        for (metric, values, base) in [
                            ("upload_fresh_vs_staging", &s.fresh, &base.fresh),
                            ("upload_again_vs_staging", &s.again, &base.again),
                            ("readback_vs_staging", &s.readback, &base.readback),
                            ("sample_vs_staging", &s.steady, &base.steady),
                        ] {
                            records.push((
                                name.clone(),
                                v,
                                metric,
                                paired_ratio(values, base),
                                "x",
                            ));
                        }
                    }
                    if matches!(
                        variant,
                        TextureVariant::Host | TextureVariant::HostOptimized
                    ) && let Some(allowed) = allows_gpu_optimized_contents(&textures[i])
                    {
                        records.push((
                            name.clone(),
                            v,
                            "allowGPUOptimizedContents",
                            allowed as u32 as f64,
                            "bool",
                        ));
                    }
                }
            }
        }
    }
    for (case, variant, metric, value, unit) in records {
        ctx.record("texture", &case, variant, metric, value, unit);
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

    // The implicit variant gets its own instance; everything else measures
    // `write_texture` through staging.
    let instance_with = |implicit: bool| {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::VULKAN | wgpu::Backends::METAL | wgpu::Backends::DX12;
        descriptor.flags = wgpu::InstanceFlags::empty();
        let mut descriptor = descriptor.with_env();
        descriptor.flags.set(
            wgpu::InstanceFlags::DISABLE_IMPLICIT_HOST_TEXTURE_COPY,
            !implicit,
        );
        wgpu::Instance::new(descriptor)
    };
    let instance = instance_with(false);
    let implicit_instance = instance_with(true);
    let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
    let implicit_adapters =
        pollster::block_on(implicit_instance.enumerate_adapters(wgpu::Backends::all()));

    let mut report = String::from("# host-copy-bench\n\n");
    let mut all_rows = Vec::new();
    let mut skipped = Vec::new();
    let mut benched = 0;
    for adapter in adapters {
        let info = adapter.get_info();
        let skip =
            if info.backend == wgpu::Backend::Gl || info.backend == wgpu::Backend::BrowserWebGpu {
                Some("OpenGL and WebGPU are not benchmarked")
            } else if info.device_type == wgpu::DeviceType::Cpu && !options.include_cpu {
                Some("software renderer; pass --include-cpu to include it")
            } else if let Some(filter) = &options.adapter_filter
                && !info.name.to_lowercase().contains(filter)
            {
                Some("doesn't match --adapter")
            } else {
                None
            };
        if let Some(reason) = skip {
            skipped.push(format!("{} ({:?}): {reason}", info.name, info.backend));
            continue;
        }
        benched += 1;
        println!("{} ({:?})", info.name, info.backend);
        let mut section = String::new();
        describe(&adapter, &mut section);

        let mut rows = Vec::new();
        if options.runs("textures") {
            describe_vulkan_host_copy(&adapter, &mut section, &mut rows);
        }
        if let Some(mut ctx) = Ctx::new(&adapter, options.quick) {
            if options.runs("textures") {
                let implicit = implicit_adapters
                    .iter()
                    .find(|a| {
                        let other = a.get_info();
                        (
                            other.name.as_str(),
                            other.backend,
                            other.vendor,
                            other.device,
                        ) == (info.name.as_str(), info.backend, info.vendor, info.device)
                    })
                    .and_then(|a| Ctx::new(a, options.quick));
                let control = Ctx::new(&adapter, options.quick);
                bench_textures(&mut ctx, control.as_ref(), implicit.as_ref());
            }
            if options.runs("buffers") {
                bench_buffers(&mut ctx);
                scenarios::buffer_readback(&mut ctx);
            }
            if options.runs("scenarios") {
                scenarios::render_targets(&mut ctx);
                scenarios::atlas_updates(&mut ctx);
                scenarios::mip_chains(&mut ctx);
                scenarios::threaded_uploads(&mut ctx);
                scenarios::busy_uploads(&mut ctx);
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

    if benched == 0 {
        eprintln!("error: no GPU was benchmarked.");
        for line in &skipped {
            eprintln!("  skipped {line}");
        }
        if skipped.is_empty() {
            eprintln!("  wgpu found no adapters. Check that the GPU drivers are installed.");
        }
        std::process::exit(2);
    }

    println!();
    print!("{report}");
    if let Some(path) = &options.csv {
        write_csv(path, &all_rows);
        println!("wrote {path}");
    }
}
