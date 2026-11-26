//! `MAP_READ | MAP_WRITE | STORAGE` buffers that a shader writes and the CPU
//! reads, with `MAP_READ | STORAGE` (placement unchanged) as the control.

use std::time::Instant;

use crate::common::{
    KIB, MIB, Pair, Rec, gb_per_s, iterations, mb_per_s, ms_since, pattern, pattern_sum, size_name,
    sum_words,
};

const BENCH: &str = "mappable_readwrite";

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
        let rw = wgpu::BufferUsages::MAP_READ
            | wgpu::BufferUsages::MAP_WRITE
            | wgpu::BufferUsages::STORAGE;
        case(rec, pair, "map_read_write", rw, size);
        let r = wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::STORAGE;
        case(rec, pair, "map_read", r, size);
    }
}

fn case(rec: &mut Rec, pair: &Pair, usage_name: &str, usage: wgpu::BufferUsages, size: u64) {
    let name = format!("{usage_name} {}", size_name(size));
    println!("  {BENCH}: {name}");
    let words = (size / 4) as u32;
    let seeds = [0x1234_5678, 0x8765_4321];
    let expected = seeds.map(|seed| pattern_sum(words, seed));
    let states: Vec<_> = pair
        .ctxs
        .iter()
        .map(|ctx| {
            let buffer = ctx.buffer(size, usage);
            let bind_groups = seeds.map(|seed| ctx.fill_bind_group(&buffer, seed));
            (buffer, bind_groups)
        })
        .collect();

    let iterations = iterations(pair.quick(), size);
    let mut run = crate::common::interleave(iterations, |v, i, sample| {
        let ctx = &pair.ctxs[v];
        let (buffer, bind_groups) = &states[v];
        let (bind_group, workgroups) = &bind_groups[i % 2];
        let seed = seeds[i % 2];

        let mut encoder = ctx.encoder();
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: ctx.pass_timestamps(),
            });
            ctx.dispatch_fill(&mut pass, bind_group, *workgroups);
        }
        ctx.resolve_timestamps(&mut encoder);
        let wall = ctx.submit_and_wait(encoder);
        let gpu = ctx.gpu_ms(wall);

        ctx.map(buffer, wgpu::MapMode::Read);
        let (sum, first, last, cpu) = {
            let view = buffer.get_mapped_range(..).unwrap();
            let start = Instant::now();
            let sum = std::hint::black_box(sum_words(&view));
            let cpu = ms_since(start);
            let word = |w: usize| u32::from_le_bytes(view[w * 4..w * 4 + 4].try_into().unwrap());
            (sum, word(0), word(words as usize - 1), cpu)
        };
        buffer.unmap();

        sample.add("gpu_write", "ms", gpu);
        sample.add("gpu_write_rate", "GB/s", gb_per_s(size, gpu));
        sample.add("cpu_read", "ms", cpu);
        sample.add("cpu_read_rate", "MB/s", mb_per_s(size, cpu));
        ctx.error_check()?;
        if first != pattern(0, seed) || last != pattern(words - 1, seed) {
            Err(format!("iteration {i}: first or last word is wrong"))
        } else if sum != expected[i % 2] {
            Err(format!(
                "iteration {i}: sum {sum:#010x}, expected {:#010x}",
                expected[i % 2]
            ))
        } else {
            Ok(())
        }
    });
    pair.collect_errors(&mut run);
    rec.finish(BENCH, &name, run);
}
