//! Compares wgpu's new mappable buffer placement with trunk's.
//!
//! See `README.md` for what each group measures and how to run it.

use std::fmt::Write as _;

mod common;
mod controls;
mod info;
mod mappable_readwrite;
mod mappable_write;
mod output;
mod staging_belt;
mod upload;
mod write_texture;

#[cfg(not(target_arch = "wasm32"))]
mod bar_pressure;

use common::{Pair, Rec};

const GROUPS: [&str; 8] = [
    "info",
    "upload",
    "write_texture",
    "staging_belt",
    "mappable_write",
    "mappable_readwrite",
    "controls",
    "bar_pressure",
];

struct Options {
    quick: bool,
    csv: Option<String>,
    memory_json: Option<String>,
    adapter_filter: Option<String>,
    include_cpu: bool,
    skip: Vec<String>,
}

impl Options {
    fn parse() -> Self {
        let mut options = Options {
            quick: false,
            csv: None,
            memory_json: None,
            adapter_filter: None,
            include_cpu: false,
            skip: Vec::new(),
        };
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--quick" => options.quick = true,
                "--csv" => options.csv = args.next(),
                "--memory-json" => options.memory_json = args.next(),
                "--adapter" => options.adapter_filter = args.next().map(|s| s.to_lowercase()),
                "--include-cpu" => options.include_cpu = true,
                "--skip" => {
                    let Some(group) = args.next() else {
                        eprintln!("--skip needs a group name, see --help");
                        std::process::exit(2);
                    };
                    if !GROUPS.contains(&group.as_str()) {
                        eprintln!("unknown group {group:?}, see --help");
                        std::process::exit(2);
                    }
                    options.skip.push(group);
                }
                "--help" | "-h" => {
                    println!(
                        "usage: buffer-placement-bench [--quick] [--csv FILE] [--memory-json FILE] \
                         [--adapter NAME] [--include-cpu] [--skip GROUP]...\n\
                         groups: {}",
                        GROUPS.join(", ")
                    );
                    std::process::exit(0);
                }
                other => {
                    eprintln!("unknown argument {other:?}, see --help");
                    std::process::exit(2);
                }
            }
        }
        options
    }

    fn runs(&self, group: &str) -> bool {
        !self.skip.iter().any(|s| s == group)
    }
}

fn main() {
    env_logger::init();
    let options = Options::parse();
    if cfg!(debug_assertions) {
        eprintln!(
            "warning: this is a debug build; run with `cargo run --release` for real numbers"
        );
    }

    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::VULKAN | wgpu::Backends::METAL | wgpu::Backends::DX12;
    descriptor.flags = wgpu::InstanceFlags::empty();
    let instance = wgpu::Instance::new(descriptor.with_env());
    let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));

    let mut report = String::from("# buffer-placement-bench\n\n");
    let mut all_rows = Vec::new();
    let mut failures = Vec::new();
    let mut memory_json = Vec::new();
    let mut skipped = Vec::new();
    let mut benched = 0;
    for adapter in adapters {
        let info = adapter.get_info();
        let skip =
            if info.backend == wgpu::Backend::Gl || info.backend == wgpu::Backend::BrowserWebGpu {
                Some("OpenGL and WebGPU are not benchmarked")
            } else if info.device_type == wgpu::DeviceType::Cpu && !options.include_cpu {
                Some("software renderer; pass --include-cpu to include it")
            } else if let Some(filter) = &options.adapter_filter
                && !info.name.to_lowercase().contains(filter)
            {
                Some("doesn't match --adapter")
            } else {
                None
            };
        if let Some(reason) = skip {
            skipped.push(format!("{} ({:?}): {reason}", info.name, info.backend));
            continue;
        }
        benched += 1;
        println!("{} ({:?})", info.name, info.backend);
        let mut section = String::new();
        let _ = writeln!(section, "## {} ({:?})", info.name, info.backend);
        let _ = writeln!(section);
        let _ = writeln!(section, "- device type: {:?}", info.device_type);
        let _ = writeln!(section, "- driver: {} {}", info.driver, info.driver_info);
        let _ = writeln!(
            section,
            "- vendor/device: {:#x}/{:#x}",
            info.vendor, info.device
        );

        let mut rec = Rec::new(&info);
        match Pair::new(&adapter, options.quick) {
            Ok(pair) => run_groups(
                &options,
                &adapter,
                &pair,
                &mut rec,
                &mut section,
                &mut memory_json,
            ),
            Err(e) => {
                eprintln!("  {e}");
                let _ = writeln!(section, "- {e}");
                rec.failures
                    .push(format!("{} ({:?}): {e}", info.name, info.backend));
            }
        }
        report.push_str(&section);
        failures.append(&mut rec.failures);
        all_rows.append(&mut rec.rows);
    }

    if benched == 0 {
        eprintln!("error: no GPU was benchmarked.");
        for line in &skipped {
            eprintln!("  skipped {line}");
        }
        if skipped.is_empty() {
            eprintln!("  wgpu found no adapters. Check that the GPU drivers are installed.");
        }
        std::process::exit(2);
    }

    println!();
    print!("{report}");
    if let Some(path) = &options.csv {
        output::write_csv(path, &all_rows);
        println!("wrote {}", file_name(path));
    }
    if let Some(path) = &options.memory_json {
        let json = serde_json::json!({ "adapters": memory_json });
        std::fs::write(path, serde_json::to_string_pretty(&json).unwrap())
            .expect("failed to write the memory properties file");
        println!("wrote {}", file_name(path));
    }
    if !failures.is_empty() {
        eprintln!("{} check(s) failed:", failures.len());
        for failure in &failures {
            eprintln!("  {failure}");
        }
        std::process::exit(3);
    }
}

fn run_groups(
    options: &Options,
    adapter: &wgpu::Adapter,
    pair: &Pair,
    rec: &mut Rec,
    section: &mut String,
    memory_json: &mut Vec<serde_json::Value>,
) {
    for group in GROUPS {
        if !options.runs(group) {
            continue;
        }
        let _ = writeln!(section);
        let _ = writeln!(section, "### {group}");
        let _ = writeln!(section);
        let start = rec.rows.len();
        match group {
            "info" => memory_json.push(info::run(adapter, pair, rec, section)),
            "upload" => upload::run(rec, pair),
            "write_texture" => write_texture::run(rec, pair),
            "staging_belt" => staging_belt::run(rec, pair),
            "mappable_write" => mappable_write::run(rec, pair),
            "mappable_readwrite" => mappable_readwrite::run(rec, pair),
            "controls" => controls::run(rec, pair),
            #[cfg(not(target_arch = "wasm32"))]
            "bar_pressure" => bar_pressure::run(adapter, pair, rec, section),
            _ => {}
        }
        if group == "info" {
            // The section already describes everything but the checks.
            let checks: Vec<_> = rec.rows[start..]
                .iter()
                .filter(|r| r.metric == "check")
                .collect();
            if !checks.is_empty() {
                output::print_table(checks, section);
            }
        } else {
            output::print_table(&rec.rows[start..], section);
        }
    }
    let _ = writeln!(section);
}

/// Only the file name, so the output never records a directory.
fn file_name(path: &str) -> &str {
    std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
}
