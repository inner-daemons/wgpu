//! Benchmarks that model how applications use uploads and readbacks, rather
//! than single copies into idle resources.

use std::collections::VecDeque;

use super::*;

const RGBA8: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn host_visible(ctx: &Ctx, format: wgpu::TextureFormat) -> bool {
    ctx.device
        .features()
        .contains(wgpu::Features::HOST_TEXTURE_COPY)
        && ctx
            .adapter
            .get_texture_format_features(format)
            .allowed_usages
            .contains(wgpu::TextureUsages::HOST_VISIBLE)
}

fn create_texture(
    ctx: &Ctx,
    size: wgpu::Extent3d,
    mip_level_count: u32,
    usage: wgpu::TextureUsages,
    mapped_at_creation: bool,
) -> wgpu::Texture {
    ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size,
        mip_level_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: RGBA8,
        usage,
        view_formats: &[],
        texture_binding_view_dimension: None,
        mapped_at_creation,
    })
}

fn square(size: u32) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: 1,
    }
}

fn rgba8_layout(width: u32, height: u32) -> wgpu::TexelCopyBufferLayout {
    wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(width * 4),
        rows_per_image: Some(height),
    }
}

fn host_range(mip_level: u32, origin: wgpu::Origin3d) -> wgpu::TexelCopyTextureInfoBase<()> {
    wgpu::TexelCopyTextureInfoBase {
        texture: (),
        mip_level,
        origin,
        aspect: wgpu::TextureAspect::All,
    }
}

fn staging_usage() -> wgpu::TextureUsages {
    wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING
}

fn host_usage() -> wgpu::TextureUsages {
    wgpu::TextureUsages::HOST_VISIBLE | wgpu::TextureUsages::TEXTURE_BINDING
}

/// Like `Ctx::time_compute`, for work that writes timestamps 0 and 1 itself.
fn time_gpu(
    ctx: &Ctx,
    record: impl FnOnce(&mut wgpu::CommandEncoder, Option<&wgpu::QuerySet>),
) -> f64 {
    let mut encoder = ctx.device.create_command_encoder(&Default::default());
    let queries = ctx.timestamps.then(|| {
        let set = ctx.device.create_query_set(&wgpu::QuerySetDescriptor {
            label: None,
            ty: wgpu::QueryType::Timestamp,
            count: 2,
        });
        let resolve = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 16,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 16,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        (set, resolve, readback)
    });
    record(&mut encoder, queries.as_ref().map(|(set, ..)| set));
    if let Some((set, resolve, readback)) = &queries {
        encoder.resolve_query_set(set, 0..2, resolve, 0);
        encoder.copy_buffer_to_buffer(resolve, 0, readback, 0, 16);
    }
    let start = Instant::now();
    ctx.queue.submit(Some(encoder.finish()));
    ctx.wait();
    let wall = ms_since(start);
    let Some((_, _, readback)) = queries else {
        return wall;
    };
    map_buffer(ctx, &readback, wgpu::MapMode::Read);
    let ticks: [u64; 2] = {
        let view = readback.get_mapped_range(..).unwrap();
        bytemuck::pod_read_unaligned(&view[..16])
    };
    readback.unmap();
    let gpu =
        ticks[1].wrapping_sub(ticks[0]) as f64 * ctx.queue.get_timestamp_period() as f64 / 1e6;
    plausible_gpu_time(gpu, wall)
}

/// Submits one cheap pass that samples `texture`, without waiting for it.
fn use_texture(
    ctx: &Ctx,
    sampler: &Sampler,
    texture: &wgpu::Texture,
) -> Option<wgpu::SubmissionIndex> {
    let size = texture.size();
    let params = ctx.uniform([size.width, size.height, 1, 0]);
    let view = texture.create_view(&Default::default());
    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &sampler.pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: sampler.sink.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: params.as_entire_binding(),
            },
        ],
    });
    let mut encoder = ctx.device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&sampler.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(size.width.div_ceil(8), size.height.div_ceil(8), 1);
    }
    ctx.queue.submit(Some(encoder.finish()))
}

