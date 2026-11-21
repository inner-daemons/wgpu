// language: metal1.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

constant uint g1_ = 1u;
constant float g3_ = 1.0;
constant metal::int4 g4_ = metal::int4 {};
constant metal::int4 g5_ = metal::int4(1);
constant metal::float2x2 g6_ = metal::float2x2(metal::float2(0.0, 0.0), metal::float2(0.0, 0.0));

kernel void main_(
) {
    int g0x;
    float g2x;
    metal::float2x2 g7x;
    int c0x;
    uint c1x;
    float c2x;
    float c3x;
    metal::int4 c4x;
    metal::int4 c5x;
    metal::float2x2 c6x;
    metal::float2x2 c7x;
    int l0x;
    uint l1x;
    float l2x;
    float l3x;
    metal::int4 l4x;
    int v0_;
    uint v1_;
    float v2_;
    float v3_;
    metal::int4 v4_;
    metal::int4 v5_;
    metal::float2x2 v6_;
    metal::float2x2 v7_;
    g0x = 1;
    g2x = 1.0;
    g7x = metal::float2x2(metal::float2(1.0, 1.0), metal::float2(1.0, 1.0));
    c0x = 1;
    c1x = 1u;
    c2x = 1.0;
    c3x = 1.0;
    c4x = metal::int4 {};
    c5x = metal::int4(1);
    c6x = metal::float2x2(metal::float2(0.0, 0.0), metal::float2(0.0, 0.0));
    c7x = metal::float2x2(metal::float2(1.0, 1.0), metal::float2(1.0, 1.0));
    l0x = 0;
    l1x = 0u;
    l2x = 0.0;
    l3x = 0.0;
    l4x = metal::int4 {};
    v0_ = 1;
    v1_ = 1u;
    v2_ = 1.0;
    v3_ = 1.0;
    v4_ = metal::int4 {};
    v5_ = metal::int4(1);
    v6_ = metal::float2x2(metal::float2(0.0, 0.0), metal::float2(0.0, 0.0));
    v7_ = metal::float2x2(metal::float2(1.0, 1.0), metal::float2(1.0, 1.0));
    metal::int4 l5_ = metal::int4(1);
    metal::float2x2 l6_ = metal::float2x2(metal::float2(0.0, 0.0), metal::float2(0.0, 0.0));
    metal::float2x2 l7_ = metal::float2x2(metal::float2(1.0, 1.0), metal::float2(1.0, 1.0));
    l0x = 1;
    l1x = 1u;
    l2x = 1.0;
    l3x = 1.0;
    l4x = metal::int4 {};
    return;
}
