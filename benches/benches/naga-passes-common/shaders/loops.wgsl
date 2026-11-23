// Loop heavy: nested loops, an explicit `loop`, and a `while`, all with trip
// counts that are only known at run time.

struct Params {
    ctl: vec4<u32>, // x: seed, y: outer trips, z: inner trips, w: tail trips
}

@group(0) @binding(0) var<storage, read_write> results: array<u32>;
@group(0) @binding(1) var<uniform> params: Params;

override BENCH_SALT: u32 = 0u;

const WG: u32 = 64u;

@compute @workgroup_size(WG)
fn main(
    @builtin(local_invocation_index) lid: u32,
    @builtin(workgroup_id) wid: vec3<u32>,
) {
    let seed = params.ctl.x ^ BENCH_SALT;
    let n0 = params.ctl.y;
    let n1 = params.ctl.z;
    let n2 = params.ctl.w;

    var acc = seed + lid;
    for (var i = 0u; i < n0; i++) {
        for (var j = 0u; j < n1; j++) {
            acc = acc * 1664525u + (i ^ j) + 1013904223u;
            if (acc & 15u) == 3u {
                continue;
            }
            acc ^= acc >> 7u;
            if acc == 0xdeadbeefu {
                break;
            }
        }
    }

    var k = 0u;
    loop {
        if k >= n2 {
            break;
        }
        acc += k * 3u + (acc >> 5u);
        continuing {
            k++;
        }
    }

    var w = n2;
    while w > 0u {
        acc = acc * 5u + w;
        w -= 1u;
    }

    results[wid.x * WG + lid] = acc;
}
