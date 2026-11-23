//! How long the benchmark shaders take to run.
//!
//! Each shader is dispatched many times with the same inputs. The time of a dispatch comes from
//! timestamp queries on the compute pass when the device supports them, and otherwise from the
//! wall time around submit and poll. The first dispatch also reads back the results and hashes
//! them, so a pass that changes what a shader computes shows up as a different checksum.

use std::{
    borrow::Cow,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::mpsc,
    time::{Duration, Instant},
};

use pollster::block_on;
use wgpu_benchmark::{BenchmarkContext, SubBenchResult};

use crate::{
    common::{
        corpus::{self, ShaderSource},
        env_u32,
        results::{self, Record},
    },
    pipeline, DeviceState,
};

const BENCH: &str = "gpu_runtime";
/// Read by wgpu-hal's Vulkan backend on the bench branches only: use naga's zero-init polyfill
/// even when the device can zero workgroup memory natively.
const FORCE_POLYFILL_ENV: &str = "WGPU_BENCH_ONLY_FORCE_ZERO_INIT_POLYFILL";
const PARAMS_SIZE: u64 = 4096;
/// The largest workgroup size of the benchmark shaders.
const MAX_WORKGROUP_SIZE: u64 = 256;

struct Config {
    groups: u32,
    runs: u32,
    warmup: u32,
    cap: Duration,
    timestamps: bool,
}

/// The uniform buffer: `ctl` first, then small integers as floats for the matrix shaders.
fn params_bytes() -> Vec<u8> {
    let mut words = vec![0u32; PARAMS_SIZE as usize / 4];
    words[..4].copy_from_slice(&[12345, 32, 32, 128]);
    for (i, word) in words.iter_mut().enumerate().skip(4) {
        *word = (((i * 7 + 3) % 5) as f32).to_bits();
    }
    bytemuck::cast_slice(&words).to_vec()
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

fn read_buffer(device: &wgpu::Device, buffer: &wgpu::Buffer, size: u64) -> Result<Vec<u8>, String> {
    let slice = buffer.slice(..size);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|e| format!("poll: {e}"))?;
    receiver
        .recv()
        .map_err(|e| e.to_string())?
        .map_err(|e| format!("map: {e}"))?;
    let data = slice
        .get_mapped_range()
        .map_err(|e| format!("map range: {e}"))?
        .to_vec();
    buffer.unmap();
    Ok(data)
}

struct Resources {
    pipeline_layout: wgpu::PipelineLayout,
    bind_group: wgpu::BindGroup,
    results: wgpu::Buffer,
    results_readback: wgpu::Buffer,
    query_set: Option<wgpu::QuerySet>,
    resolve: wgpu::Buffer,
    timestamps_readback: wgpu::Buffer,
}

fn resources(state: &DeviceState, config: &Config) -> Resources {
    let device = &state.device;
    let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            entry(0, wgpu::BufferBindingType::Storage { read_only: false }),
            entry(1, wgpu::BufferBindingType::Uniform),
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });

    let results_size = u64::from(config.groups) * MAX_WORKGROUP_SIZE * 4;
    let results = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("results"),
        size: results_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let results_readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("results readback"),
        size: results_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let params = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("params"),
        size: PARAMS_SIZE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    state.queue.write_buffer(&params, 0, &params_bytes());

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: results.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: params.as_entire_binding(),
            },
        ],
    });

    let query_set = config.timestamps.then(|| {
        device.create_query_set(&wgpu::QuerySetDescriptor {
            label: None,
            ty: wgpu::QueryType::Timestamp,
            count: 2,
        })
    });
    let resolve = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("timestamps"),
        size: 16,
        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let timestamps_readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("timestamps readback"),
        size: 16,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    Resources {
        pipeline_layout,
        bind_group,
        results,
        results_readback,
        query_set,
        resolve,
        timestamps_readback,
    }
}

struct Run {
    wall_ms: f64,
    gpu_ms: Option<f64>,
}

