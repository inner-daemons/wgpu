#![cfg_attr(target_arch = "wasm32", no_main)]
#![cfg(not(target_arch = "wasm32"))]
//! The naga half of compiling a pipeline, on the CPU only. See `benches/naga-passes/README.md`.

use wgpu_benchmark::Benchmark;

#[path = "../naga-passes-common/mod.rs"]
mod common;
mod cpu;
mod naga_adapter;

fn main() {
    wgpu_benchmark::main(vec![Benchmark {
        name: "naga_passes::cpu_compile",
        func: cpu::cpu_compile,
    }]);
}
