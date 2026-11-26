// Writes `pattern(word index, seed)` to every u32 of `dst`.

struct Params {
    count: u32,
    seed: u32,
    _pad0: u32,
    _pad1: u32,
}

@group(0) @binding(0) var<storage, read_write> dst: array<vec4<u32>>;
@group(0) @binding(1) var<uniform> params: Params;

fn pattern(word: u32) -> u32 {
    return (word * 0x9e3779b1u) ^ params.seed;
}

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) id: vec3<u32>,
    @builtin(num_workgroups) groups: vec3<u32>,
) {
    let stride = groups.x * 256u;
    for (var i = id.x; i < params.count; i += stride) {
        let w = i * 4u;
        dst[i] = vec4<u32>(pattern(w), pattern(w + 1u), pattern(w + 2u), pattern(w + 3u));
    }
}
