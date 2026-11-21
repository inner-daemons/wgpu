#version 460
#extension GL_EXT_shader_explicit_arithmetic_types_int16 : require
#extension GL_EXT_shader_16bit_storage : require
#extension GL_KHR_shader_subgroup_basic : require
#extension GL_EXT_shader_subgroup_extended_types_int16 : require
#extension GL_KHR_shader_subgroup_arithmetic : require
#extension GL_KHR_shader_subgroup_ballot : require
#extension GL_KHR_shader_subgroup_shuffle : require
layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

struct _14
{
    uint _m0;
    int _m1;
    float _m2;
    uint16_t _m3;
    u16vec2 _m4;
    u16vec3 _m5;
    u16vec4 _m6;
    int16_t _m7;
    i16vec2 _m8;
    i16vec3 _m9;
    i16vec4 _m10;
    uint16_t _m11;
};

struct _18
{
    uint16_t _m0[2];
    int16_t _m1[2];
};

layout(set = 0, binding = 0, std140) uniform _26_25
{
    _14 _m0;
} _25;

layout(set = 0, binding = 1, std430) readonly buffer _29_28
{
    _14 _m0;
} _28;

layout(set = 0, binding = 2, std430) readonly buffer _32_31
{
    _18 _m0;
} _31;

layout(set = 0, binding = 3, std430) buffer _35_34
{
    _14 _m0;
} _34;

layout(set = 0, binding = 4, std430) buffer _38_37
{
    _18 _m0;
} _37;

int16_t _23;
shared uint16_t _40;

uint16_t _242(uint16_t _244, uint16_t _245)
{
    return _244 / ((_245 == 0us) ? 1us : _245);
}

uint16_t _251(uint16_t _252, uint16_t _253)
{
    return _252 % ((_253 == 0us) ? 1us : _253);
}

uint16_t _260(uint16_t _259)
{
    uint16_t _273 = 20us;
    _273 += 5us;
    _273 += uint16_t(_25._m0._m0);
    _273 += uint16_t(_25._m0._m1);
    _273 += u16vec3(_25._m0._m3).z;
    _34._m0._m3 = _25._m0._m3 + _28._m0._m3;
    _34._m0._m4 = _25._m0._m4 + _28._m0._m4;
    _34._m0._m5 = _25._m0._m5 + _28._m0._m5;
    _34._m0._m6 = _25._m0._m6 + _28._m0._m6;
    _37._m0._m0 = _31._m0._m0;
    uint16_t _334 = _273;
    _273 = _334;
    _273 = max(_273, _273);
    _273 = min(_273, _273);
    _273 = min(max(_273, _273), _273);
    _273 -= 1us;
    _273 *= 2us;
    _273 = _242(_273, 3us);
    _273 = _251(_273, 4us);
    _273 &= 255us;
    _273 |= 16us;
    _273 ^= 1us;
    _34._m0._m0 = uint(_273);
    _34._m0._m1 = int(int16_t(_273));
    _34._m0._m2 = float(_273);
    _273 = uint16_t(_34._m0._m0);
    return _273;
}

int16_t _42(int16_t _44, int16_t _45)
{
    return _44 / (((_45 == 0s) || ((_44 == (-32768s)) && (_45 == (-1s)))) ? 1s : _45);
}

int16_t _59(int16_t _60, int16_t _61)
{
    int16_t _68 = ((_61 == 0s) || ((_60 == (-32768s)) && (_61 == (-1s)))) ? 1s : _61;
    return _60 - ((_60 / _68) * _68);
}

int16_t _74(int16_t _73)
{
    int16_t _96 = 20s;
    int16_t _98[4] = int16_t[](1s, 2s, 3s, 4s);
    _96 += 5s;
    _96 += int16_t(_25._m0._m0);
    _96 += int16_t(_25._m0._m1);
    _96 += i16vec3(_25._m0._m7).z;
    _34._m0._m7 = _25._m0._m7 + _28._m0._m7;
    _34._m0._m8 = _25._m0._m8 + _28._m0._m8;
    _34._m0._m9 = _25._m0._m9 + _28._m0._m9;
    _34._m0._m10 = _25._m0._m10 + _28._m0._m10;
    _37._m0._m1 = _31._m0._m1;
    _96 = abs(_96);
    _96 = max(_96, _96);
    _96 = min(_96, _96);
    _96 = min(max(_96, _96), _96);
    _96 = sign(_96);
    _96 -= 1s;
    _96 *= 2s;
    _96 = _42(_96, 3s);
    _96 = _59(_96, 4s);
    _96 &= 255s;
    _96 |= 16s;
    _96 ^= 1s;
    _96 = _96 << 2u;
    _96 = _96 >> 1u;
    _96 = -_96;
    _96 = (_96 < 0s) ? 2s : 1s;
    _98[0u] = _96;
    _96 = _98[1u];
    _96 = _98[1u];
    _34._m0._m0 = uint(uint16_t(_96));
    _34._m0._m1 = int(_96);
    _34._m0._m2 = float(_96);
    _96 = int16_t(_34._m0._m0);
    _96 = int16_t(uint16_t(_96));
    _34._m0._m8 = (_25._m0._m8 + _25._m0._m8) * i16vec2(2);
    return _96;
}

void main()
{
    int16_t _388 = 0s;
    uint16_t _389 = 0us;
    _23 = 1s;
    if (gl_LocalInvocationIndex == 0u)
    {
        _40 = 0us;
    }
    barrier();
    _40 = 0us;
    uint16_t _395 = _260(67us);
    int16_t _396 = _74(60s);
    _34._m0._m11 = _395 + uint16_t(_396);
    _388 = int16_t(gl_SubgroupInvocationID);
    _388 = subgroupAdd(_388);
    _388 = subgroupMul(_388);
    _388 = subgroupMin(_388);
    _388 = subgroupMax(_388);
    _388 = subgroupExclusiveAdd(_388);
    _388 = subgroupInclusiveAdd(_388);
    _388 = subgroupBroadcastFirst(_388);
    _388 = subgroupShuffle(_388, 4u);
    _389 = uint16_t(gl_SubgroupInvocationID);
    _389 = subgroupAdd(_389);
    _389 = subgroupMin(_389);
    _389 = subgroupMax(_389);
    _34._m0._m7 = _388;
    _34._m0._m3 = _389;
}

