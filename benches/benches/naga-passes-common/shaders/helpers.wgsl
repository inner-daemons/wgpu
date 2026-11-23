// Helper heavy: many small functions with pointer parameters, nested several
// calls deep. Backends that cannot express a call (or that inline everything,
// like HLSL after the inlining passes) pay for it here.

struct Params {
    ctl: vec4<u32>, // x: seed, y: trips
}

struct State {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    arr: array<u32, 8>,
}

@group(0) @binding(0) var<storage, read_write> results: array<u32>;
@group(0) @binding(1) var<uniform> params: Params;

override BENCH_SALT: u32 = 0u;

const WG: u32 = 64u;

fn pure_0(x: u32, y: u32) -> u32 {
    return (x ^ (y << 0u)) * 3u + y;
}

fn pure_1(x: u32, y: u32) -> u32 {
    return (x ^ (y << 1u)) * 5u + y;
}

fn pure_2(x: u32, y: u32) -> u32 {
    return (x ^ (y << 2u)) * 7u + y;
}

fn pure_3(x: u32, y: u32) -> u32 {
    return (x ^ (y << 3u)) * 9u + y;
}

fn pure_4(x: u32, y: u32) -> u32 {
    return (x ^ (y << 4u)) * 11u + y;
}

fn pure_5(x: u32, y: u32) -> u32 {
    return (x ^ (y << 5u)) * 13u + y;
}

fn pure_6(x: u32, y: u32) -> u32 {
    return (x ^ (y << 6u)) * 15u + y;
}

fn pure_7(x: u32, y: u32) -> u32 {
    return (x ^ (y << 7u)) * 17u + y;
}

fn leaf_0(p: ptr<function, u32>, x: u32) {
    *p = pure_0(*p, x);
}

fn leaf_1(p: ptr<function, u32>, x: u32) {
    *p = pure_1(*p, x);
    leaf_0(p, x ^ 1u);
}

fn leaf_2(p: ptr<function, u32>, x: u32) {
    *p = pure_2(*p, x);
    leaf_1(p, x ^ 2u);
}

fn leaf_3(p: ptr<function, u32>, x: u32) {
    *p = pure_3(*p, x);
    leaf_2(p, x ^ 3u);
}

fn leaf_4(p: ptr<function, u32>, x: u32) {
    *p = pure_4(*p, x);
    leaf_3(p, x ^ 4u);
}

fn leaf_5(p: ptr<function, u32>, x: u32) {
    *p = pure_5(*p, x);
    leaf_4(p, x ^ 5u);
}

fn leaf_6(p: ptr<function, u32>, x: u32) {
    *p = pure_6(*p, x);
    leaf_5(p, x ^ 6u);
}

fn leaf_7(p: ptr<function, u32>, x: u32) {
    *p = pure_7(*p, x);
    leaf_6(p, x ^ 7u);
}

fn arr_0(a: ptr<function, array<u32, 8>>, idx: u32) {
    let i = idx & 7u;
    (*a)[i] = pure_0((*a)[i], idx);
}

fn arr_1(a: ptr<function, array<u32, 8>>, idx: u32) {
    let i = idx & 7u;
    (*a)[i] = pure_1((*a)[i], idx);
    arr_0(a, idx + 1u);
}

fn arr_2(a: ptr<function, array<u32, 8>>, idx: u32) {
    let i = idx & 7u;
    (*a)[i] = pure_2((*a)[i], idx);
    arr_1(a, idx + 1u);
}

fn arr_3(a: ptr<function, array<u32, 8>>, idx: u32) {
    let i = idx & 7u;
    (*a)[i] = pure_3((*a)[i], idx);
    arr_2(a, idx + 1u);
}

fn arr_4(a: ptr<function, array<u32, 8>>, idx: u32) {
    let i = idx & 7u;
    (*a)[i] = pure_4((*a)[i], idx);
    arr_3(a, idx + 1u);
}

fn arr_5(a: ptr<function, array<u32, 8>>, idx: u32) {
    let i = idx & 7u;
    (*a)[i] = pure_5((*a)[i], idx);
    arr_4(a, idx + 1u);
}

fn arr_6(a: ptr<function, array<u32, 8>>, idx: u32) {
    let i = idx & 7u;
    (*a)[i] = pure_6((*a)[i], idx);
    arr_5(a, idx + 1u);
}

fn arr_7(a: ptr<function, array<u32, 8>>, idx: u32) {
    let i = idx & 7u;
    (*a)[i] = pure_7((*a)[i], idx);
    arr_6(a, idx + 1u);
}

fn state_0(s: ptr<function, State>, x: u32) {
    leaf_0(&(*s).a, x);
    arr_0(&(*s).arr, x + 0u);
    (*s).b += pure_0((*s).c, x);
}

fn state_1(s: ptr<function, State>, x: u32) {
    leaf_1(&(*s).a, x);
    arr_1(&(*s).arr, x + 1u);
    (*s).b += pure_1((*s).c, x);
    state_0(s, x + 1u);
}

fn state_2(s: ptr<function, State>, x: u32) {
    leaf_2(&(*s).a, x);
    arr_2(&(*s).arr, x + 2u);
    (*s).b += pure_2((*s).c, x);
    state_1(s, x + 1u);
}

fn state_3(s: ptr<function, State>, x: u32) {
    leaf_3(&(*s).a, x);
    arr_3(&(*s).arr, x + 3u);
    (*s).b += pure_3((*s).c, x);
    state_2(s, x + 1u);
}

fn state_4(s: ptr<function, State>, x: u32) {
    leaf_4(&(*s).a, x);
    arr_4(&(*s).arr, x + 4u);
    (*s).b += pure_4((*s).c, x);
    state_3(s, x + 1u);
}

fn state_5(s: ptr<function, State>, x: u32) {
    leaf_5(&(*s).a, x);
    arr_5(&(*s).arr, x + 5u);
    (*s).b += pure_5((*s).c, x);
    state_4(s, x + 1u);
}

fn state_6(s: ptr<function, State>, x: u32) {
    leaf_6(&(*s).a, x);
    arr_6(&(*s).arr, x + 6u);
    (*s).b += pure_6((*s).c, x);
    state_5(s, x + 1u);
}

fn state_7(s: ptr<function, State>, x: u32) {
    leaf_7(&(*s).a, x);
    arr_7(&(*s).arr, x + 7u);
    (*s).b += pure_7((*s).c, x);
    state_6(s, x + 1u);
}

@compute @workgroup_size(WG)
fn main(
    @builtin(local_invocation_index) lid: u32,
    @builtin(workgroup_id) wid: vec3<u32>,
) {
    let seed = params.ctl.x ^ BENCH_SALT;
    let trips = params.ctl.y;

    var st = State(seed, lid, 3u, 5u, array<u32, 8>());
    var acc = 0u;
    for (var j = 0u; j < trips; j++) {
        state_7(&st, j ^ seed);
        leaf_4(&st.d, j);
        arr_2(&st.arr, j + lid);
        acc += pure_6(st.a, st.b) ^ st.arr[j & 7u];
    }
    results[wid.x * WG + lid] = acc + st.a + st.b + st.c + st.d;
}
