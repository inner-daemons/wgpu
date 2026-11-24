//! `create_shader_module` and pipeline creation on a real device, so the driver's compiler is
//! included.
//!
//! Every iteration compiles a slightly different shader (see `vary`), because drivers cache.
//! No `PipelineCache` is used. The first iteration of each pipeline is not part of the
//! statistics, but is reported as `first_ms`.

use std::{
    borrow::Cow,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use pollster::block_on;
use wgpu_benchmark::{BenchmarkContext, SubBenchResult};

use crate::{
    common::{
        corpus::{self, ShaderSource},
        env_list, env_u32, inspect,
        results::{self, Record},
    },
    reflect::{self, PipelineSpec},
    vary, DeviceState,
};

const BENCH: &str = "pipeline_creation";

pub fn set_device_meta(store: &mut results::Store, state: &DeviceState) {
    let info = &state.adapter_info;
    store.set_meta("os", std::env::consts::OS);
    store.set_meta("arch", std::env::consts::ARCH);
    store.set_meta("adapter_name", &info.name);
    store.set_meta("backend", format!("{:?}", info.backend));
    store.set_meta("device_type", format!("{:?}", info.device_type));
    store.set_meta("driver", &info.driver);
    store.set_meta("driver_info", &info.driver_info);
    store.set_meta("vendor_id", info.vendor);
    store.set_meta("device_id", info.device);
}

struct Timing {
    module_ms: f64,
    pipeline_ms: f64,
}

pub fn pipeline_creation(ctx: BenchmarkContext) -> anyhow::Result<Vec<SubBenchResult>> {
    let test = ctx.is_test();
    let iterations = if test {
        1
    } else {
        env_u32("NAGA_PASSES_PIPELINE_ITERS", 8)
    };
    let skip = env_list("NAGA_PASSES_SKIP", "");

    let state = DeviceState::new();
    let mut store = results::Store::open(BENCH);
    set_device_meta(&mut store, &state);

    let lost = Arc::new(AtomicBool::new(false));
    {
        let lost = lost.clone();
        state
            .device
            .set_device_lost_callback(move |reason, message| {
                eprintln!("  Device lost ({reason:?}): {message}");
                lost.store(true, Ordering::SeqCst);
            });
    }

    let limit = state.device.limits().max_compute_workgroup_storage_size;
    let shaders = corpus::select(limit);

    for shader in &shaders {
        let module = match inspect::parse(&shader.source) {
            Ok(module) => module,
            Err(e) => {
                store.finish(
                    &shader.name,
                    vec![Record::failed(
                        BENCH,
                        shader.group,
                        &shader.name,
                        "total",
                        "error",
                        format!("naga parse: {e}"),
                    )],
                );
                continue;
            }
        };

        let missing = missing_capability(&module, state.capabilities);
        for spec in reflect::pipeline_specs(&module) {
            let case = format!("{}::{}", shader.name, spec.name());
            if store.is_done(&case) {
                continue;
            }
            if let PipelineSpec::Skipped { reason, .. } = &spec {
                store.finish(
                    &case,
                    vec![Record::failed(
                        BENCH,
                        shader.group,
                        &case,
                        "total",
                        "skipped",
                        reason.clone(),
                    )],
                );
                continue;
            }
            if let Some(reason) = &missing {
                store.finish(
                    &case,
                    vec![Record::failed(
                        BENCH,
                        shader.group,
                        &case,
                        "total",
                        "skipped",
                        reason.clone(),
                    )],
                );
                continue;
            }
            if skip.iter().any(|s| case.contains(s.as_str())) {
                store.finish(
                    &case,
                    vec![Record::failed(
                        BENCH,
                        shader.group,
                        &case,
                        "total",
                        "skipped",
                        "listed in NAGA_PASSES_SKIP",
                    )],
                );
                continue;
            }
            if lost.load(Ordering::SeqCst) {
                store.finish(
                    &case,
                    vec![Record::failed(
                        BENCH,
                        shader.group,
                        &case,
                        "total",
                        "error",
                        "the device was lost earlier in this run",
                    )],
                );
                continue;
            }

            store.begin(&case);
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                measure(&state, shader, &module, &spec, &case, iterations)
            }));
            let records = outcome.unwrap_or_else(|payload| {
                let message = payload
                    .downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| payload.downcast_ref::<String>().cloned())
                    .unwrap_or_default();
                vec![Record::failed(
                    BENCH,
                    shader.group,
                    &case,
                    "total",
                    "panic",
                    message,
                )]
            });
            store.finish(&case, records);
        }
    }

    Ok(store
        .records()
        .iter()
        .filter(|r| r.metric == "total" && r.status == "ok")
        .map(|r| SubBenchResult {
            name: r.case.clone(),
            avg_duration_per_iteration: Duration::from_secs_f64(r.mean / 1e3),
            iterations: r.n,
            throughput_unit: "pipelines".to_string(),
            throughput_count_per_iteration: 1,
        })
        .collect())
}

