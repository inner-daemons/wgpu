struct Params {
    width: u32,
    height: u32,
    reps: u32,
    count: u32,
}

@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> sink: array<vec4<f32>>;
@group(0) @binding(2) var<uniform> params: Params;

// Reads `reps` texels per invocation, offset so a workgroup's reads stay
// spatially coherent but cover the whole texture. Sizes are powers of two, so
// wrapping is a mask, keeping the shader limited by texture reads.
@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= params.width || id.y >= params.height {
        return;
    }
    var acc = vec4<f32>(0.0);
    for (var r = 0u; r < params.reps; r++) {
        let p = vec2<u32>((id.x + r * 61u) & (params.width - 1u), (id.y + r * 37u) & (params.height - 1u));
        acc += textureLoad(tex, p, 0);
    }
    if all(acc == vec4<f32>(-1.0)) {
        sink[0] = acc;
    }
}
