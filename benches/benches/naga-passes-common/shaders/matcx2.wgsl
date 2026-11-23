// Uniform matCx2 heavy: reads of mat2x2, mat3x2 and mat4x2 uniform members, in
// plain members and in arrays, with constant and dynamic indices.
// All values are small integers so the float math is exact.

struct M32 {
    @size(32) m: mat3x2<f32>,
}

struct Mats {
    ctl: vec4<u32>, // x: seed, y: trips
    a: mat2x2<f32>,
    b: mat3x2<f32>,
    c: mat4x2<f32>,
    d: mat2x2<f32>,
    @align(16) arr2: array<mat2x2<f32>, 8>,
    @align(16) arr3: array<M32, 8>,
    @align(16) arr4: array<mat4x2<f32>, 8>,
}

@group(0) @binding(0) var<storage, read_write> results: array<u32>;
@group(0) @binding(1) var<uniform> u: Mats;

override BENCH_SALT: u32 = 0u;

const WG: u32 = 64u;

@compute @workgroup_size(WG)
fn main(
    @builtin(local_invocation_index) lid: u32,
    @builtin(workgroup_id) wid: vec3<u32>,
) {
    let seed = u.ctl.x ^ BENCH_SALT;
    let trips = u.ctl.y;

    var sum = 0.0;
    for (var i = 0u; i < trips; i++) {
        let k = (i + lid + seed) & 7u;
        let f = f32((i + lid) & 3u);
        let v2 = vec2<f32>(f, 1.0);
        let v3 = vec3<f32>(f, 1.0, 2.0);
        let v4 = vec4<f32>(f, 1.0, 2.0, 3.0);

        var r = u.a * v2 + u.b * v3 + u.c * v4 + u.d * v2;
        r += u.arr2[k] * v2;
        r += u.arr3[k].m * v3;
        r += u.arr4[k] * v4;
        r += u.b[1] + u.c[2] + vec2<f32>(u.a[1][0], u.d[0][1]);
        r += u.arr3[(k + 1u) & 7u].m[2];
        sum += r.x + 2.0 * r.y;
    }
    results[wid.x * WG + lid] = u32(sum);
}
