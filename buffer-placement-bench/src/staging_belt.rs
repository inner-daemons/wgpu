//! `wgpu::util::StagingBelt` writing 1 MiB chunks into a device-only buffer.

use std::time::Instant;

use crate::common::{MIB, Pair, Rec, iterations, mb_per_s, ms_since, noise, size_name};

const BENCH: &str = "staging_belt";
const CHUNK: u64 = MIB;

pub fn run(rec: &mut Rec, pair: &Pair) {
    let frame = 64 * MIB;
    let name = format!("{} per frame in 1 MiB chunks", size_name(frame));
    println!("  {BENCH}: {name}");
    let data = [noise(frame as usize, 5), noise(frame as usize, 6)];
    let mut states: Vec<(wgpu::util::StagingBelt, wgpu::Buffer)> = pair
        .ctxs
        .iter()
        .map(|ctx| {
            (
                wgpu::util::StagingBelt::new(ctx.device.clone(), CHUNK),
                ctx.buffer(
                    frame,
                    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                ),
            )
        })
        .collect();

    let iterations = iterations(pair.quick(), frame);
    let mut run = crate::common::interleave(iterations, |v, i, sample| {
        let ctx = &pair.ctxs[v];
        let (belt, dst) = &mut states[v];
        let d = &data[i % 2];

        let start = Instant::now();
        let mut encoder = ctx.encoder();
        ctx.mark(&mut encoder, 0);
        for k in 0..frame / CHUNK {
            let range = (k * CHUNK) as usize..((k + 1) * CHUNK) as usize;
            belt.write_buffer(
                &mut encoder,
                dst,
                k * CHUNK,
                wgpu::BufferSize::new(CHUNK).unwrap(),
            )
            .copy_from_slice(&d[range]);
        }
        ctx.mark(&mut encoder, 1);
        ctx.resolve_timestamps(&mut encoder);
        belt.finish();
        let submitted = Instant::now();
        ctx.queue.submit(Some(encoder.finish()));
        let cpu = ms_since(start);
        ctx.wait();
        let done = ms_since(start);
        let gpu = ctx.gpu_ms(ms_since(submitted));
        belt.recall();
        ctx.wait();

        sample.add("cpu", "ms", cpu);
        sample.add("total", "ms", done);
        sample.add("throughput", "MB/s", mb_per_s(frame, done));
        sample.add("gpu_copy", "ms", gpu);
        sample.add("gpu_copy_rate", "MB/s", mb_per_s(frame, gpu));
        ctx.error_check()
    });
    let last = &data[iterations % 2];
    for (v, ctx) in pair.ctxs.iter().enumerate() {
        if run.results[v].is_ok()
            && let Err(e) = ctx.check_contents(&states[v].1, last)
        {
            run.fail(v, e);
        }
    }
    pair.collect_errors(&mut run);
    rec.finish(BENCH, &name, run);
}
