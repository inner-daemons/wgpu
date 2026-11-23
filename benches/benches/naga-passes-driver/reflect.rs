//! Reflection over naga modules, to build pipelines for arbitrary shaders.

use naga::{Binding, BuiltIn, EntryPoint, Module, ScalarKind, ShaderStage, TypeInner};

/// One shader input or output with a `@location`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationIo {
    pub location: u32,
    pub kind: ScalarKind,
    pub width: u8,
    pub components: u32,
}

#[derive(Debug, Default)]
pub struct Io {
    pub locations: Vec<LocationIo>,
    pub builtins: Vec<BuiltIn>,
}

fn collect_io(
    module: &Module,
    ty: naga::Handle<naga::Type>,
    binding: Option<&Binding>,
    io: &mut Io,
) {
    match binding {
        Some(Binding::BuiltIn(builtin)) => io.builtins.push(*builtin),
        Some(Binding::Location { location, .. }) => {
            let (kind, width, components) = match module.types[ty].inner {
                TypeInner::Scalar(scalar) => (scalar.kind, scalar.width, 1),
                TypeInner::Vector { size, scalar } => (scalar.kind, scalar.width, size as u32),
                _ => return,
            };
            io.locations.push(LocationIo {
                location: *location,
                kind,
                width,
                components,
            });
        }
        None => {
            if let TypeInner::Struct { ref members, .. } = module.types[ty].inner {
                for member in members {
                    collect_io(module, member.ty, member.binding.as_ref(), io);
                }
            }
        }
    }
}

pub fn inputs(module: &Module, ep: &EntryPoint) -> Io {
    let mut io = Io::default();
    for arg in &ep.function.arguments {
        collect_io(module, arg.ty, arg.binding.as_ref(), &mut io);
    }
    io.locations.sort_by_key(|l| l.location);
    io
}

pub fn outputs(module: &Module, ep: &EntryPoint) -> Io {
    let mut io = Io::default();
    if let Some(ref result) = ep.function.result {
        collect_io(module, result.ty, result.binding.as_ref(), &mut io);
    }
    io.locations.sort_by_key(|l| l.location);
    io
}

pub fn vertex_format(io: &LocationIo) -> Option<(wgpu::VertexFormat, u64)> {
    use wgpu::VertexFormat as F;
    let formats: [(F, u64); 4] = match (io.kind, io.width) {
        (ScalarKind::Float, 4) => [
            (F::Float32, 4),
            (F::Float32x2, 8),
            (F::Float32x3, 12),
            (F::Float32x4, 16),
        ],
        (ScalarKind::Uint, 4) => [
            (F::Uint32, 4),
            (F::Uint32x2, 8),
            (F::Uint32x3, 12),
            (F::Uint32x4, 16),
        ],
        (ScalarKind::Sint, 4) => [
            (F::Sint32, 4),
            (F::Sint32x2, 8),
            (F::Sint32x3, 12),
            (F::Sint32x4, 16),
        ],
        (ScalarKind::Float, 2) => [
            (F::Float16x2, 4),
            (F::Float16x2, 4),
            (F::Float16x4, 8),
            (F::Float16x4, 8),
        ],
        _ => return None,
    };
    Some(formats[io.components as usize - 1])
}

/// An 8-bit color target format that accepts the output, at most as wide as the output.
pub fn color_format(io: &LocationIo) -> Option<wgpu::TextureFormat> {
    use wgpu::TextureFormat as F;
    let formats = match io.kind {
        ScalarKind::Float => [F::R8Unorm, F::Rg8Unorm, F::Rg8Unorm, F::Rgba8Unorm],
        ScalarKind::Uint => [F::R8Uint, F::Rg8Uint, F::Rg8Uint, F::Rgba8Uint],
        ScalarKind::Sint => [F::R8Sint, F::Rg8Sint, F::Rg8Sint, F::Rgba8Sint],
        _ => return None,
    };
    Some(formats[io.components as usize - 1])
}

#[derive(Clone, Debug)]
pub enum PipelineSpec {
    Compute { ep: String },
    Render { vs: String, fs: Option<String> },
    Skipped { name: String, reason: String },
}

impl PipelineSpec {
    pub fn name(&self) -> String {
        match self {
            Self::Compute { ep } => ep.clone(),
            Self::Render { vs, fs: Some(fs) } => format!("{vs}+{fs}"),
            Self::Render { vs, fs: None } => vs.clone(),
            Self::Skipped { name, .. } => name.clone(),
        }
    }
}

/// The pipelines a module's entry points make.
///
/// Each fragment entry point is paired with the first vertex entry point whose outputs feed its
/// inputs, and the vertex entry points left over become vertex-only pipelines.
pub fn pipeline_specs(module: &Module) -> Vec<PipelineSpec> {
    let mut specs = Vec::new();
    let mut vertex = Vec::new();
    let mut fragment = Vec::new();
    for ep in &module.entry_points {
        match ep.stage {
            ShaderStage::Compute => specs.push(PipelineSpec::Compute {
                ep: ep.name.clone(),
            }),
            ShaderStage::Vertex => vertex.push(ep),
            ShaderStage::Fragment => fragment.push(ep),
            _ => specs.push(PipelineSpec::Skipped {
                name: ep.name.clone(),
                reason: format!("{:?} pipelines are not benchmarked", ep.stage),
            }),
        }
    }

    let mut used = vec![false; vertex.len()];
    for fs in fragment {
        let wanted = inputs(module, fs).locations;
        let compatible = vertex.iter().position(|vs| {
            let provided = outputs(module, vs).locations;
            wanted.iter().all(|w| provided.contains(w))
        });
        match compatible.or(if vertex.is_empty() { None } else { Some(0) }) {
            Some(index) => {
                used[index] = true;
                specs.push(PipelineSpec::Render {
                    vs: vertex[index].name.clone(),
                    fs: Some(fs.name.clone()),
                });
            }
            None => specs.push(PipelineSpec::Skipped {
                name: fs.name.clone(),
                reason: "fragment entry point without a vertex entry point".to_string(),
            }),
        }
    }
    for (vs, used) in vertex.iter().zip(used) {
        if !used {
            specs.push(PipelineSpec::Render {
                vs: vs.name.clone(),
                fs: None,
            });
        }
    }
    specs
}
