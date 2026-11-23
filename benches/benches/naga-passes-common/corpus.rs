use std::{fs, path::Path};

use super::env_list;

const ZERO_INIT_ARRAY: &str = include_str!("shaders/zero_init_array.wgsl");
const ZERO_INIT_2D: &str = include_str!("shaders/zero_init_2d.wgsl");
const ZERO_INIT_ATOMIC: &str = include_str!("shaders/zero_init_atomic.wgsl");
const LOOPS: &str = include_str!("shaders/loops.wgsl");
const HELPERS: &str = include_str!("shaders/helpers.wgsl");
const MATCX2: &str = include_str!("shaders/matcx2.wgsl");

const EXAMPLES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/features/src");
const SNAPSHOTS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../naga/tests/in/wgsl");

#[derive(Clone, Debug)]
pub struct ShaderSource {
    /// `bench`, `example` or `snapshot`.
    pub group: &'static str,
    pub name: String,
    pub source: String,
    /// Workgroup size of the `main` entry point of a `bench` shader.
    pub workgroup_size: u32,
    /// Bytes of `var<workgroup>` memory of a `bench` shader.
    pub workgroup_bytes: u32,
}

/// Replace the value of `const NAME: u32 = ...;` declarations.
fn instantiate(template: &str, values: &[(&str, u32)]) -> String {
    let mut source = template.to_string();
    for (name, value) in values {
        let needle = format!("const {name}: u32 = ");
        let start = source
            .find(&needle)
            .unwrap_or_else(|| panic!("benchmark shader has no `{needle}`"));
        let end = start + source[start..].find(';').unwrap();
        source.replace_range(start..=end, &format!("{needle}{value}u;"));
    }
    source
}

/// The shaders written for this benchmark.
///
/// `max_storage_bytes` is `max_compute_workgroup_storage_size`; the `max` shaders use almost all
/// of it.
pub fn bench_shaders(max_storage_bytes: u32) -> Vec<ShaderSource> {
    let max_words = (max_storage_bytes.saturating_sub(1024) / 4).max(1);
    let mut shaders = Vec::new();
    let mut push = |name: String, template: &str, values: &[(&str, u32)], wg: u32, bytes: u32| {
        shaders.push(ShaderSource {
            group: "bench",
            name,
            source: instantiate(template, values),
            workgroup_size: wg,
            workgroup_bytes: bytes,
        });
    };

    for wg in [64, 256] {
        for (label, words) in [("1k", 256), ("16k", 4096), ("max", max_words)] {
            push(
                format!("zeroinit_{label}_wg{wg}"),
                ZERO_INIT_ARRAY,
                &[("WG", wg), ("WORDS", words)],
                wg,
                words * 4,
            );
        }
        push(
            format!("zeroinit_2d_wg{wg}"),
            ZERO_INIT_2D,
            &[("WG", wg)],
            wg,
            32 * 64 * 4,
        );
        push(
            format!("zeroinit_atomic_wg{wg}"),
            ZERO_INIT_ATOMIC,
            &[("WG", wg)],
            wg,
            1024 * 4 + 4,
        );
        push(format!("loops_wg{wg}"), LOOPS, &[("WG", wg)], wg, 0);
        push(format!("helpers_wg{wg}"), HELPERS, &[("WG", wg)], wg, 0);
        push(format!("matcx2_wg{wg}"), MATCX2, &[("WG", wg)], wg, 0);
    }
    shaders
}

fn wgsl_files(dir: &Path, recursive: bool, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                wgsl_files(&path, recursive, out);
            }
        } else if path.extension().is_some_and(|ext| ext == "wgsl") {
            out.push(path);
        }
    }
}

fn read_dir_shaders(
    dir: &str,
    recursive: bool,
    group: &'static str,
    prefix: &str,
) -> Vec<ShaderSource> {
    let root = Path::new(dir);
    let mut files = Vec::new();
    wgsl_files(root, recursive, &mut files);
    files.sort();
    files
        .into_iter()
        .filter_map(|path| {
            let source = fs::read_to_string(&path).ok()?;
            let relative = path.strip_prefix(root).ok()?.to_string_lossy();
            Some(ShaderSource {
                group,
                name: format!("{prefix}/{}", relative.replace('\\', "/")),
                source,
                workgroup_size: 0,
                workgroup_bytes: 0,
            })
        })
        .collect()
}

/// wgpu's own feature examples.
pub fn example_shaders() -> Vec<ShaderSource> {
    read_dir_shaders(EXAMPLES_DIR, true, "example", "examples")
}

/// The inputs of naga's snapshot tests, without the very large ones.
pub fn snapshot_shaders() -> Vec<ShaderSource> {
    read_dir_shaders(SNAPSHOTS_DIR, false, "snapshot", "snapshot")
        .into_iter()
        .filter(|shader| !shader.name.contains("large-source"))
        .collect()
}

/// The shaders the benchmarks run.
///
/// `NAGA_PASSES_CORPUS` is a comma separated list of `bench`, `examples` and `snapshots`
/// (default `bench,examples`). `NAGA_PASSES_FILTER` is a comma separated list of substrings of
/// shader names; if set, only matching shaders run.
pub fn select(max_storage_bytes: u32) -> Vec<ShaderSource> {
    let wanted = env_list("NAGA_PASSES_CORPUS", "bench,examples");
    let mut shaders = Vec::new();
    if wanted.iter().any(|w| w == "bench") {
        shaders.extend(bench_shaders(max_storage_bytes));
    }
    if wanted.iter().any(|w| w == "examples") {
        shaders.extend(example_shaders());
    }
    if wanted.iter().any(|w| w == "snapshots") {
        shaders.extend(snapshot_shaders());
    }
    apply_filter(&mut shaders);
    shaders
}

pub fn apply_filter(shaders: &mut Vec<ShaderSource>) {
    let filter = env_list("NAGA_PASSES_FILTER", "");
    if !filter.is_empty() {
        shaders.retain(|shader| filter.iter().any(|f| shader.name.contains(f.as_str())));
    }
}
