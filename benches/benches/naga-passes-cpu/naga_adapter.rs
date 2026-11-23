//! The only part of the naga pass benchmarks that depends on the naga backend API.
//!
//! This file differs between the `trunk` and `naga-passes-combined` branches. Each function
//! runs one backend writer the way wgpu-hal does and returns the output size in bytes.

use naga::{back, valid::ModuleInfo, Module};

pub const NAGA_API: &str = "combined";

pub fn write_spv(
    module: &Module,
    _info: &ModuleInfo,
    options: &back::spv::Options<'_>,
    pipeline: &back::spv::PipelineOptions,
) -> Result<usize, String> {
    back::spv::write_vec(module, options, Some(pipeline))
        .map(|words| words.len() * 4)
        .map_err(|e| e.to_string())
}

pub fn write_hlsl(
    module: &Module,
    _info: &ModuleInfo,
    options: &back::hlsl::Options,
    pipeline: &back::hlsl::PipelineOptions,
) -> Result<usize, String> {
    let mut out = String::new();
    back::hlsl::Writer::new(&mut out, options, pipeline)
        .write(module, None)
        .map_err(|e| format!("{e:?}"))?;
    Ok(out.len())
}

pub fn write_msl(
    module: &Module,
    _info: &ModuleInfo,
    options: &back::msl::Options,
    pipeline: &back::msl::PipelineOptions,
) -> Result<usize, String> {
    back::msl::write_string(module, options, pipeline)
        .map(|(source, _)| source.len())
        .map_err(|e| format!("{e:?}"))
}

pub fn write_glsl(
    module: &Module,
    _info: &ModuleInfo,
    options: &back::glsl::Options,
    pipeline: &back::glsl::PipelineOptions,
    policies: naga::proc::BoundsCheckPolicies,
) -> Result<usize, String> {
    let mut out = String::new();
    back::glsl::Writer::new(&mut out, options, pipeline, policies)
        .write(module)
        .map_err(|e| e.to_string())?;
    Ok(out.len())
}
