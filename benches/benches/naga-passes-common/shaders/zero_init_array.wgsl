// Workgroup memory heavy: one large workgroup array.
// The runtime instantiates it with different `WG` and `WORDS` values.

struct Params {
    ctl: vec4<u32>, // x: seed, y/z/w: trip counts
}

@group(0) @binding(0) var<storage, read_write> results: array<u32>;
@group(0) @binding(1) var<uniform> params: Params;

override BENCH_SALT: u32 = 0u;

const WG: u32 = 64u;
const WORDS: u32 = 256u;

var<workgroup> scratch: array<u32, WORDS>;

@compute @workgroup_size(WG)
fn main(
    @builtin(local_invocation_index) lid: u32,
    @builtin(workgroup_id) wid: vec3<u32>,
) {
    let seed = params.ctl.x ^ BENCH_SALT;

    for (var i = lid; i < WORDS; i += WG) {
        scratch[i] = (i * 2654435761u) ^ seed ^ wid.x;
    }
    workgroupBarrier();

    var acc = 0u;
    for (var i = lid; i < WORDS; i += WG) {
        let j = (i * 7u + lid) % WORDS;
        acc = acc * 31u + scratch[j];
    }
    results[wid.x * WG + lid] = acc;
}