/// Why the device cannot take the shader, if it validates with every capability but not with the
/// ones this device has.
fn missing_capability(
    module: &naga::Module,
    capabilities: naga::valid::Capabilities,
) -> Option<String> {
    let validate = |capabilities| {
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), capabilities)
            .validate(module)
    };
    let error = validate(capabilities).err()?;
    validate(naga::valid::Capabilities::all()).ok()?;
    let mut message = error.as_inner().to_string();
    let mut source = std::error::Error::source(error.as_inner());
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    Some(format!("needs a feature this device lacks: {message}"))
}

fn measure(
    state: &DeviceState,
    shader: &ShaderSource,
    module: &naga::Module,
    spec: &PipelineSpec,
    case: &str,
    iterations: u32,
) -> Vec<Record> {
    let eps: Vec<&str> = match spec {
        PipelineSpec::Compute { ep } => vec![ep],
        PipelineSpec::Render { vs, fs } => {
            std::iter::once(vs.as_str()).chain(fs.as_deref()).collect()
        }
        PipelineSpec::Skipped { .. } => unreachable!(),
    };
    let base_constants = inspect::override_constants(module);

    let warmup = iterations > 1;
    let total_runs = iterations + u32::from(warmup);
    let mut module_ms = Vec::new();
    let mut pipeline_ms = Vec::new();
    let mut total_ms = Vec::new();
    let mut first_ms = None;
    let mut kind = "";

    for run in 0..total_runs {
        let varied = vary::vary(&shader.source, &eps, vary::salt(run));
        kind = varied.kind;
        let mut constants = base_constants.clone();
        constants.extend(varied.constants.iter().cloned());

        match create(state, spec, &varied, &constants, &shader.name) {
            Ok(timing) => {
                let total = timing.module_ms + timing.pipeline_ms;
                if run == 0 {
                    first_ms = Some(total);
                    if warmup {
                        continue;
                    }
                }
                module_ms.push(timing.module_ms);
                pipeline_ms.push(timing.pipeline_ms);
                total_ms.push(total);
            }
            Err(error) => {
                return vec![
                    Record::failed(BENCH, shader.group, case, "total", "error", error)
                        .aux("variation", kind),
                ];
            }
        }
    }

    let mut records = Vec::new();
    for (metric, samples) in [
        ("create_shader_module", &mut module_ms),
        ("create_pipeline", &mut pipeline_ms),
        ("total", &mut total_ms),
    ] {
        let mut record =
            Record::ok(BENCH, shader.group, case, metric, "ms", samples).aux("variation", kind);
        if let Some(first) = first_ms {
            record = record.aux("first_ms", format!("{first:.3}"));
        }
        records.push(record);
    }
    records
}

