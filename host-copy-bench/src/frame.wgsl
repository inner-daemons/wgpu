struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOut {
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    var out: VertexOut;
    out.position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    var acc = vec4<f32>(0.0);
    for (var i = 0; i < 16; i++) {
        acc += textureSample(tex, samp, in.uv + vec2<f32>(f32(i) * 0.003, 0.0));
    }
    return acc / 16.0;
}
