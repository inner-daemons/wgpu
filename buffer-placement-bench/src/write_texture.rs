//! `queue.write_texture`, which goes through wgpu-core's internal staging
//! buffers.

use std::time::Instant;

use crate::common::{Ctx, Pair, Rec, iterations, mb_per_s, ms_since, noise};

const BENCH: &str = "write_texture";

pub fn run(rec: &mut Rec, pair: &Pair) {
    let max = pair.ctxs[0].device.limits().max_texture_dimension_2d;
    let sizes: Vec<u32> = if pair.quick() {
        vec![1024]
    } else {
        vec![256, 1024, 4096]
    };
    for size in sizes.into_iter().filter(|&s| s <= max) {
        case(rec, pair, size);
    }
}

struct State {
    texture: wgpu::Texture,
    staging: [wgpu::Buffer; 2],
    copy_dst: wgpu::Texture,
}

fn texture(ctx: &Ctx, size: u32) -> wgpu::Texture {
    ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: extent(size),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
        texture_binding_view_dimension: None,
    })
}

fn extent(size: u32) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: 1,
    }
}

fn layout(size: u32) -> wgpu::TexelCopyBufferLayout {
    wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(size * 4),
        rows_per_image: Some(size),
    }
}

fn check(ctx: &Ctx, texture: &wgpu::Texture, size: u32, expected: &[u8]) -> Result<(), String> {
    let readback = ctx.buffer(
        expected.len() as u64,
        wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    );
    let mut encoder = ctx.encoder();
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: layout(size),
        },
        extent(size),
    );
    ctx.submit_and_wait(encoder);
    ctx.map(&readback, wgpu::MapMode::Read);
    let view = readback.get_mapped_range(..).unwrap();
    match view.iter().zip(expected).position(|(a, b)| a != b) {
        None => Ok(()),
        Some(i) => Err(format!("texture byte {i} differs")),
    }
}

fn case(rec: &mut Rec, pair: &Pair, size: u32) {
    let name = format!("Rgba8Unorm {size}x{size}");
    println!("  {BENCH}: {name}");
    let bytes = size as u64 * size as u64 * 4;
    let data = [
        noise(bytes as usize, bytes),
        noise(bytes as usize, bytes + 1),
    ];
    let states: Vec<State> = pair
        .ctxs
        .iter()
        .map(|ctx| State {
            texture: texture(ctx, size),
            staging: [0, 1].map(|d| {
                ctx.buffer_with(
                    &data[d],
                    wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                )
            }),
            copy_dst: texture(ctx, size),
        })
        .collect();

    let iterations = iterations(pair.quick(), bytes);
    let mut run = crate::common::interleave(iterations, |v, i, sample| {
        let ctx = &pair.ctxs[v];
        let state = &states[v];

        let start = Instant::now();
        ctx.queue.write_texture(
            state.texture.as_image_copy(),
            &data[i % 2],
            layout(size),
            extent(size),
        );
        ctx.queue.submit(None);
        let cpu = ms_since(start);
        ctx.wait();
        let done = ms_since(start);

        let mut encoder = ctx.encoder();
        ctx.mark(&mut encoder, 0);
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &state.staging[i % 2],
                layout: layout(size),
            },
            state.copy_dst.as_image_copy(),
            extent(size),
        );
        ctx.mark(&mut encoder, 1);
        ctx.resolve_timestamps(&mut encoder);
        let wall = ctx.submit_and_wait(encoder);
        let gpu = ctx.gpu_ms(wall);

        sample.add("cpu", "ms", cpu);
        sample.add("total", "ms", done);
        sample.add("throughput", "MB/s", mb_per_s(bytes, done));
        sample.add("gpu_copy", "ms", gpu);
        sample.add("gpu_copy_rate", "MB/s", mb_per_s(bytes, gpu));
        ctx.error_check()
    });
    let last = &data[iterations % 2];
    for (v, ctx) in pair.ctxs.iter().enumerate() {
        if run.results[v].is_ok() {
            let result = check(ctx, &states[v].texture, size, last)
                .map_err(|e| format!("write_texture destination: {e}"))
                .and_then(|()| {
                    check(ctx, &states[v].copy_dst, size, last)
                        .map_err(|e| format!("copy destination: {e}"))
                });
            if let Err(e) = result {
                run.fail(v, e);
            }
        }
    }
    pair.collect_errors(&mut run);
    rec.finish(BENCH, &name, run);
}
