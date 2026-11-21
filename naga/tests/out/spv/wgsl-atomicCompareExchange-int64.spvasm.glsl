////////////////////////////////////////////////////////////
// Entry point: "test_atomic_compare_exchange_i64" (comp) //
////////////////////////////////////////////////////////////
#version 460
#if defined(GL_ARB_gpu_shader_int64)
#extension GL_ARB_gpu_shader_int64 : require
#else
#error No extension available for 64-bit integers.
#endif
#extension GL_EXT_shader_atomic_int64 : require
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

struct _10
{
    int64_t _m0;
    bool _m1;
};

struct _11
{
    uint64_t _m0;
    bool _m1;
};

layout(set = 0, binding = 0, std430) buffer _13_12
{
    int64_t _m0[128];
} _12;

layout(set = 0, binding = 1, std430) buffer _16_15
{
    uint64_t _m0[128];
} _15;

void main()
{
    uvec2 _45 = uvec2(4294967295u);
    uvec2 _72 = uvec2(4294967295u);
    int64_t _30 = 0l;
    bool _32 = false;
    uint _28 = 0u;
    for (;;)
    {
        if (all(equal(uvec2(0u), _45)))
        {
            break;
        }
        _45 -= uvec2(uint(_45.y == 0u), 1u);
        if (!(_28 < 128u))
        {
            break;
        }
        int64_t _65 = atomicAdd(_12._m0[_28], 0);
        _30 = _65;
        _32 = false;
        for (;;)
        {
            if (all(equal(uvec2(0u), _72)))
            {
                break;
            }
            _72 -= uvec2(uint(_72.y == 0u), 1u);
            if (!(!_32))
            {
                break;
            }
            int64_t _92 = _30;
            int64_t _95 = atomicCompSwap(_12._m0[_28], _92, _30 + 10l);
            _10 _93 = _10(_95, _95 == _92);
            _30 = _93._m0;
            _32 = _93._m1;
            continue;
        }
        _28++;
        continue;
    }
}


////////////////////////////////////////////////////////////
// Entry point: "test_atomic_compare_exchange_u64" (comp) //
////////////////////////////////////////////////////////////
#version 460
#if defined(GL_ARB_gpu_shader_int64)
#extension GL_ARB_gpu_shader_int64 : require
#else
#error No extension available for 64-bit integers.
#endif
#extension GL_EXT_shader_atomic_int64 : require
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

struct _10
{
    int64_t _m0;
    bool _m1;
};

struct _11
{
    uint64_t _m0;
    bool _m1;
};

layout(set = 0, binding = 0, std430) buffer _13_12
{
    int64_t _m0[128];
} _12;

layout(set = 0, binding = 1, std430) buffer _16_15
{
    uint64_t _m0[128];
} _15;

void main()
{
    uvec2 _116 = uvec2(4294967295u);
    uvec2 _141 = uvec2(4294967295u);
    uint64_t _108 = 0ul;
    bool _110 = false;
    uint _107 = 0u;
    for (;;)
    {
        if (all(equal(uvec2(0u), _116)))
        {
            break;
        }
        _116 -= uvec2(uint(_116.y == 0u), 1u);
        if (!(_107 < 128u))
        {
            break;
        }
        uint64_t _136 = atomicAdd(_15._m0[_107], 0);
        _108 = _136;
        _110 = false;
        for (;;)
        {
            if (all(equal(uvec2(0u), _141)))
            {
                break;
            }
            _141 -= uvec2(uint(_141.y == 0u), 1u);
            if (!(!_110))
            {
                break;
            }
            uint64_t _161 = _108;
            uint64_t _164 = atomicCompSwap(_15._m0[_107], _161, _108 + 10ul);
            _11 _162 = _11(_164, _164 == _161);
            _108 = _162._m0;
            _110 = _162._m1;
            continue;
        }
        _107++;
        continue;
    }
}

