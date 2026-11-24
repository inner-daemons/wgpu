@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> first_block: array<vec4<f32>, 16>;
@group(0) @binding(2) var<storage, read_write> error_texels: atomic<u32>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(tex);
    if id.x >= size.x || id.y >= size.y {
        return;
    }
    let texel = textureLoad(tex, id.xy, 0);
    if id.x < 4u && id.y < 4u {
        first_block[id.y * 4u + id.x] = texel;
    }
    if all(abs(texel - vec4(1.0, 0.0, 1.0, 1.0)) < vec4(0.002)) {
        atomicAdd(&error_texels, 1u);
    }
}
