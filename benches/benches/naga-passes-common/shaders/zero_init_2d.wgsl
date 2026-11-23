// Workgroup memory heavy: a nested 2D workgroup array.

struct Params {
    ctl: vec4<u32>,
}

@group(0) @binding(0) var<storage, read_write> results: array<u32>;
@group(0) @binding(1) var<uniform> params: Params;

override BENCH_SALT: u32 = 0u;

const WG: u32 = 64u;
const ROWS: u32 = 32u;
const COLS: u32 = 64u;

var<workgroup> tile: array<array<u32, COLS>, ROWS>;

@compute @workgroup_size(WG)
fn main(
    @builtin(local_invocation_index) lid: u32,
    @builtin(workgroup_id) wid: vec3<u32>,
) {
    let seed = params.ctl.x ^ BENCH_SALT;

    for (var idx = lid; idx < ROWS * COLS; idx += WG) {
        tile[idx / COLS][idx % COLS] = (idx * 2654435761u) ^ seed ^ wid.x;
    }
    workgroupBarrier();

    var acc = 0u;
    for (var c = 0u; c < COLS; c++) {
        let r = (lid + c * 5u) % ROWS;
        acc = acc * 31u + tile[r][c];
    }
    results[wid.x * WG + lid] = acc;
}