fn wait_for(ctx: &Ctx, index: Option<wgpu::SubmissionIndex>) {
    ctx.device
        .poll(wgpu::PollType::Wait {
            submission_index: index,
            timeout: None,
        })
        .unwrap();
}

fn frame_module(ctx: &Ctx) -> wgpu::ShaderModule {
    ctx.device
        .create_shader_module(wgpu::include_wgsl!("frame.wgsl"))
}

fn render_pipeline(ctx: &Ctx, module: &wgpu::ShaderModule, fragment: &str) -> wgpu::RenderPipeline {
    ctx.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(fragment),
            layout: None,
            vertex: wgpu::VertexState {
                module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some(fragment),
                compilation_options: Default::default(),
                targets: &[Some(RGBA8.into())],
            }),
            multiview_mask: None,
            cache: None,
        })
}

fn color_attachment(view: &wgpu::TextureView) -> wgpu::RenderPassColorAttachment<'_> {
    wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
        },
    }
}

/// Rendering into a `HOST_VISIBLE` texture vs a normal one, and sampling the
/// result. Host copy usage can turn off framebuffer compression.
pub(crate) fn render_targets(ctx: &mut Ctx) {
    let size = if ctx.quick { 1024 } else { 2048 };
    let passes = 16;
    let module = frame_module(ctx);
    let pipeline = render_pipeline(ctx, &module, "fs_fill");
    let sampler = Sampler::new(ctx);
    warm_up(ctx, &sampler);
    let mut variants = vec![(
        "normal",
        wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST,
    )];
    if host_visible(ctx, RGBA8) {
        variants.push((
            "host",
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::HOST_VISIBLE,
        ));
    }
    let case = format!("Rgba8Unorm {size}x{size}, {passes} passes");
    println!("  render targets: {case}");
    let textures: Vec<wgpu::Texture> = variants
        .iter()
        .map(|&(_, usage)| create_texture(ctx, square(size), 1, usage, false))
        .collect();
    let render = |ctx: &Ctx, texture: &wgpu::Texture| {
        let view = texture.create_view(&Default::default());
        time_gpu(ctx, |encoder, queries| {
            for i in 0..passes {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(color_attachment(&view))],
                    timestamp_writes: queries.filter(|_| i == 0 || i == passes - 1).map(
                        |query_set| wgpu::RenderPassTimestampWrites {
                            query_set,
                            beginning_of_pass_write_index: (i == 0).then_some(0),
                            end_of_pass_write_index: (i == passes - 1).then_some(1),
                        },
                    ),
                    ..Default::default()
                });
                pass.set_pipeline(&pipeline);
                pass.draw(0..3, 0..1);
            }
        })
    };
    let reps = sample_reps(size);
    let mut render_times = vec![Vec::new(); textures.len()];
    let mut sample_times = vec![Vec::new(); textures.len()];
    for texture in &textures {
        render(ctx, texture);
    }
    // Alternate between the variants so clock changes affect both equally.
    for _ in 0..8 {
        for (i, texture) in textures.iter().enumerate() {
            render_times[i].push(render(ctx, texture));
            sample_times[i].push(sampler.run(ctx, texture, reps));
        }
    }
    if ctx.took_error() {
        eprintln!("  render targets: skipped after an error");
        return;
    }
    let pixels = (size as f64) * (size as f64);
    for (i, &(variant, _)) in variants.iter().enumerate() {
        let render_ms = median(render_times[i].clone());
        let sample_ms = median(sample_times[i].clone());
        ctx.record("render_target", &case, variant, "render", render_ms, "ms");
        ctx.record(
            "render_target",
            &case,
            variant,
            "render_rate",
            pixels * passes as f64 / (render_ms * 1e6),
            "Gpixel/s",
        );
        ctx.record(
            "render_target",
            &case,
            variant,
            "sample_rate",
            pixels * reps as f64 / (sample_ms * 1e6),
            "Gtexel/s",
        );
        if i > 0 {
            ctx.record(
                "render_target",
                &case,
                variant,
                "render_vs_normal",
                paired_ratio(&render_times[i], &render_times[0]),
                "x",
            );
            ctx.record(
                "render_target",
                &case,
                variant,
                "sample_vs_normal",
                paired_ratio(&sample_times[i], &sample_times[0]),
                "x",
            );
        }
    }
}

