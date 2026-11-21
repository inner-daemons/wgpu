#version 460
#if defined(GL_ARB_gpu_shader_int64)
#extension GL_ARB_gpu_shader_int64 : require
#else
#error No extension available for 64-bit integers.
#endif
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

struct _14
{
    uint _m0;
    int _m1;
    float _m2;
    uint64_t _m3;
    u64vec2 _m4;
    u64vec3 _m5;
    u64vec4 _m6;
    int64_t _m7;
    i64vec2 _m8;
    i64vec3 _m9;
    i64vec4 _m10;
    uint64_t _m11;
};

struct _18
{
    uint64_t _m0[2];
    int64_t _m1[2];
};

layout(set = 0, binding = 0, std140) uniform _23_22
{
    _14 _m0;
} _22;

layout(set = 0, binding = 1, std430) readonly buffer _26_25
{
    _14 _m0;
} _25;

layout(set = 0, binding = 2, std430) readonly buffer _29_28
{
    _18 _m0;
} _28;

layout(set = 0, binding = 3, std430) buffer _32_31
{
    _14 _m0;
} _31;

layout(set = 0, binding = 4, std430) buffer _35_34
{
    _18 _m0;
} _34;

int64_t _20;

uint64_t _221(uint64_t _220)
{
    uint64_t _231 = 20ul;
    _231 += ((31ul + 18446744073709551615ul) - 18446744073709551615ul);
    _231 += (_231 + 5ul);
    _231 += uint64_t(_22._m0._m0 + uint(_231));
    _231 += uint64_t(uint(_22._m0._m1 + int(_231)));
    _231 += uint64_t(clamp(_22._m0._m2 + float(_231), 0.0, 18446742974197923840.0));
    _231 += u64vec3(_22._m0._m3).z;
    _231 += uint64_t(_22._m0._m7);
    _231 += u64vec2(_22._m0._m8).y;
    _231 += u64vec3(_22._m0._m9).z;
    _231 += u64vec4(_22._m0._m10).w;
    _31._m0._m3 = _22._m0._m3 + _25._m0._m3;
    _31._m0._m4 = _22._m0._m4 + _25._m0._m4;
    _31._m0._m5 = _22._m0._m5 + _25._m0._m5;
    _31._m0._m6 = _22._m0._m6 + _25._m0._m6;
    _34._m0._m0 = _28._m0._m0;
    uint64_t _332 = _231;
    _231 += _332;
    _231 += min(max(_231, _231), _231);
    u64vec2 _343 = u64vec2(_231);
    u64vec2 _345 = u64vec2(_231);
    _231 += ((0ul + (_343.x * _345.x)) + (_343.y * _345.y));
    _231 += max(_231, _231);
    _231 += min(_231, _231);
    return _231;
}

int64_t _39(int64_t _38)
{
    int64_t _56 = 20l;
    _56 += ((31l - 1002003004005006l) + (-9223372036854775807l));
    _56 += (_56 + 5l);
    _56 += int64_t(int(_22._m0._m0 + uint(_56)));
    _56 += int64_t(_22._m0._m1 + int(_56));
    _56 += int64_t(clamp(_22._m0._m2 + float(_56), -9223372036854775808.0, 9223371487098961920.0));
    _56 += i64vec3(_22._m0._m7).z;
    _56 += int64_t(_22._m0._m3);
    _56 += i64vec2(_22._m0._m4).y;
    _56 += i64vec3(_22._m0._m5).z;
    _56 += i64vec4(_22._m0._m6).w;
    _56 += int64_t(0x8000000000000000ul);
    _31._m0._m7 = _22._m0._m7 + _25._m0._m7;
    _31._m0._m8 = _22._m0._m8 + _25._m0._m8;
    _31._m0._m9 = _22._m0._m9 + _25._m0._m9;
    _31._m0._m10 = _22._m0._m10 + _25._m0._m10;
    _34._m0._m1 = _28._m0._m1;
    _56 += abs(_56);
    _56 += min(max(_56, _56), _56);
    i64vec2 _191 = i64vec2(_56);
    i64vec2 _193 = i64vec2(_56);
    _56 += ((0l + (_191.x * _193.x)) + (_191.y * _193.y));
    _56 += max(_56, _56);
    _56 += min(_56, _56);
    _56 += sign(_56);
    return _56;
}

void main()
{
    _20 = 1l;
    uint64_t _379 = _221(67ul);
    int64_t _380 = _39(60l);
    _31._m0._m11 = _379 + uint64_t(_380);
}

