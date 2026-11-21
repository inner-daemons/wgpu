// language: metal1.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;


kernel void main_(
) {
    half h;
    short i;
    ushort u;
    metal::half4 h4_;
    metal::ushort4 u4_;
    h = 1.0h;
    i = static_cast<short>(1);
    u = static_cast<ushort>(2);
    h4_ = metal::half4(1.0h, 2.0h, 3.0h, 4.0h);
    u4_ = metal::ushort4(static_cast<ushort>(1), static_cast<ushort>(2), static_cast<ushort>(3), static_cast<ushort>(4));
    half _e8 = h;
    i = as_type<short>(_e8);
    half _e10 = h;
    u = as_type<ushort>(_e10);
    short _e12 = i;
    h = as_type<half>(_e12);
    ushort _e14 = u;
    h = as_type<half>(_e14);
    metal::half4 _e16 = h4_;
    u4_ = as_type<metal::ushort4>(_e16);
    metal::ushort4 _e18 = u4_;
    h4_ = as_type<metal::half4>(_e18);
    return;
}
