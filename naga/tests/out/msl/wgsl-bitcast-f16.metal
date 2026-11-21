// language: metal1.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;


kernel void main_(
) {
    metal::half2 h2_;
    metal::half4 h4_;
    int i1_;
    uint u1_;
    float f1_;
    metal::int2 i2_;
    metal::uint2 u2_;
    metal::float2 f2_;
    h2_ = metal::half2(1.0h, 2.0h);
    h4_ = metal::half4(1.0h, 2.0h, 3.0h, 4.0h);
    i1_ = 0;
    u1_ = 0u;
    f1_ = 0.0;
    i2_ = metal::int2(0);
    u2_ = metal::uint2(0u);
    f2_ = metal::float2(0.0);
    metal::half2 _e11 = h2_;
    i1_ = as_type<int>(_e11);
    metal::half2 _e13 = h2_;
    u1_ = as_type<uint>(_e13);
    metal::half2 _e15 = h2_;
    f1_ = as_type<float>(_e15);
    metal::half4 _e17 = h4_;
    i2_ = as_type<metal::int2>(_e17);
    metal::half4 _e19 = h4_;
    u2_ = as_type<metal::uint2>(_e19);
    metal::half4 _e21 = h4_;
    f2_ = as_type<metal::float2>(_e21);
    int _e23 = i1_;
    h2_ = as_type<metal::half2>(_e23);
    uint _e25 = u1_;
    h2_ = as_type<metal::half2>(_e25);
    float _e27 = f1_;
    h2_ = as_type<metal::half2>(_e27);
    metal::int2 _e29 = i2_;
    h4_ = as_type<metal::half4>(_e29);
    metal::uint2 _e31 = u2_;
    h4_ = as_type<metal::half4>(_e31);
    metal::float2 _e33 = f2_;
    h4_ = as_type<metal::half4>(_e33);
    metal::half2 _e35 = h2_;
    h2_ = as_type<metal::half2>(_e35);
    uint _e37 = u1_;
    u1_ = as_type<uint>(_e37);
    u1_ = as_type<uint>(4294967295u);
    f1_ = as_type<float>(1);
    i1_ = as_type<int>(1.0);
    h2_ = as_type<metal::half2>(1);
    h2_ = as_type<metal::half2>(metal::half2(1.0h, 2.0h));
    h4_ = as_type<metal::half4>(metal::int2(1, 2));
    u2_ = as_type<metal::uint2>(metal::uint2(4294967295u, 1u));
    return;
}
