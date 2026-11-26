// Every invocation sums the whole uniform buffer, starting at a different
// element, and adds the result into `sink[0]`.

@group(0) @binding(0) var<uniform> data: array<vec4<u32>, 4096>;
@group(0) @binding(1) var<storage, read_write> sink: array<atomic<u32>, 4>;

var<workgroup> partial: atomic<u32>;

@compute @workgroup_size(64)
fn main(
    @builtin(global_invocation_id) id: vec3<u32>,
    @builtin(local_invocation_index) local: u32,
) {
    var acc = 0u;
    for (var j = 0u; j < 4096u; j++) {
        let v = data[(j + id.x) & 4095u];
        acc += v.x + v.y + v.z + v.w;
    }
    atomicAdd(&partial, acc);
    workgroupBarrier();
    if local == 0u {
        atomicAdd(&sink[0], atomicLoad(&partial));
    }
}
