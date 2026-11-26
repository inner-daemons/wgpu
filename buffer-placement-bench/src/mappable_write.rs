//! `MAP_WRITE | STORAGE` and `MAP_WRITE | UNIFORM` buffers used as per-frame
//! dynamic buffers: the CPU writes them, then a shader reads them.

use std::time::Instant;

use crate::common::{
    Ctx, KIB, MIB, Pair, Rec, Sink, gb_per_s, iterations, mb_per_s, ms_since, noise, size_name,
    sum_words,
};

const BENCH: &str = "mappable_write";
const UNIFORM_SIZE: u64 = 64 * KIB;
/// Workgroups of 64 invocations for the uniform read, each of which reads the
/// whole buffer.
const UNIFORM_WORKGROUPS: u32 = 64;

pub fn run(rec: &mut Rec, pair: &Pair) {
    if !pair.ctxs[0].mappable_primary_buffers() {
        println!("  {BENCH}: skipped, needs MAPPABLE_PRIMARY_BUFFERS");
        return;
    }
    let sizes = if pair.quick() {
        vec![MIB]
    } else {
        vec![64 * KIB, MIB, 16 * MIB]
    };
    let max = pair.ctxs[0].device.limits().max_storage_buffer_binding_size;
    for size in sizes.into_iter().filter(|&s| s <= max) {
        for reps in [1, 16] {
            case(rec, pair, Kind::Storage, size, reps);
        }
    }
    if pair.ctxs[0].device.limits().max_uniform_buffer_binding_size >= UNIFORM_SIZE {
        for reps in [1, 16] {
            case(rec, pair, Kind::Uniform, UNIFORM_SIZE, reps);
        }
    } else {
        println!("  {BENCH}: uniform skipped, max_uniform_buffer_binding_size is below 64 KiB");
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Storage,
    Uniform,
}

struct State {
    buffer: wgpu::Buffer,
    sink: Sink,
    bind_group: wgpu::BindGroup,
    workgroups: u32,
}

impl State {
    fn new(ctx: &Ctx, kind: Kind, size: u64) -> Self {
        let usage = match kind {
            Kind::Storage => wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::STORAGE,
            Kind::Uniform => wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::UNIFORM,
        };
        let buffer = ctx.buffer(size, usage);
        let sink = ctx.sink();
        let (bind_group, workgroups) = match kind {
            Kind::Storage => ctx.read_bind_group(&buffer, 0, size, &sink),
            Kind::Uniform => (ctx.uniform_bind_group(&buffer, &sink), UNIFORM_WORKGROUPS),
        };
        State {
            buffer,
            sink,
            bind_group,
            workgroups,
        }
    }
}

fn case(rec: &mut Rec, pair: &Pair, kind: Kind, size: u64, reps: u32) {
    let kind_name = match kind {
        Kind::Storage => "storage",
        Kind::Uniform => "uniform",
    };
    let plural = if reps == 1 { "" } else { "s" };
    let name = format!("{kind_name} {}, {reps} read{plural}", size_name(size));
    println!("  {BENCH}: {name}");
    let data = [
        noise(size as usize, size + 10),
        noise(size as usize, size + 11),
    ];
    // How many times each dispatch reads the buffer.
    let reads_per_dispatch = match kind {
        Kind::Storage => 1,
        Kind::Uniform => UNIFORM_WORKGROUPS * 64,
    };
    let expected = data
        .each_ref()
        .map(|d| sum_words(d).wrapping_mul(reads_per_dispatch * reps));
    let bytes_read = size * reads_per_dispatch as u64 * reps as u64;
    let states: Vec<State> = pair
        .ctxs
        .iter()
        .map(|ctx| State::new(ctx, kind, size))
        .collect();

    let iterations = iterations(pair.quick(), size);
    let mut run = crate::common::interleave(iterations, |v, i, sample| {
        let ctx = &pair.ctxs[v];
        let state = &states[v];

        let start = Instant::now();
        ctx.map(&state.buffer, wgpu::MapMode::Write);
        let mapped = Instant::now();
        state
            .buffer
            .get_mapped_range_mut(..)
            .unwrap()
            .copy_from_slice(&data[i % 2]);
        state.buffer.unmap();
        let cpu = ms_since(start);
        let write = ms_since(mapped);

        let mut encoder = ctx.encoder();
        state.sink.clear(&mut encoder);
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: ctx.pass_timestamps(),
            });
            match kind {
                Kind::Storage => {
                    ctx.dispatch_read(&mut pass, &state.bind_group, state.workgroups, reps)
                }
                Kind::Uniform => {
                    ctx.dispatch_uniform_read(&mut pass, &state.bind_group, state.workgroups, reps)
                }
            }
        }
        state.sink.copy_out(&mut encoder);
        ctx.resolve_timestamps(&mut encoder);
        let wall = ctx.submit_and_wait(encoder);
        let gpu = ctx.gpu_ms(wall);
        let got = state.sink.read(ctx);

        sample.add("cpu_map_write", "ms", cpu);
        sample.add("cpu_write_rate", "MB/s", mb_per_s(size, write));
        sample.add("gpu_read", "ms", gpu);
        sample.add("gpu_read_rate", "GB/s", gb_per_s(bytes_read, gpu));
        ctx.error_check()?;
        if got == expected[i % 2] {
            Ok(())
        } else {
            Err(format!(
                "iteration {i}: checksum {got:#010x}, expected {:#010x}",
                expected[i % 2]
            ))
        }
    });
    pair.collect_errors(&mut run);
    rec.finish(BENCH, &name, run);
}
