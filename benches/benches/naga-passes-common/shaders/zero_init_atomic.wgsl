// Workgroup memory heavy: an array of atomics plus a scalar atomic.

struct Params {
    ctl: vec4<u32>,
}

@group(0) @binding(0) var<storage, read_write> results: array<u32>;
@group(0) @binding(1) var<uniform> params: Params;

override BENCH_SALT: u32 = 0u;

const WG: u32 = 64u;
const SLOTS: u32 = 1024u;

var<workgroup> counters: array<atomic<u32>, SLOTS>;
var<workgroup> total: atomic<u32>;

@compute @workgroup_size(WG)
fn main(
    @builtin(local_invocation_index) lid: u32,
    @builtin(workgroup_id) wid: vec3<u32>,
) {
    let seed = params.ctl.x ^ BENCH_SALT;

    for (var i = 0u; i < 16u; i++) {
        let k = (lid * 2654435761u + i * 40503u + seed) % SLOTS;
        atomicAdd(&counters[k], i + 1u);
    }
    atomicAdd(&total, lid);
    workgroupBarrier();

    var acc = atomicLoad(&total);
    for (var i = lid; i < SLOTS; i += WG) {
        acc = acc * 31u + atomicLoad(&counters[i]);
    }
    results[wid.x * WG + lid] = acc;
}
