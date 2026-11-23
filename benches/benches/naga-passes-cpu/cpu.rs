//! The naga half of creating one pipeline, on the CPU only.
//!
//! One iteration is what wgpu does for one entry point: parse the WGSL, validate it, resolve the
//! overrides for that entry point, and write it with a backend. Parsing and validation are
//! shared between backends in wgpu, but each backend is timed separately here so they can be
//! reported per backend, by running them one after another on the same validated module.

use std::{
    hint::black_box,
    panic::{catch_unwind, AssertUnwindSafe},
    time::{Duration, Instant},
};

use naga::{
    back::{glsl, hlsl, msl, spv},
    proc::{BoundsCheckPolicies, BoundsCheckPolicy},
    valid::{Capabilities, ShaderStages, SubgroupOperationSet, ValidationFlags, Validator},
    ShaderStage,
};
use wgpu_benchmark::{BenchmarkContext, SubBenchResult};

use crate::common::{corpus, env_u32, inspect, results};
use crate::naga_adapter;

const BENCH: &str = "cpu_compile";

/// The backends, in the order they run within an iteration.
const BACKENDS: [&str; 5] = ["spv", "spv_polyfill", "hlsl", "msl", "glsl"];

struct Options {
    spv_native: spv::Options<'static>,
    spv_polyfill: spv::Options<'static>,
    hlsl: hlsl::Options,
    msl: msl::Options,
    glsl: glsl::Options,
    glsl_policies: BoundsCheckPolicies,
}

const TASK_LIMITS: naga::back::TaskDispatchLimits = naga::back::TaskDispatchLimits {
    max_mesh_workgroups_per_dim: 65535,
    max_mesh_workgroups_total: 4_194_304,
};

fn restrict_all() -> BoundsCheckPolicies {
    BoundsCheckPolicies {
        index: BoundsCheckPolicy::Restrict,
        buffer: BoundsCheckPolicy::Restrict,
        image_load: BoundsCheckPolicy::Restrict,
        binding_array: BoundsCheckPolicy::Unchecked,
    }
}

/// Options like the ones wgpu-hal builds for a device without robust buffer access: workgroup
/// zero initialization on, loop bounding on, and `Restrict` bounds checks.
fn options() -> Options {
    let spv = |zero_initialize_workgroup_memory| spv::Options {
        lang_version: (1, 6),
        flags: spv::WriterFlags::LABEL_VARYINGS | spv::WriterFlags::FORCE_POINT_SIZE,
        fake_missing_bindings: true,
        bounds_check_policies: restrict_all(),
        zero_initialize_workgroup_memory,
        force_loop_bounding: true,
        ray_query_initialization_tracking: true,
        task_dispatch_limits: Some(TASK_LIMITS),
        mesh_shader_primitive_indices_clamp: true,
        emit_int_div_checks: true,
        ..Default::default()
    };

    Options {
        spv_native: spv(spv::ZeroInitializeWorkgroupMemoryMode::Native),
        spv_polyfill: spv(spv::ZeroInitializeWorkgroupMemoryMode::Polyfill),
        hlsl: hlsl::Options {
            shader_model: hlsl::ShaderModel::V6_6,
            fake_missing_bindings: true,
            zero_initialize_workgroup_memory: true,
            restrict_indexing: true,
            force_loop_bounding: true,
            task_dispatch_limits: Some(TASK_LIMITS),
            ..Default::default()
        },
        msl: msl::Options {
            lang_version: (3, 1),
            fake_missing_bindings: true,
            bounds_check_policies: restrict_all(),
            zero_initialize_workgroup_memory: true,
            force_loop_bounding: true,
            task_dispatch_limits: Some(TASK_LIMITS),
            ..Default::default()
        },
        glsl: glsl::Options {
            version: glsl::Version::new_gles(310),
            writer_flags: glsl::WriterFlags::ADJUST_COORDINATE_SPACE
                | glsl::WriterFlags::TEXTURE_SHADOW_LOD
                | glsl::WriterFlags::FORCE_POINT_SIZE,
            zero_initialize_workgroup_memory: true,
            ..Default::default()
        },
        glsl_policies: BoundsCheckPolicies::default(),
    }
}

#[derive(Default)]
struct Samples {
    /// Microseconds, one per iteration.
    parse: Vec<f64>,
    validate: Vec<f64>,
    overrides: Vec<f64>,
    backends: [Vec<f64>; BACKENDS.len()],
    total: Vec<f64>,
}

