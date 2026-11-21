// language: metal1.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;


kernel void main_(
) {
    metal::int2 x0_;
    metal::float2 i1_;
    x0_ = metal::int2(1, 2);
    i1_ = metal::float2 {};
    int _e9 = x0_.x;
    int _e11 = x0_.y;
    i1_ = (_e9 < _e11) ? metal::float2(0.0, 1.0) : metal::float2(1.0, 0.0);
    return;
}
