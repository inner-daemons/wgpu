//! Tests that buffers with mappable usages stay usable by the GPU and the host.

use wgpu::{BufferUsages as Bu, MapMode};
use wgpu_test::{
    apply, gpu_test, GpuTestConfiguration, GpuTestInitializer, TestParameters, TestingContext,
};

pub fn all_tests(vec: &mut Vec<GpuTestInitializer>) {
    vec.push(MAPPABLE_BUFFER_PLACEMENT_ROUNDTRIP);
}

const ELEMENTS: usize = 16 * 1024;
const BUFFER_SIZE: u64 = (ELEMENTS * size_of::<u32>()) as u64;

const SHADER: &str = "
@group(0) @binding(0) var<storage, read_write> data: array<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    data[id.x] = data[id.x] * 3u + 1u;
}
";

#[derive(Clone, Copy)]
enum Upload {
    Map,
    WriteBuffer,
}

#[derive(Clone, Copy)]
enum Readback {
    Map,
    CopyToReadBuffer,
}

#[apply(gpu_test!)]
static MAPPABLE_BUFFER_PLACEMENT_ROUNDTRIP: GpuTestConfiguration = GpuTestConfiguration::new()
    .parameters(
        TestParameters::default()
            .features(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS)
            .limits(wgpu::Limits::downlevel_defaults())
            .enable_noop(),
    )
    .run_async(|ctx| async move {
        roundtrip(
            &ctx,
            Bu::MAP_READ | Bu::MAP_WRITE | Bu::STORAGE | Bu::COPY_DST,
            Upload::Map,
            Readback::Map,
        )
        .await;
        roundtrip(
            &ctx,
            Bu::MAP_WRITE | Bu::STORAGE | Bu::COPY_DST | Bu::COPY_SRC,
            Upload::Map,
            Readback::CopyToReadBuffer,
        )
        .await;
        roundtrip(
            &ctx,
            Bu::MAP_READ | Bu::STORAGE | Bu::COPY_DST,
            Upload::WriteBuffer,
            Readback::Map,
        )
        .await;
    });

async fn map(ctx: &TestingContext, buffer: &wgpu::Buffer, mode: MapMode) {
    buffer.slice(..).map_async(mode, Result::unwrap);
    ctx.async_poll(wgpu::PollType::wait_indefinitely())
        .await
        .unwrap();
}

async fn roundtrip(ctx: &TestingContext, usage: Bu, upload: Upload, readback: Readback) {
    let initial: Vec<u32> = (0..ELEMENTS as u32).collect();

    let buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("placement buffer"),
        size: BUFFER_SIZE,
        usage,
        mapped_at_creation: false,
    });

    match upload {
        Upload::Map => {
            map(ctx, &buffer, MapMode::Write).await;
            buffer
                .slice(..)
                .get_mapped_range_mut()
                .unwrap()
                .copy_from_slice(bytemuck::cast_slice(&initial));
            buffer.unmap();
        }
        Upload::WriteBuffer => ctx
            .queue
            .write_buffer(&buffer, 0, bytemuck::cast_slice(&initial)),
    }

    let module = ctx
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
    let pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });

    let read_buffer = matches!(readback, Readback::CopyToReadBuffer).then(|| {
        ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("placement readback"),
            size: BUFFER_SIZE,
            usage: Bu::MAP_READ | Bu::COPY_DST,
            mapped_at_creation: false,
        })
    });

    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups((ELEMENTS / 64) as u32, 1, 1);
    }
    if let Some(read_buffer) = &read_buffer {
        encoder.copy_buffer_to_buffer(&buffer, 0, read_buffer, 0, BUFFER_SIZE);
    }
    ctx.queue.submit([encoder.finish()]);

    let result = read_buffer.as_ref().unwrap_or(&buffer);
    map(ctx, result, MapMode::Read).await;
    {
        let view = result.slice(..).get_mapped_range().unwrap();
        let values: &[u32] = bytemuck::cast_slice(&view);
        assert_eq!(values.len(), ELEMENTS);
        for (i, &value) in values.iter().enumerate() {
            assert_eq!(value, i as u32 * 3 + 1, "usage {usage:?}, element {i}");
        }
    }
    result.unmap();
}
