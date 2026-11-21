////////////////////////////////////////////////////////////
// Entry point: "test_atomic_compare_exchange_i32" (comp) //
////////////////////////////////////////////////////////////
#version 460
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

struct _10
{
    int _m0;
    bool _m1;
};

struct _11
{
    uint _m0;
    bool _m1;
};

layout(set = 0, binding = 0, std430) buffer _13_12
{
    int _m0[128];
} _12;

layout(set = 0, binding = 1, std430) buffer _16_15
{
    uint _m0[128];
} _15;

void main()
{
    uvec2 _45 = uvec2(4294967295u);
    uvec2 _71 = uvec2(4294967295u);
    int _30 = 0;
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
        int _65 = atomicAdd(_12._m0[_28], 0);
        _30 = _65;
        _32 = false;
        for (;;)
        {
            if (all(equal(uvec2(0u), _71)))
            {
                break;
            }
            _71 -= uvec2(uint(_71.y == 0u), 1u);
            if (!(!_32))
            {
                break;
            }
            int _93 = _30;
            int _96 = atomicCompSwap(_12._m0[_28], _93, floatBitsToInt(intBitsToFloat(_30) + 1.0));
            _10 _94 = _10(_96, _96 == _93);
            _30 = _94._m0;
            _32 = _94._m1;
            continue;
        }
        _28++;
        continue;
    }
}


////////////////////////////////////////////////////////////
// Entry point: "test_atomic_compare_exchange_u32" (comp) //
////////////////////////////////////////////////////////////
#version 460
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

struct _10
{
    int _m0;
    bool _m1;
};

struct _11
{
    uint _m0;
    bool _m1;
};

layout(set = 0, binding = 0, std430) buffer _13_12
{
    int _m0[128];
} _12;

layout(set = 0, binding = 1, std430) buffer _16_15
{
    uint _m0[128];
} _15;

void main()
{
    uvec2 _114 = uvec2(4294967295u);
    uvec2 _139 = uvec2(4294967295u);
    uint _107 = 0u;
    bool _108 = false;
    uint _106 = 0u;
    for (;;)
    {
        if (all(equal(uvec2(0u), _114)))
        {
            break;
        }
        _114 -= uvec2(uint(_114.y == 0u), 1u);
        if (!(_106 < 128u))
        {
            break;
        }
        uint _134 = atomicAdd(_15._m0[_106], 0u);
        _107 = _134;
        _108 = false;
        for (;;)
        {
            if (all(equal(uvec2(0u), _139)))
            {
                break;
            }
            _139 -= uvec2(uint(_139.y == 0u), 1u);
            if (!(!_108))
            {
                break;
            }
            uint _161 = _107;
            uint _164 = atomicCompSwap(_15._m0[_106], _161, floatBitsToUint(uintBitsToFloat(_107) + 1.0));
            _11 _162 = _11(_164, _164 == _161);
            _107 = _162._m0;
            _108 = _162._m1;
            continue;
        }
        _106++;
        continue;
    }
}