fn dispatch(
    state: &DeviceState,
    res: &Resources,
    pipeline: &wgpu::ComputePipeline,
    groups: u32,
    use_timestamps: bool,
    readback: Option<u64>,
) -> Result<Run, String> {
    let device = &state.device;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    let timestamps = res.query_set.as_ref().filter(|_| use_timestamps);
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: timestamps.map(|query_set| wgpu::ComputePassTimestampWrites {
                query_set,
                beginning_of_pass_write_index: Some(0),
                end_of_pass_write_index: Some(1),
            }),
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &res.bind_group, &[]);
        pass.dispatch_workgroups(groups, 1, 1);
    }
    if let Some(query_set) = timestamps {
        encoder.resolve_query_set(query_set, 0..2, &res.resolve, 0);
        encoder.copy_buffer_to_buffer(&res.resolve, 0, &res.timestamps_readback, 0, 16);
    }
    if let Some(size) = readback {
        encoder.copy_buffer_to_buffer(&res.results, 0, &res.results_readback, 0, size);
    }

    let start = Instant::now();
    state.queue.submit([encoder.finish()]);
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|e| format!("poll: {e}"))?;
    let wall_ms = start.elapsed().as_secs_f64() * 1e3;

    let gpu_ms = if timestamps.is_some() {
        let bytes = read_buffer(device, &res.timestamps_readback, 16)?;
        let begin = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let end = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let ms =
            end.wrapping_sub(begin) as f64 * f64::from(state.queue.get_timestamp_period()) / 1e6;
        (end > begin && ms.is_finite()).then_some(ms)
    } else {
        None
    };
    Ok(Run { wall_ms, gpu_ms })
}

pub fn gpu_runtime(ctx: BenchmarkContext) -> anyhow::Result<Vec<SubBenchResult>> {
    let test = ctx.is_test();
    let state = DeviceState::new();
    let timestamps = state
        .device
        .features()
        .contains(wgpu::Features::TIMESTAMP_QUERY)
        && std::env::var_os("NAGA_PASSES_GPU_WALL").is_none();
    let config = Config {
        groups: if test {
            16
        } else {
            env_u32("NAGA_PASSES_GPU_GROUPS", 4096).clamp(1, 65535)
        },
        runs: if test {
            1
        } else {
            env_u32("NAGA_PASSES_GPU_RUNS", 40)
        },
        warmup: if test {
            0
        } else {
            env_u32("NAGA_PASSES_GPU_WARMUP", 5)
        },
        cap: Duration::from_millis(u64::from(env_u32("NAGA_PASSES_GPU_CAP_MS", 8000))),
        timestamps,
    };

    let mut store = results::Store::open(BENCH);
    pipeline::set_device_meta(&mut store, &state);
    store.set_meta("timestamp_queries", config.timestamps);
    store.set_meta("groups", config.groups);

    let limit = state.device.limits().max_compute_workgroup_storage_size;
    store.set_meta("max_compute_workgroup_storage_size", limit);
    let mut shaders = corpus::bench_shaders(limit);
    corpus::apply_filter(&mut shaders);

    let forced_polyfill = std::env::var_os(FORCE_POLYFILL_ENV).is_some();
    store.set_meta("zero_init_polyfill_forced", forced_polyfill);
    if forced_polyfill && state.adapter_info.backend != wgpu::Backend::Vulkan {
        for shader in &shaders {
            store.finish(
                &shader.name,
                vec![Record::failed(
                    BENCH,
                    shader.group,
                    &shader.name,
                    "gpu_time",
                    "skipped",
                    "N/A: only the Vulkan backend has a native zero-init mode",
                )],
            );
        }
        return Ok(Vec::new());
    }

    let res = resources(&state, &config);

    for shader in &shaders {
        if store.is_done(&shader.name) {
            continue;
        }
        store.begin(&shader.name);
        let outcome = catch_unwind(AssertUnwindSafe(|| measure(&state, &res, &config, shader)));
        let records = outcome.unwrap_or_else(|payload| {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_default();
            vec![Record::failed(
                BENCH,
                shader.group,
                &shader.name,
                "gpu_time",
                "panic",
                message,
            )]
        });
        store.finish(&shader.name, records);
    }

    Ok(store
        .records()
        .iter()
        .filter(|r| r.metric == "gpu_time" && r.status == "ok")
        .map(|r| SubBenchResult {
            name: r.case.clone(),
            avg_duration_per_iteration: Duration::from_secs_f64(r.mean / 1e3),
            iterations: r.n,
            throughput_unit: "dispatches".to_string(),
            throughput_count_per_iteration: 1,
        })
        .collect())
}

