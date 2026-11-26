//! `queue.write_buffer` into a device-only buffer.

use std::time::Instant;

use crate::common::{KIB, MIB, Pair, Rec, iterations, mb_per_s, ms_since, noise, size_name};

const BENCH: &str = "upload";

pub fn run(rec: &mut Rec, pair: &Pair) {
    let sizes = if pair.quick() {
        vec![MIB]
    } else {
        vec![4 * KIB, 64 * KIB, MIB, 16 * MIB, 64 * MIB]
    };
    for size in sizes {
        case(
            rec,
            pair,
            &format!("1 write of {}", size_name(size)),
            size,
            1,
        );
    }
    case(rec, pair, "256 writes of 4 KiB", 4 * KIB, 256);
}

struct State {
    dst: wgpu::Buffer,
    /// `MAP_WRITE | COPY_SRC` like wgpu-core's staging buffers, one per data
    /// set, for timing the same copies on the GPU.
    staging: [wgpu::Buffer; 2],
    copy_dst: wgpu::Buffer,
}

fn case(rec: &mut Rec, pair: &Pair, name: &str, write_size: u64, writes: u64) {
    println!("  {BENCH}: {name}");
    let total = write_size * writes;
    let data = [
        noise(total as usize, total),
        noise(total as usize, total + 1),
    ];
    let dst_usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
    let states: Vec<State> = pair
        .ctxs
        .iter()
        .map(|ctx| State {
            dst: ctx.buffer(total, dst_usage),
            staging: [0, 1].map(|d| {
                ctx.buffer_with(
                    &data[d],
                    wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                )
            }),
            copy_dst: ctx.buffer(total, dst_usage),
        })
        .collect();

    let iterations = iterations(pair.quick(), total);
    let mut run = crate::common::interleave(iterations, |v, i, sample| {
        let ctx = &pair.ctxs[v];
        let state = &states[v];
        let d = &data[i % 2];

        let start = Instant::now();
        for k in 0..writes {
            let range = (k * write_size) as usize..((k + 1) * write_size) as usize;
            ctx.queue
                .write_buffer(&state.dst, k * write_size, &d[range]);
        }
        ctx.queue.submit(None);
        let cpu = ms_since(start);
        ctx.wait();
        let done = ms_since(start);

        let mut encoder = ctx.encoder();
        ctx.mark(&mut encoder, 0);
        for k in 0..writes {
            encoder.copy_buffer_to_buffer(
                &state.staging[i % 2],
                k * write_size,
                &state.copy_dst,
                k * write_size,
                write_size,
            );
        }
        ctx.mark(&mut encoder, 1);
        ctx.resolve_timestamps(&mut encoder);
        let wall = ctx.submit_and_wait(encoder);
        let gpu = ctx.gpu_ms(wall);

        sample.add("cpu", "ms", cpu);
        sample.add("total", "ms", done);
        sample.add("throughput", "MB/s", mb_per_s(total, done));
        sample.add("gpu_copy", "ms", gpu);
        sample.add("gpu_copy_rate", "MB/s", mb_per_s(total, gpu));
        ctx.error_check()
    });
    let last = &data[iterations % 2];
    for (v, ctx) in pair.ctxs.iter().enumerate() {
        if run.results[v].is_ok() {
            let result = ctx
                .check_contents(&states[v].dst, last)
                .map_err(|e| format!("write_buffer destination: {e}"))
                .and_then(|()| {
                    ctx.check_contents(&states[v].copy_dst, last)
                        .map_err(|e| format!("copy destination: {e}"))
                });
            if let Err(e) = result {
                run.fail(v, e);
            }
        }
    }
    pair.collect_errors(&mut run);
    rec.finish(BENCH, name, run);
}
