// Sums every u32 of `src` (wrapping) into `sink[0]`.

struct Params {
    count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(0) var<storage, read> src: array<vec4<u32>>;
@group(0) @binding(1) var<storage, read_write> sink: array<atomic<u32>, 4>;
@group(0) @binding(2) var<uniform> params: Params;

var<workgroup> partial: atomic<u32>;

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) id: vec3<u32>,
    @builtin(local_invocation_index) local: u32,
    @builtin(num_workgroups) groups: vec3<u32>,
) {
    let stride = groups.x * 256u;
    var acc = 0u;
    for (var i = id.x; i < params.count; i += stride) {
        let v = src[i];
        acc += v.x + v.y + v.z + v.w;
    }
    atomicAdd(&partial, acc);
    workgroupBarrier();
    if local == 0u {
        atomicAdd(&sink[0], atomicLoad(&partial));
    }
}
