int naga_insertBits(
    int e,
    int newbits,
    uint offset,
    uint count
) {
    uint w = 32u;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    uint mask = ((4294967295u >> (32u - c)) << o);
    return (c == 0 ? e : ((e & ~mask) | ((newbits << o) & mask)));
}
int2 naga_insertBits(
    int2 e,
    int2 newbits,
    uint offset,
    uint count
) {
    uint w = 32u;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    uint mask = ((4294967295u >> (32u - c)) << o);
    return (c == 0 ? e : ((e & ~mask) | ((newbits << o) & mask)));
}
int3 naga_insertBits(
    int3 e,
    int3 newbits,
    uint offset,
    uint count
) {
    uint w = 32u;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    uint mask = ((4294967295u >> (32u - c)) << o);
    return (c == 0 ? e : ((e & ~mask) | ((newbits << o) & mask)));
}
int4 naga_insertBits(
    int4 e,
    int4 newbits,
    uint offset,
    uint count
) {
    uint w = 32u;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    uint mask = ((4294967295u >> (32u - c)) << o);
    return (c == 0 ? e : ((e & ~mask) | ((newbits << o) & mask)));
}
uint naga_insertBits(
    uint e,
    uint newbits,
    uint offset,
    uint count
) {
    uint w = 32u;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    uint mask = ((4294967295u >> (32u - c)) << o);
    return (c == 0 ? e : ((e & ~mask) | ((newbits << o) & mask)));
}
uint2 naga_insertBits(
    uint2 e,
    uint2 newbits,
    uint offset,
    uint count
) {
    uint w = 32u;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    uint mask = ((4294967295u >> (32u - c)) << o);
    return (c == 0 ? e : ((e & ~mask) | ((newbits << o) & mask)));
}
uint3 naga_insertBits(
    uint3 e,
    uint3 newbits,
    uint offset,
    uint count
) {
    uint w = 32u;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    uint mask = ((4294967295u >> (32u - c)) << o);
    return (c == 0 ? e : ((e & ~mask) | ((newbits << o) & mask)));
}
uint4 naga_insertBits(
    uint4 e,
    uint4 newbits,
    uint offset,
    uint count
) {
    uint w = 32u;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    uint mask = ((4294967295u >> (32u - c)) << o);
    return (c == 0 ? e : ((e & ~mask) | ((newbits << o) & mask)));
}
int naga_extractBits(
    int e,
    uint offset,
    uint count
) {
    uint w = 32;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    return (c == 0 ? 0 : (e << (w - c - o)) >> (w - c));
}
int2 naga_extractBits(
    int2 e,
    uint offset,
    uint count
) {
    uint w = 32;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    return (c == 0 ? 0 : (e << (w - c - o)) >> (w - c));
}
int3 naga_extractBits(
    int3 e,
    uint offset,
    uint count
) {
    uint w = 32;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    return (c == 0 ? 0 : (e << (w - c - o)) >> (w - c));
}
int4 naga_extractBits(
    int4 e,
    uint offset,
    uint count
) {
    uint w = 32;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    return (c == 0 ? 0 : (e << (w - c - o)) >> (w - c));
}
uint naga_extractBits(
    uint e,
    uint offset,
    uint count
) {
    uint w = 32;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    return (c == 0 ? 0 : (e << (w - c - o)) >> (w - c));
}
uint2 naga_extractBits(
    uint2 e,
    uint offset,
    uint count
) {
    uint w = 32;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    return (c == 0 ? 0 : (e << (w - c - o)) >> (w - c));
}
uint3 naga_extractBits(
    uint3 e,
    uint offset,
    uint count
) {
    uint w = 32;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    return (c == 0 ? 0 : (e << (w - c - o)) >> (w - c));
}
uint4 naga_extractBits(
    uint4 e,
    uint offset,
    uint count
) {
    uint w = 32;
    uint o = min(offset, w);
    uint c = min(count, w - o);
    return (c == 0 ? 0 : (e << (w - c - o)) >> (w - c));
}
[numthreads(1, 1, 1)]
void main()
{
    int i;
    int2 i2_;
    int3 i3_;
    int4 i4_;
    uint u;
    uint2 u2_;
    uint3 u3_;
    uint4 u4_;
    float2 f2_;
    float4 f4_;

    i = int(0);
    i2_ = (int(0)).xx;
    i3_ = (int(0)).xxx;
    i4_ = (int(0)).xxxx;
    u = 0u;
    u2_ = (0u).xx;
    u3_ = (0u).xxx;
    u4_ = (0u).xxxx;
    f2_ = (0.0).xx;
    f4_ = (0.0).xxxx;
    float4 _e12 = f4_;
    u = uint((int(round(clamp(_e12[0], -1.0, 1.0) * 127.0)) & 0xFF) | ((int(round(clamp(_e12[1], -1.0, 1.0) * 127.0)) & 0xFF) << 8) | ((int(round(clamp(_e12[2], -1.0, 1.0) * 127.0)) & 0xFF) << 16) | ((int(round(clamp(_e12[3], -1.0, 1.0) * 127.0)) & 0xFF) << 24));
    float4 _e14 = f4_;
    u = (uint(round(clamp(_e14[0], 0.0, 1.0) * 255.0)) | uint(round(clamp(_e14[1], 0.0, 1.0) * 255.0)) << 8 | uint(round(clamp(_e14[2], 0.0, 1.0) * 255.0)) << 16 | uint(round(clamp(_e14[3], 0.0, 1.0) * 255.0)) << 24);
    float2 _e16 = f2_;
    u = uint((int(round(clamp(_e16[0], -1.0, 1.0) * 32767.0)) & 0xFFFF) | ((int(round(clamp(_e16[1], -1.0, 1.0) * 32767.0)) & 0xFFFF) << 16));
    float2 _e18 = f2_;
    u = (uint(round(clamp(_e18[0], 0.0, 1.0) * 65535.0)) | uint(round(clamp(_e18[1], 0.0, 1.0) * 65535.0)) << 16);
    float2 _e20 = f2_;
    u = (f32tof16(_e20[0]) | f32tof16(_e20[1]) << 16);
    int4 _e22 = i4_;
    u = uint((_e22[0] & 0xFF) | ((_e22[1] & 0xFF) << 8) | ((_e22[2] & 0xFF) << 16) | ((_e22[3] & 0xFF) << 24));
    uint4 _e24 = u4_;
    u = (_e24[0] & 0xFF) | ((_e24[1] & 0xFF) << 8) | ((_e24[2] & 0xFF) << 16) | ((_e24[3] & 0xFF) << 24);
    int4 _e26 = i4_;
    u = uint((clamp(_e26, -128, 127)[0] & 0xFF) | ((clamp(_e26, -128, 127)[1] & 0xFF) << 8) | ((clamp(_e26, -128, 127)[2] & 0xFF) << 16) | ((clamp(_e26, -128, 127)[3] & 0xFF) << 24));
    uint4 _e28 = u4_;
    u = (clamp(_e28, 0, 255)[0] & 0xFF) | ((clamp(_e28, 0, 255)[1] & 0xFF) << 8) | ((clamp(_e28, 0, 255)[2] & 0xFF) << 16) | ((clamp(_e28, 0, 255)[3] & 0xFF) << 24);
    uint _e30 = u;
    f4_ = (float4(int4(_e30 << 24, _e30 << 16, _e30 << 8, _e30) >> 24) / 127.0);
    uint _e32 = u;
    f4_ = (float4(_e32 & 0xFF, _e32 >> 8 & 0xFF, _e32 >> 16 & 0xFF, _e32 >> 24) / 255.0);
    uint _e34 = u;
    f2_ = (float2(int2(_e34 << 16, _e34) >> 16) / 32767.0);
    uint _e36 = u;
    f2_ = (float2(_e36 & 0xFFFF, _e36 >> 16) / 65535.0);
    uint _e38 = u;
    f2_ = float2(f16tof32(_e38), f16tof32((_e38) >> 16));
    uint _e40 = u;
    i4_ = (int4(_e40, _e40 >> 8, _e40 >> 16, _e40 >> 24) << 24 >> 24);
    uint _e42 = u;
    u4_ = (uint4(_e42, _e42 >> 8, _e42 >> 16, _e42 >> 24) << 24 >> 24);
    int _e44 = i;
    int _e45 = i;
    i = naga_insertBits(_e44, _e45, 5u, 10u);
    int2 _e49 = i2_;
    int2 _e50 = i2_;
    i2_ = naga_insertBits(_e49, _e50, 5u, 10u);
    int3 _e54 = i3_;
    int3 _e55 = i3_;
    i3_ = naga_insertBits(_e54, _e55, 5u, 10u);
    int4 _e59 = i4_;
    int4 _e60 = i4_;
    i4_ = naga_insertBits(_e59, _e60, 5u, 10u);
    uint _e64 = u;
    uint _e65 = u;
    u = naga_insertBits(_e64, _e65, 5u, 10u);
    uint2 _e69 = u2_;
    uint2 _e70 = u2_;
    u2_ = naga_insertBits(_e69, _e70, 5u, 10u);
    uint3 _e74 = u3_;
    uint3 _e75 = u3_;
    u3_ = naga_insertBits(_e74, _e75, 5u, 10u);
    uint4 _e79 = u4_;
    uint4 _e80 = u4_;
    u4_ = naga_insertBits(_e79, _e80, 5u, 10u);
    int _e84 = i;
    i = naga_extractBits(_e84, 5u, 10u);
    int2 _e88 = i2_;
    i2_ = naga_extractBits(_e88, 5u, 10u);
    int3 _e92 = i3_;
    i3_ = naga_extractBits(_e92, 5u, 10u);
    int4 _e96 = i4_;
    i4_ = naga_extractBits(_e96, 5u, 10u);
    uint _e100 = u;
    u = naga_extractBits(_e100, 5u, 10u);
    uint2 _e104 = u2_;
    u2_ = naga_extractBits(_e104, 5u, 10u);
    uint3 _e108 = u3_;
    u3_ = naga_extractBits(_e108, 5u, 10u);
    uint4 _e112 = u4_;
    u4_ = naga_extractBits(_e112, 5u, 10u);
    int _e116 = i;
    i = asint(firstbitlow(_e116));
    uint2 _e118 = u2_;
    u2_ = firstbitlow(_e118);
    int3 _e120 = i3_;
    i3_ = asint(firstbithigh(_e120));
    uint3 _e122 = u3_;
    u3_ = firstbithigh(_e122);
    int _e124 = i;
    i = asint(firstbithigh(_e124));
    uint _e126 = u;
    u = firstbithigh(_e126);
    int _e128 = i;
    i = asint(countbits(asuint(_e128)));
    int2 _e130 = i2_;
    i2_ = asint(countbits(asuint(_e130)));
    int3 _e132 = i3_;
    i3_ = asint(countbits(asuint(_e132)));
    int4 _e134 = i4_;
    i4_ = asint(countbits(asuint(_e134)));
    uint _e136 = u;
    u = countbits(_e136);
    uint2 _e138 = u2_;
    u2_ = countbits(_e138);
    uint3 _e140 = u3_;
    u3_ = countbits(_e140);
    uint4 _e142 = u4_;
    u4_ = countbits(_e142);
    int _e144 = i;
    i = asint(reversebits(asuint(_e144)));
    int2 _e146 = i2_;
    i2_ = asint(reversebits(asuint(_e146)));
    int3 _e148 = i3_;
    i3_ = asint(reversebits(asuint(_e148)));
    int4 _e150 = i4_;
    i4_ = asint(reversebits(asuint(_e150)));
    uint _e152 = u;
    u = reversebits(_e152);
    uint2 _e154 = u2_;
    u2_ = reversebits(_e154);
    uint3 _e156 = u3_;
    u3_ = reversebits(_e156);
    uint4 _e158 = u4_;
    u4_ = reversebits(_e158);
    return;
}