/// Many small writes into a large atlas that the GPU samples after every batch,
/// like glyph or sprite atlases. With host copies, every batch has to wait for
/// the GPU to finish with the atlas before it can be mapped.
pub(crate) fn atlas_updates(ctx: &mut Ctx) {
    let size = if ctx.quick { 2048 } else { 4096 };
    let tile = 32u32;
    let count = if ctx.quick { 300 } else { 1000 };
    let iterations = if ctx.quick { 10 } else { 30 };
    let data = noise((tile * tile * 4) as usize, 11);
    let layout = rgba8_layout(tile, tile);
    let tiles_per_row = size / tile;
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    let origins: Vec<wgpu::Origin3d> = (0..count)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let index = (state % (tiles_per_row * tiles_per_row) as u64) as u32;
            wgpu::Origin3d {
                x: index % tiles_per_row * tile,
                y: index / tiles_per_row * tile,
                z: 0,
            }
        })
        .collect();
    let sampler = Sampler::new(ctx);
    let case = format!("{count} {tile}x{tile} writes into {size}x{size}");
    println!("  atlas: {case}");
    let mut variants = vec![TextureVariant::Staging];
    if host_visible(ctx, RGBA8) {
        variants.push(TextureVariant::Host);
    }
    for variant in variants {
        let usage = match variant {
            TextureVariant::Staging => staging_usage(),
            _ => host_usage(),
        };
        let texture = create_texture(ctx, square(size), 1, usage, false);
        let batch = |ctx: &Ctx| {
            match variant {
                TextureVariant::Staging => {
                    for &origin in &origins {
                        ctx.queue.write_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: &texture,
                                mip_level: 0,
                                origin,
                                aspect: wgpu::TextureAspect::All,
                            },
                            &data,
                            layout,
                            square(tile),
                        );
                    }
                }
                _ => {
                    map_texture(ctx, &texture, wgpu::MapMode::Write);
                    let mapped = texture.get_mapped_texture_mut().unwrap();
                    for &origin in &origins {
                        mapped.copy_from_memory(host_range(0, origin), &data, layout, square(tile));
                    }
                    drop(mapped);
                    texture.unmap();
                }
            }
            use_texture(ctx, &sampler, &texture)
        };
        batch(ctx);
        ctx.wait();
        let mut in_flight = VecDeque::new();
        let start = Instant::now();
        for _ in 0..iterations {
            in_flight.push_back(batch(ctx));
            if in_flight.len() > 2 {
                wait_for(ctx, in_flight.pop_front().flatten());
            }
        }
        ctx.wait();
        let ms = ms_since(start) / iterations as f64;
        if ctx.took_error() {
            eprintln!("  atlas {}: skipped after an error", variant.name());
            continue;
        }
        ctx.record("atlas", &case, variant.name(), "batch", ms, "ms");
    }
}

