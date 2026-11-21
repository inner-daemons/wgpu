// language: metal1.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;


kernel void main_(
) {
    metal::int2 i2_;
    metal::int3 i3_;
    metal::int4 i4_;
    metal::uint2 u2_;
    metal::uint3 u3_;
    metal::uint4 u4_;
    metal::float2 f2_;
    metal::float3 f3_;
    metal::float4 f4_;
    i2_ = metal::int2(0);
    i3_ = metal::int3(0);
    i4_ = metal::int4(0);
    u2_ = metal::uint2(0u);
    u3_ = metal::uint3(0u);
    u4_ = metal::uint4(0u);
    f2_ = metal::float2(0.0);
    f3_ = metal::float3(0.0);
    f4_ = metal::float4(0.0);
    metal::int2 _e9 = i2_;
    u2_ = as_type<metal::uint2>(_e9);
    metal::int3 _e11 = i3_;
    u3_ = as_type<metal::uint3>(_e11);
    metal::int4 _e13 = i4_;
    u4_ = as_type<metal::uint4>(_e13);
    metal::uint2 _e15 = u2_;
    i2_ = as_type<metal::int2>(_e15);
    metal::uint3 _e17 = u3_;
    i3_ = as_type<metal::int3>(_e17);
    metal::uint4 _e19 = u4_;
    i4_ = as_type<metal::int4>(_e19);
    metal::int2 _e21 = i2_;
    f2_ = as_type<metal::float2>(_e21);
    metal::int3 _e23 = i3_;
    f3_ = as_type<metal::float3>(_e23);
    metal::int4 _e25 = i4_;
    f4_ = as_type<metal::float4>(_e25);
    return;
}
