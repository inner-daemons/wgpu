struct Params {
    width: u32,
    height: u32,
    reps: u32,
    count: u32,
}

@group(0) @binding(0) var<storage, read> src: array<vec4<u32>>;
@group(0) @binding(1) var<storage, read_write> sink: array<vec4<u32>>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) id: vec3<u32>,
    @builtin(num_workgroups) groups: vec3<u32>,
) {
    let stride = groups.x * 256u;
    var acc = vec4<u32>(0u);
    for (var i = id.x; i < params.count; i += stride) {
        acc ^= src[i];
    }
    if all(acc == vec4<u32>(0xdeadbeefu)) {
        sink[0] = acc;
    }
}

@compute @workgroup_size(1)
fn tiny() {
    sink[0] += vec4<u32>(1u);
}