/// Uploading a full mip chain.
pub(crate) fn mip_chains(ctx: &mut Ctx) {
    let size: u32 = if ctx.quick { 1024 } else { 2048 };
    let mips = size.ilog2() + 1;
    let iterations = if ctx.quick { 3 } else { 5 };
    let levels: Vec<(u32, Vec<u8>)> = (0..mips)
        .map(|m| {
            let s = size >> m;
            (s, noise((s * s * 4) as usize, m as u64))
        })
        .collect();
    let case = format!("Rgba8Unorm {size}x{size}, {mips} mips");
    println!("  mip chains: {case}");
    let mut variants = vec![TextureVariant::Staging];
    if host_visible(ctx, RGBA8) {
        variants.push(TextureVariant::Host);
    }
    for variant in variants {
        let mut times = Vec::new();
        for _ in 0..iterations {
            let start = Instant::now();
            match variant {
                TextureVariant::Staging => {
                    let texture = create_texture(ctx, square(size), mips, staging_usage(), false);
                    for (m, (s, data)) in levels.iter().enumerate() {
                        ctx.queue.write_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: &texture,
                                mip_level: m as u32,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            data,
                            rgba8_layout(*s, *s),
                            square(*s),
                        );
                    }
                    ctx.submit_empty_and_wait();
                }
                _ => {
                    let texture = create_texture(ctx, square(size), mips, host_usage(), true);
                    let mapped = texture.get_mapped_texture_mut().unwrap();
                    for (m, (s, data)) in levels.iter().enumerate() {
                        mapped.copy_from_memory(
                            host_range(m as u32, wgpu::Origin3d::ZERO),
                            data,
                            rgba8_layout(*s, *s),
                            square(*s),
                        );
                    }
                    drop(mapped);
                    texture.unmap();
                }
            }
            times.push(ms_since(start));
        }
        if ctx.took_error() {
            eprintln!("  mip chains {}: skipped after an error", variant.name());
            continue;
        }
        ctx.record(
            "mips",
            &case,
            variant.name(),
            "upload_fresh",
            median(times),
            "ms",
        );
    }
}

/// Several threads uploading their own textures at once.
pub(crate) fn threaded_uploads(ctx: &mut Ctx) {
    let size = 1024;
    let per_thread = if ctx.quick { 4 } else { 8 };
    let thread_counts: &[usize] = if ctx.quick { &[1, 4] } else { &[1, 2, 4, 8] };
    let data = noise((size * size * 4) as usize, 3);
    let layout = rgba8_layout(size, size);
    let mut variants = vec![TextureVariant::Staging];
    if host_visible(ctx, RGBA8) {
        variants.push(TextureVariant::Host);
    }
    println!("  threaded uploads");
    for variant in variants {
        for &threads in thread_counts {
            let run = |ctx: &Ctx| {
                let start = Instant::now();
                std::thread::scope(|s| {
                    for _ in 0..threads {
                        s.spawn(|| {
                            for _ in 0..per_thread {
                                match variant {
                                    TextureVariant::Staging => {
                                        let texture = create_texture(
                                            ctx,
                                            square(size),
                                            1,
                                            staging_usage(),
                                            false,
                                        );
                                        ctx.queue.write_texture(
                                            texture.as_image_copy(),
                                            &data,
                                            layout,
                                            square(size),
                                        );
                                    }
                                    _ => {
                                        let texture = create_texture(
                                            ctx,
                                            square(size),
                                            1,
                                            host_usage(),
                                            true,
                                        );
                                        texture.get_mapped_texture_mut().unwrap().copy_from_memory(
                                            host_range(0, wgpu::Origin3d::ZERO),
                                            &data,
                                            layout,
                                            square(size),
                                        );
                                        texture.unmap();
                                    }
                                }
                            }
                        });
                    }
                });
                ctx.submit_empty_and_wait();
                ms_since(start)
            };
            run(ctx);
            let ms = median((0..3).map(|_| run(ctx)).collect());
            if ctx.took_error() {
                eprintln!(
                    "  threaded uploads {}: skipped after an error",
                    variant.name()
                );
                continue;
            }
            let bytes = (threads * per_thread) as f64 * data.len() as f64;
            let case = format!("{threads} threads x {per_thread} Rgba8Unorm {size}x{size}");
            ctx.record("threads", &case, variant.name(), "total", ms, "ms");
            ctx.record(
                "threads",
                &case,
                variant.name(),
                "rate",
                bytes / (ms * 1e6),
                "GB/s",
            );
        }
    }
}

