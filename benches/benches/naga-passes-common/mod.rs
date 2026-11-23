//! Code shared by the `naga-passes-cpu` and `naga-passes-driver` benchmarks.
//!
//! Each writes `<NAGA_PASSES_OUT>/<name>.json`, which `benches/naga-passes/compare.py` reads.

pub mod corpus;
pub mod inspect;
pub mod results;

pub fn env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(default)
}

pub fn env_list(name: &str, default: &str) -> Vec<String> {
    std::env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}