fn measure(
    state: &DeviceState,
    res: &Resources,
    config: &Config,
    shader: &ShaderSource,
) -> Vec<Record> {
    let device = &state.device;
    let case = shader.name.as_str();
    let fail = |status: &str, error: String| {
        vec![Record::failed(
            BENCH,
            shader.group,
            case,
            "gpu_time",
            status,
            error,
        )]
    };

    let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let memory = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
    let result = run_case(state, res, config, shader);
    let mut errors = Vec::new();
    for scope in [internal, memory, validation] {
        if let Some(error) = block_on(scope.pop()) {
            errors.push(error.to_string());
        }
    }
    match (result, errors.into_iter().next()) {
        (_, Some(error)) => fail("error", error),
        (Err(error), None) => fail("error", error),
        (Ok(records), None) => records,
    }
}

fn run_case(
    state: &DeviceState,
    res: &Resources,
    config: &Config,
    shader: &ShaderSource,
) -> Result<Vec<Record>, String> {
    let device = &state.device;
    let case = shader.name.as_str();

    let start = Instant::now();
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(case),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader.source)),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(case),
        layout: Some(&res.pipeline_layout),
        module: &module,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let create_ms = start.elapsed().as_secs_f64() * 1e3;

    // The first dispatch includes any compilation the driver deferred.
    let size = u64::from(config.groups) * u64::from(shader.workgroup_size) * 4;
    let first = dispatch(
        state,
        res,
        &pipeline,
        config.groups,
        config.timestamps,
        Some(size),
    )?;
    let checksum = fnv1a(&read_buffer(device, &res.results_readback, size)?);

    let mut use_timestamps = config.timestamps && first.gpu_ms.is_some();
    for _ in 0..config.warmup {
        let run = dispatch(state, res, &pipeline, config.groups, use_timestamps, None)?;
        use_timestamps &= run.gpu_ms.is_some();
    }

    let mut gpu = Vec::new();
    let mut wall = Vec::new();
    let begin = Instant::now();
    for _ in 0..config.runs {
        let run = dispatch(state, res, &pipeline, config.groups, use_timestamps, None)?;
        match (use_timestamps, run.gpu_ms) {
            (true, Some(ms)) => gpu.push(ms),
            (true, None) => return Err("timestamp query returned no usable time".to_string()),
            (false, _) => gpu.push(run.wall_ms),
        }
        wall.push(run.wall_ms);
        if gpu.len() >= 5 && begin.elapsed() > config.cap {
            break;
        }
    }

    let source = if use_timestamps { "timestamp" } else { "wall" };
    let annotate = |record: Record| {
        record
            .aux("timing_source", source)
            .aux("checksum", format!("{checksum:016x}"))
            .aux("groups", config.groups)
            .aux("workgroup_size", shader.workgroup_size)
            .aux("workgroup_bytes", shader.workgroup_bytes)
            .aux("first_run_ms", format!("{:.3}", first.wall_ms))
            .aux("create_ms", format!("{create_ms:.3}"))
    };
    Ok(vec![
        annotate(Record::ok(
            BENCH,
            shader.group,
            case,
            "gpu_time",
            "ms",
            &mut gpu,
        )),
        annotate(Record::ok(
            BENCH,
            shader.group,
            case,
            "wall_time",
            "ms",
            &mut wall,
        )),
    ])
}