/// Frame times of a GPU-heavy render loop that uploads a new texture every
/// frame and samples it, against the same loop without uploads.
pub(crate) fn busy_uploads(ctx: &mut Ctx) {
    let frames = if ctx.quick { 60 } else { 240 };
    let warmup = 10;
    let upload_size = 1024;
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: 2560,
            height: 1440,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: RGBA8,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
        texture_binding_view_dimension: None,
        mapped_at_creation: false,
    });
    let target_view = target.create_view(&Default::default());
    let module = frame_module(ctx);
    let pipeline = render_pipeline(ctx, &module, "fs_main");
    let filter = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let data = noise((upload_size * upload_size * 4) as usize, 5);
    let layout = rgba8_layout(upload_size, upload_size);
    let fixed = TextureCase::new(RGBA8, upload_size, Content::Noise)
        .upload_fresh(ctx, TextureVariant::Staging)
        .0;
    let busy = BusyFrame {
        pipeline,
        filter,
        target_view,
    };
    // Enough passes that a frame without uploads takes about 12 ms, so the loop is
    // limited by the GPU on any device.
    let calibrate = |ctx: &Ctx, passes: u32| {
        busy.submit(ctx, &fixed, passes);
        ctx.wait();
        let start = Instant::now();
        for _ in 0..5 {
            busy.submit(ctx, &fixed, passes);
        }
        ctx.wait();
        ms_since(start) / 5.0
    };
    let probe = 6;
    let probe_ms = calibrate(ctx, probe);
    let passes = ((probe as f64 * 12.0 / probe_ms.max(0.01)).round() as u32).clamp(probe, 400);
    let case = format!("2560x1440, one {upload_size}x{upload_size} upload per frame");
    println!("  busy uploads: {case}");
    let mut variants = vec![
        ("no uploads", None),
        ("staging", Some(TextureVariant::Staging)),
    ];
    if host_visible(ctx, RGBA8) {
        variants.push(("host", Some(TextureVariant::Host)));
    }
    let frame = |ctx: &Ctx, variant: Option<TextureVariant>| {
        let source = match variant {
            None => fixed.clone(),
            Some(TextureVariant::Staging) => {
                let texture = create_texture(ctx, square(upload_size), 1, staging_usage(), false);
                ctx.queue.write_texture(
                    texture.as_image_copy(),
                    &data,
                    layout,
                    square(upload_size),
                );
                texture
            }
            Some(_) => {
                let texture = create_texture(ctx, square(upload_size), 1, host_usage(), true);
                texture.get_mapped_texture_mut().unwrap().copy_from_memory(
                    host_range(0, wgpu::Origin3d::ZERO),
                    &data,
                    layout,
                    square(upload_size),
                );
                texture.unmap();
                texture
            }
        };
        busy.submit(ctx, &source, passes)
    };
    // Runs `count` frames with two in flight and returns each frame's time.
    let run = |ctx: &Ctx, variant: Option<TextureVariant>, count: u32| {
        let mut in_flight = VecDeque::new();
        let mut times = Vec::new();
        for _ in 0..count {
            let start = Instant::now();
            in_flight.push_back(frame(ctx, variant));
            if in_flight.len() > 2 {
                wait_for(ctx, in_flight.pop_front().flatten());
            }
            times.push(ms_since(start));
        }
        ctx.wait();
        times
    };
    for &(_, variant) in &variants {
        run(ctx, variant, warmup);
    }
    // Alternate between the variants in chunks so clock changes affect all of
    // them equally.
    let chunk = 20;
    let mut times = vec![Vec::new(); variants.len()];
    let mut chunk_means = vec![Vec::new(); variants.len()];
    for _ in 0..frames / chunk {
        for (i, &(_, variant)) in variants.iter().enumerate() {
            let chunk_times = run(ctx, variant, chunk);
            chunk_means[i].push(chunk_times.iter().sum::<f64>() / chunk as f64);
            times[i].extend(chunk_times);
        }
    }
    if ctx.took_error() {
        eprintln!("  busy uploads: skipped after an error");
        return;
    }
    for (i, &(name, _)) in variants.iter().enumerate() {
        let times = &mut times[i];
        let mean = times.iter().sum::<f64>() / times.len() as f64;
        times.sort_by(f64::total_cmp);
        let p95 = times[times.len() * 95 / 100];
        let max = *times.last().unwrap();
        ctx.record("busy", &case, name, "passes", passes as f64, "passes");
        ctx.record("busy", &case, name, "frame_mean", mean, "ms");
        ctx.record("busy", &case, name, "frame_p95", p95, "ms");
        ctx.record("busy", &case, name, "frame_max", max, "ms");
        if i > 0 {
            ctx.record(
                "busy",
                &case,
                name,
                "frame_vs_no_uploads",
                paired_ratio(&chunk_means[i], &chunk_means[0]),
                "x",
            );
        }
    }
}