struct Timing {
    parse: f64,
    validate: f64,
    overrides: f64,
    backends: [Option<f64>; BACKENDS.len()],
}

fn micros(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1e6
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "panic".to_string())
}

struct Case<'a> {
    source: &'a str,
    stage: ShaderStage,
    name: &'a str,
    constants: naga::back::PipelineConstants,
    options: &'a Options,
}

impl Case<'_> {
    /// One full iteration. `failed` marks the backends to skip, and is updated when one fails.
    fn iterate(
        &self,
        failed: &mut [Option<(&'static str, String)>; BACKENDS.len()],
        output_bytes: &mut [usize; BACKENDS.len()],
    ) -> Result<Timing, String> {
        let start = Instant::now();
        let module = inspect::parse(self.source).map_err(|e| format!("parse: {e}"))?;
        let parse = micros(start.elapsed());

        let start = Instant::now();
        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
        validator
            .subgroup_stages(ShaderStages::all())
            .subgroup_operations(SubgroupOperationSet::all());
        let info = validator
            .validate(&module)
            .map_err(|e| format!("validation: {e:?}"))?;
        let validate = micros(start.elapsed());

        let start = Instant::now();
        let (module, info) = naga::back::pipeline_constants::process_overrides(
            &module,
            &info,
            Some((self.stage, self.name)),
            &self.constants,
        )
        .map_err(|e| format!("process_overrides: {e}"))?;
        let overrides = micros(start.elapsed());

        let ep = self.name.to_string();
        let spv_pipeline = spv::PipelineOptions {
            shader_stage: self.stage,
            entry_point: ep.clone(),
        };
        let hlsl_pipeline = hlsl::PipelineOptions {
            entry_point: Some((self.stage, ep.clone())),
            multisampled: false,
        };
        let msl_pipeline = msl::PipelineOptions {
            entry_point: Some((self.stage, ep.clone())),
            allow_and_force_point_size: false,
            vertex_pulling_transform: false,
            ..Default::default()
        };
        let glsl_pipeline = glsl::PipelineOptions {
            shader_stage: self.stage,
            entry_point: ep,
            multiview: None,
        };

        let options = self.options;
        let mut times = [None; BACKENDS.len()];
        for (index, backend) in BACKENDS.iter().enumerate() {
            if failed[index].is_some() {
                continue;
            }
            let start = Instant::now();
            let result = catch_unwind(AssertUnwindSafe(|| match *backend {
                "spv" => {
                    naga_adapter::write_spv(&module, &info, &options.spv_native, &spv_pipeline)
                }
                "spv_polyfill" => {
                    naga_adapter::write_spv(&module, &info, &options.spv_polyfill, &spv_pipeline)
                }
                "hlsl" => naga_adapter::write_hlsl(&module, &info, &options.hlsl, &hlsl_pipeline),
                "msl" => naga_adapter::write_msl(&module, &info, &options.msl, &msl_pipeline),
                _ => naga_adapter::write_glsl(
                    &module,
                    &info,
                    &options.glsl,
                    &glsl_pipeline,
                    options.glsl_policies,
                ),
            }));
            let elapsed = micros(start.elapsed());
            match result {
                Ok(Ok(bytes)) => {
                    output_bytes[index] = black_box(bytes);
                    times[index] = Some(elapsed);
                }
                Ok(Err(e)) => failed[index] = Some(("error", e)),
                Err(payload) => failed[index] = Some(("panic", panic_message(payload))),
            }
        }
        Ok(Timing {
            parse,
            validate,
            overrides,
            backends: times,
        })
    }
}

pub fn cpu_compile(ctx: BenchmarkContext) -> anyhow::Result<Vec<SubBenchResult>> {
    let test = ctx.is_test();
    let min_iterations = if test {
        1
    } else {
        env_u32("NAGA_PASSES_CPU_MIN_ITERS", 10)
    };
    let max_iterations = if test {
        1
    } else {
        env_u32("NAGA_PASSES_CPU_MAX_ITERS", 5000)
    };
    let budget = Duration::from_millis(u64::from(env_u32("NAGA_PASSES_CPU_BUDGET_MS", 150)));

    let mut store = results::Store::open(BENCH);
    store.set_meta("naga_api", naga_adapter::NAGA_API);
    store.set_meta("os", std::env::consts::OS);
    store.set_meta("arch", std::env::consts::ARCH);

    let options = options();
    // The shaders that size themselves from the device limit use the wgpu default plus margin.
    let shaders = corpus::select(32768);

    for shader in &shaders {
        let module = match inspect::parse(&shader.source) {
            Ok(module) => module,
            Err(e) => {
                store.finish(
                    &shader.name,
                    vec![results::Record::failed(
                        BENCH,
                        shader.group,
                        &shader.name,
                        "total",
                        "error",
                        format!("parse: {e}"),
                    )],
                );
                continue;
            }
        };

        let mut constants = naga::back::PipelineConstants::default();
        for (key, value) in inspect::override_constants(&module) {
            constants.insert(key, value);
        }

        for ep in &module.entry_points {
            let case_name = format!("{}::{}", shader.name, ep.name);
            if store.is_done(&case_name) {
                continue;
            }
            store.begin(&case_name);

            let case = Case {
                source: &shader.source,
                stage: ep.stage,
                name: &ep.name,
                constants: constants.clone(),
                options: &options,
            };
            let mut records = measure(
                &case,
                shader.group,
                &case_name,
                test,
                min_iterations,
                max_iterations,
                budget,
            );
            if shader.workgroup_bytes > 0 {
                for record in &mut records {
                    record.aux.insert(
                        "workgroup_size".to_string(),
                        shader.workgroup_size.to_string(),
                    );
                    record.aux.insert(
                        "workgroup_bytes".to_string(),
                        shader.workgroup_bytes.to_string(),
                    );
                }
            }
            store.finish(&case_name, records);
        }
    }

    Ok(store
        .records()
        .iter()
        .filter(|r| r.metric == "total" && r.status == "ok")
        .map(|r| SubBenchResult {
            name: r.case.clone(),
            avg_duration_per_iteration: Duration::from_secs_f64(r.mean / 1e6),
            iterations: r.n,
            throughput_unit: "pipelines".to_string(),
            throughput_count_per_iteration: 1,
        })
        .collect())
}

fn measure(
    case: &Case<'_>,
    group: &str,
    case_name: &str,
    test: bool,
    min_iterations: u32,
    max_iterations: u32,
    budget: Duration,
) -> Vec<results::Record> {
    let mut failed: [Option<(&'static str, String)>; BACKENDS.len()] = Default::default();
    let mut output_bytes = [0usize; BACKENDS.len()];
    let mut samples = Samples::default();

    // The first iteration warms caches and finds the backends that fail.
    if !test {
        if let Err(e) = case.iterate(&mut failed, &mut output_bytes) {
            return vec![results::Record::failed(
                BENCH, group, case_name, "total", "error", e,
            )];
        }
    }

    let begin = Instant::now();
    let mut iterations = 0;
    loop {
        match case.iterate(&mut failed, &mut output_bytes) {
            Ok(timing) => {
                samples.parse.push(timing.parse);
                samples.validate.push(timing.validate);
                samples.overrides.push(timing.overrides);
                let mut total = timing.parse + timing.validate + timing.overrides;
                for (index, time) in timing.backends.iter().enumerate() {
                    if let Some(time) = time {
                        samples.backends[index].push(*time);
                        // spv_polyfill is an alternative to spv, not another pipeline.
                        if BACKENDS[index] != "spv_polyfill" {
                            total += time;
                        }
                    }
                }
                samples.total.push(total);
            }
            Err(e) => {
                return vec![results::Record::failed(
                    BENCH, group, case_name, "total", "error", e,
                )];
            }
        }
        iterations += 1;
        let done = iterations >= max_iterations
            || (iterations >= min_iterations && begin.elapsed() >= budget);
        if done {
            break;
        }
    }

    let mut records = Vec::new();
    let push = |records: &mut Vec<results::Record>,
                metric: &str,
                samples: &mut Vec<f64>,
                bytes: Option<usize>| {
        let mut record = results::Record::ok(BENCH, group, case_name, metric, "us", samples);
        if let Some(bytes) = bytes {
            record = record.aux("output_bytes", bytes);
        }
        records.push(record);
    };
    push(&mut records, "parse", &mut samples.parse, None);
    push(&mut records, "validate", &mut samples.validate, None);
    push(&mut records, "overrides", &mut samples.overrides, None);
    for (index, backend) in BACKENDS.iter().enumerate() {
        match &failed[index] {
            None => push(
                &mut records,
                backend,
                &mut samples.backends[index],
                Some(output_bytes[index]),
            ),
            Some((status, error)) => records.push(results::Record::failed(
                BENCH,
                group,
                case_name,
                backend,
                status,
                error.clone(),
            )),
        }
    }
    push(&mut records, "total", &mut samples.total, None);
    records
}
