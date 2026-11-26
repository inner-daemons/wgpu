//! Buffers whose placement didn't change, so `new / legacy` should be about 1.
//! They show the noise floor.

use std::time::Instant;

use crate::common::{
    MIB, Pair, Rec, gb_per_s, iterations, mb_per_s, ms_since, pattern_sum, size_name, sum_words,
};

const BENCH: &str = "controls";
const SIZE: u64 = 16 * MIB;
const SEED: u32 = 0x0bad_cafe;

pub fn run(rec: &mut Rec, pair: &Pair) {
    readback(rec, pair);
    device_read(rec, pair);
}

/// A `MAP_READ | COPY_DST` buffer: copy into it on the GPU, then map and sum it.
fn readback(rec: &mut Rec, pair: &Pair) {
    let name = format!("readback {}", size_name(SIZE));
    println!("  {BENCH}: {name}");
    let expected = pattern_sum((SIZE / 4) as u32, SEED);
    let states: Vec<_> = pair
        .ctxs
        .iter()
        .map(|ctx| {
            let src = ctx.buffer(
                SIZE,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            );
            ctx.fill(&src, SEED);
            let readback = ctx.buffer(
                SIZE,
                wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            );
            (src, readback)
        })
        .collect();

    let mut run = crate::common::interleave(iterations(pair.quick(), SIZE), |v, i, sample| {
        let ctx = &pair.ctxs[v];
        let (src, readback) = &states[v];
        let mut encoder = ctx.encoder();
        ctx.mark(&mut encoder, 0);
        encoder.copy_buffer_to_buffer(src, 0, readback, 0, SIZE);
        ctx.mark(&mut encoder, 1);
        ctx.resolve_timestamps(&mut encoder);
        let wall = ctx.submit_and_wait(encoder);
        let gpu = ctx.gpu_ms(wall);

        ctx.map(readback, wgpu::MapMode::Read);
        let (sum, cpu) = {
            let view = readback.get_mapped_range(..).unwrap();
            let start = Instant::now();
            let sum = std::hint::black_box(sum_words(&view));
            (sum, ms_since(start))
        };
        readback.unmap();

        sample.add("copy_wait", "ms", wall);
        sample.add("gpu_copy", "ms", gpu);
        sample.add("cpu_read", "ms", cpu);
        sample.add("cpu_read_rate", "MB/s", mb_per_s(SIZE, cpu));
        ctx.error_check()?;
        if sum == expected {
            Ok(())
        } else {
            Err(format!(
                "iteration {i}: sum {sum:#010x}, expected {expected:#010x}"
            ))
        }
    });
    pair.collect_errors(&mut run);
    rec.finish(BENCH, &name, run);
}

/// A device-only `STORAGE` buffer read by a shader.
fn device_read(rec: &mut Rec, pair: &Pair) {
    let reps = 16;
    let name = format!("device storage {}, {reps} reads", size_name(SIZE));
    println!("  {BENCH}: {name}");
    let expected = pattern_sum((SIZE / 4) as u32, SEED).wrapping_mul(reps);
    let states: Vec<_> = pair
        .ctxs
        .iter()
        .map(|ctx| {
            let buffer = ctx.buffer(SIZE, wgpu::BufferUsages::STORAGE);
            ctx.fill(&buffer, SEED);
            let sink = ctx.sink();
            let (bind_group, workgroups) = ctx.read_bind_group(&buffer, 0, SIZE, &sink);
            (buffer, sink, bind_group, workgroups)
        })
        .collect();

    let mut run = crate::common::interleave(iterations(pair.quick(), SIZE), |v, i, sample| {
        let ctx = &pair.ctxs[v];
        let (_, sink, bind_group, workgroups) = &states[v];
        let mut encoder = ctx.encoder();
        sink.clear(&mut encoder);
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: ctx.pass_timestamps(),
            });
            ctx.dispatch_read(&mut pass, bind_group, *workgroups, reps);
        }
        sink.copy_out(&mut encoder);
        ctx.resolve_timestamps(&mut encoder);
        let wall = ctx.submit_and_wait(encoder);
        let gpu = ctx.gpu_ms(wall);
        let got = sink.read(ctx);

        sample.add("gpu_read", "ms", gpu);
        sample.add("gpu_read_rate", "GB/s", gb_per_s(SIZE * reps as u64, gpu));
        ctx.error_check()?;
        if got == expected {
            Ok(())
        } else {
            Err(format!(
                "iteration {i}: checksum {got:#010x}, expected {expected:#010x}"
            ))
        }
    });
    pair.collect_errors(&mut run);
    rec.finish(BENCH, &name, run);
}