struct BusyFrame {
    pipeline: wgpu::RenderPipeline,
    filter: wgpu::Sampler,
    target_view: wgpu::TextureView,
}

impl BusyFrame {
    /// Submits `passes` full-screen passes that sample `source`.
    fn submit(
        &self,
        ctx: &Ctx,
        source: &wgpu::Texture,
        passes: u32,
    ) -> Option<wgpu::SubmissionIndex> {
        let view = source.create_view(&Default::default());
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.filter),
                },
            ],
        });
        let mut encoder = ctx.device.create_command_encoder(&Default::default());
        for _ in 0..passes {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(color_attachment(&self.target_view))],
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        ctx.queue.submit(Some(encoder.finish()))
    }
}

/// Reading a buffer the GPU used back to the CPU: through a copy into a
/// `MAP_READ` buffer, or by mapping a `STORAGE | MAP_READ` buffer directly.
pub(crate) fn buffer_readback(ctx: &mut Ctx) {
    let sizes: &[u64] = if ctx.quick {
        &[16 * MB]
    } else {
        &[MB, 16 * MB, 64 * MB]
    };
    let iterations = if ctx.quick { 3 } else { 5 };
    let mappable = ctx
        .device
        .features()
        .contains(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS);
    for &size in sizes {
        if size > ctx.device.limits().max_buffer_size {
            continue;
        }
        let name = format!("{} MiB", size / MB);
        println!("  buffer readback: {name}");
        let data = noise(size as usize, size + 1);
        let mut variants = vec![(
            "gpu-only",
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        )];
        if mappable {
            variants.push((
                "mappable",
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
            ));
        }
        for (variant, usage) in variants {
            let buffer = ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: &data,
                    usage,
                });
            let mut times = Vec::new();
            for _ in 0..iterations {
                // Let the GPU touch the buffer first, as it would have written it.
                let encoder = ctx.device.create_command_encoder(&Default::default());
                ctx.queue.submit(Some(encoder.finish()));
                ctx.wait();
                let start = Instant::now();
                let mut out = vec![0u8; size as usize];
                if variant == "gpu-only" {
                    let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
                        label: None,
                        size,
                        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    let mut encoder = ctx.device.create_command_encoder(&Default::default());
                    encoder.copy_buffer_to_buffer(&buffer, 0, &staging, 0, size);
                    ctx.queue.submit(Some(encoder.finish()));
                    map_buffer(ctx, &staging, wgpu::MapMode::Read);
                    out.copy_from_slice(&staging.get_mapped_range(..).unwrap());
                    staging.unmap();
                } else {
                    map_buffer(ctx, &buffer, wgpu::MapMode::Read);
                    out.copy_from_slice(&buffer.get_mapped_range(..).unwrap());
                    buffer.unmap();
                }
                std::hint::black_box(&out);
                times.push(ms_since(start));
            }
            if ctx.took_error() {
                eprintln!("  buffer readback {variant}: skipped after an error");
                continue;
            }
            let ms = median(times);
            ctx.record("buffer_readback", &name, variant, "readback", ms, "ms");
            ctx.record(
                "buffer_readback",
                &name,
                variant,
                "rate",
                size as f64 / (ms * 1e6),
                "GB/s",
            );
        }
    }
}
