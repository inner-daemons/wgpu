//! Parsing WGSL the way wgpu does, and the override values to create pipelines with.

use naga::Module;

pub fn parse(source: &str) -> Result<Module, String> {
    let options = naga::front::wgsl::Options::new();
    naga::front::wgsl::Frontend::new_with_options(options)
        .parse(source)
        .map_err(|e| e.emit_to_string(source))
}

/// Values for the overrides that have no default, so every pipeline can be created.
pub fn override_constants(module: &Module) -> Vec<(String, f64)> {
    module
        .overrides
        .iter()
        .filter(|(_, o)| o.init.is_none())
        .filter_map(|(_, o)| {
            let key = match (&o.name, o.id) {
                (_, Some(id)) => id.to_string(),
                (Some(name), None) => name.clone(),
                (None, None) => return None,
            };
            Some((key, 1.0))
        })
        .collect()
}