fn create(
    state: &DeviceState,
    spec: &PipelineSpec,
    varied: &vary::Varied,
    constants: &[(String, f64)],
    label: &str,
) -> Result<Timing, String> {
    let device = &state.device;
    let constants: Vec<(&str, f64)> = constants.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let options = wgpu::PipelineCompilationOptions {
        constants: &constants,
        zero_initialize_workgroup_memory: true,
    };

    // The module is parsed again to find the entry point inputs and outputs, outside the timers.
    let (vertex_layout, targets, depth) = match spec {
        PipelineSpec::Render { vs, fs } => {
            let reflected = inspect::parse(&varied.source).map_err(|e| format!("reparse: {e}"))?;
            render_state(&reflected, &varied.entry_points, vs, fs.as_deref())?
        }
        _ => (None, Vec::new(), None),
    };

    let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let memory = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);

    let start = Instant::now();
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&varied.source)),
    });
    let module_ms = start.elapsed().as_secs_f64() * 1e3;

    let start = Instant::now();
    let pipeline_ms;
    match spec {
        PipelineSpec::Compute { .. } => {
            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(label),
                layout: None,
                module: &module,
                entry_point: Some(&varied.entry_points[0]),
                compilation_options: options,
                cache: None,
            });
            pipeline_ms = start.elapsed().as_secs_f64() * 1e3;
            drop(pipeline);
        }
        PipelineSpec::Render { fs, .. } => {
            let buffers: Vec<Option<wgpu::VertexBufferLayout<'_>>> = match &vertex_layout {
                Some((stride, attributes)) => vec![Some(wgpu::VertexBufferLayout {
                    array_stride: *stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes,
                })],
                None => Vec::new(),
            };
            let fragment = fs.as_ref().map(|_| wgpu::FragmentState {
                module: &module,
                entry_point: Some(&varied.entry_points[1]),
                compilation_options: options.clone(),
                targets: &targets,
            });
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: None,
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some(&varied.entry_points[0]),
                    compilation_options: options,
                    buffers: &buffers,
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: depth,
                multisample: wgpu::MultisampleState::default(),
                fragment,
                multiview_mask: None,
                cache: None,
            });
            pipeline_ms = start.elapsed().as_secs_f64() * 1e3;
            drop(pipeline);
        }
        PipelineSpec::Skipped { .. } => unreachable!(),
    }
    drop(module);

    let mut errors = Vec::new();
    for scope in [internal, memory, validation] {
        if let Some(error) = block_on(scope.pop()) {
            errors.push(error.to_string());
        }
    }
    let _ = device.poll(wgpu::PollType::Poll);
    if let Some(error) = errors.into_iter().next() {
        return Err(error);
    }
    Ok(Timing {
        module_ms,
        pipeline_ms,
    })
}

type RenderState = (
    Option<(u64, Vec<wgpu::VertexAttribute>)>,
    Vec<Option<wgpu::ColorTargetState>>,
    Option<wgpu::DepthStencilState>,
);

/// Vertex buffer layout, color targets and depth state that fit the entry points.
fn render_state(
    module: &naga::Module,
    names: &[String],
    vs: &str,
    fs: Option<&str>,
) -> Result<RenderState, String> {
    // `names` are the renamed entry points, in the order [vertex, fragment]. Entry points are
    // looked up by those names in the module that was parsed from the varied source.
    let find = |name: &str| {
        module
            .entry_points
            .iter()
            .find(|ep| ep.name == name)
            .ok_or_else(|| format!("entry point {name} not found"))
    };

    let vertex = find(&names[0])?;
    let mut attributes = Vec::new();
    let mut offset = 0u64;
    for input in reflect::inputs(module, vertex).locations {
        let (format, size) = reflect::vertex_format(&input).ok_or_else(|| {
            format!(
                "{vs}: unsupported vertex input type at location {}",
                input.location
            )
        })?;
        let align = if size % 4 == 0 { 4 } else { 2 };
        offset = offset.next_multiple_of(align);
        attributes.push(wgpu::VertexAttribute {
            format,
            offset,
            shader_location: input.location,
        });
        offset += size;
    }
    let layout = (!attributes.is_empty()).then(|| (offset.next_multiple_of(4), attributes));

    let mut targets = Vec::new();
    let mut writes_depth = false;
    if let Some(fs) = fs {
        let fragment = find(&names[1])?;
        let outputs = reflect::outputs(module, fragment);
        writes_depth = outputs.builtins.contains(&naga::BuiltIn::FragDepth);
        for output in outputs.locations {
            let format = reflect::color_format(&output).ok_or_else(|| {
                format!(
                    "{fs}: unsupported fragment output type at location {}",
                    output.location
                )
            })?;
            while targets.len() <= output.location as usize {
                targets.push(None);
            }
            targets[output.location as usize] = Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            });
        }
    }

    let depth = (writes_depth || fs.is_none()).then(|| wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: Some(true),
        depth_compare: Some(wgpu::CompareFunction::Always),
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    });
    Ok((layout, targets, depth))
}
