#![cfg_attr(target_arch = "wasm32", no_main)]
#![cfg(not(target_arch = "wasm32"))]
//! Pipeline creation and shader runtime on a real device, for comparing two checkouts across
//! drivers. See `benches/naga-passes/driver/README.md`.

use pollster::block_on;
use wgpu_benchmark::Benchmark;

#[path = "../naga-passes-common/mod.rs"]
mod common;
mod pipeline;
mod reflect;
mod runtime;
mod vary;

/// The features that shaders of the default corpus need, beyond timestamp queries. Only the ones
/// the adapter has are requested: asking for everything it reports can fail on some drivers.
const EXAMPLE_FEATURES: wgpu::Features = wgpu::Features::SHADER_F16
    .union(wgpu::Features::TEXTURE_BINDING_ARRAY)
    .union(wgpu::Features::BUFFER_BINDING_ARRAY)
    .union(wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY)
    .union(wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING)
    .union(wgpu::Features::IMMEDIATES)
    .union(wgpu::Features::MULTIVIEW)
    .union(wgpu::Features::EXPERIMENTAL_RAY_QUERY)
    .union(wgpu::Features::EXPERIMENTAL_COOPERATIVE_MATRIX);

/// More features, for the naga snapshot inputs (`NAGA_PASSES_CORPUS=snapshots`).
const SNAPSHOT_FEATURES: wgpu::Features = wgpu::Features::SHADER_F64
    .union(wgpu::Features::SHADER_I16)
    .union(wgpu::Features::CLIP_DISTANCES)
    .union(wgpu::Features::DUAL_SOURCE_BLENDING)
    .union(wgpu::Features::PRIMITIVE_INDEX)
    .union(wgpu::Features::SUBGROUP)
    .union(wgpu::Features::SUBGROUP_BARRIER)
    .union(wgpu::Features::SHADER_PER_VERTEX)
    .union(wgpu::Features::SHADER_DRAW_INDEX)
    .union(wgpu::Features::SHADER_BARYCENTRICS)
    .union(wgpu::Features::SHADER_INT64)
    .union(wgpu::Features::SHADER_FLOAT32_ATOMIC)
    .union(wgpu::Features::EXPERIMENTAL_RAY_HIT_VERTEX_RETURN);

struct DeviceState {
    adapter_info: wgpu::AdapterInfo,
    device: wgpu::Device,
    queue: wgpu::Queue,
    /// What naga may validate for this device, as wgpu computes it.
    capabilities: naga::valid::Capabilities,
}

fn required_features(adapter: &wgpu::Adapter) -> wgpu::Features {
    let mut wanted = wgpu::Features::TIMESTAMP_QUERY | EXAMPLE_FEATURES;
    if common::env_list("NAGA_PASSES_CORPUS", "bench,examples")
        .iter()
        .any(|corpus| corpus == "snapshots")
    {
        wanted |= SNAPSHOT_FEATURES;
    }
    if adapter.get_info().device_type == wgpu::DeviceType::Cpu {
        // llvmpipe reports it, but Vulkan device creation rejects it.
        wanted.remove(wgpu::Features::EXPERIMENTAL_COOPERATIVE_MATRIX);
    }
    wanted & adapter.features()
}

/// The adapter's limits, capped at the defaults, plus the few that the benchmark shaders need.
fn required_limits(adapter: &wgpu::Adapter, features: wgpu::Features) -> wgpu::Limits {
    let supported = adapter.limits();
    let mut limits = wgpu::Limits::default().or_worse_values_from(&supported);
    limits.max_compute_workgroup_storage_size = supported.max_compute_workgroup_storage_size;
    if features.contains(wgpu::Features::IMMEDIATES) {
        limits.max_immediate_size = supported.max_immediate_size.min(16);
    }
    if features.contains(wgpu::Features::MULTIVIEW) {
        limits.max_multiview_view_count = supported.max_multiview_view_count.min(2);
    }
    if features
        .intersects(wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::BUFFER_BINDING_ARRAY)
    {
        limits.max_binding_array_elements_per_shader_stage =
            supported.max_binding_array_elements_per_shader_stage.min(8);
        limits.max_binding_array_sampler_elements_per_shader_stage = supported
            .max_binding_array_sampler_elements_per_shader_stage
            .min(2);
    }
    if features.contains(wgpu::Features::EXPERIMENTAL_RAY_QUERY) {
        limits = limits.using_acceleration_structure_values(supported.clone());
    }
    limits
}

impl DeviceState {
    fn new() -> Self {
        let base_backend = if cfg!(target_os = "macos") {
            // We don't want to use Molten-VK on Mac.
            wgpu::Backends::METAL
        } else {
            wgpu::Backends::all()
        };

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::from_env().unwrap_or(base_backend),
            ..wgpu::InstanceDescriptor::new_without_display_handle_from_env()
        });

        let adapter = block_on(wgpu::util::initialize_adapter_from_env_or_default(
            &instance, None,
        ))
        .unwrap();

        let adapter_info = adapter.get_info();
        println!(
            "  Using adapter: {} ({:?})",
            adapter_info.name, adapter_info.backend
        );

        let features = required_features(&adapter);
        let experimental = features.intersects(
            wgpu::Features::EXPERIMENTAL_RAY_QUERY
                | wgpu::Features::EXPERIMENTAL_COOPERATIVE_MATRIX
                | wgpu::Features::EXPERIMENTAL_RAY_HIT_VERTEX_RETURN,
        );
        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: features,
            required_limits: required_limits(&adapter, features),
            default_queue: wgpu::QueueDescriptor { label: None },
            memory_hints: wgpu::MemoryHints::Performance,
            experimental_features: if experimental {
                unsafe { wgpu::ExperimentalFeatures::enabled() }
            } else {
                wgpu::ExperimentalFeatures::disabled()
            },
            label: None,
            trace: wgpu::Trace::Off,
        }))
        .unwrap();

        let capabilities = wgpu_naga_bridge::features_to_naga_capabilities(
            device.features(),
            adapter.get_downlevel_capabilities().flags,
        );
        Self {
            adapter_info,
            device,
            queue,
            capabilities,
        }
    }
}

fn main() {
    wgpu_benchmark::main(vec![
        Benchmark {
            name: "naga_passes::pipeline_creation",
            func: pipeline::pipeline_creation,
        },
        Benchmark {
            name: "naga_passes::gpu_runtime",
            func: runtime::gpu_runtime,
        },
    ]);
}
