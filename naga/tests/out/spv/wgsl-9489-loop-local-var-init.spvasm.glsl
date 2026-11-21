#version 460
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0, std430) readonly buffer _11_10
{
    float _m0[64];
} _10;

layout(set = 0, binding = 1, std430) buffer _14_13
{
    float _m0[8];
} _13;

void main()
{
    uvec2 _47 = uvec2(4294967295u);
    uvec2 _68 = uvec2(4294967295u);
    vec4 _31 = vec4(0.0);
    vec4 _33 = vec4(0.0);
    uint _34 = 0u;
    uint _29 = 0u;
    for (;;)
    {
        if (all(equal(uvec2(0u), _47)))
        {
            break;
        }
        _47 -= uvec2(uint(_47.y == 0u), 1u);
        if (!(_29 < 4u))
        {
            break;
        }
        _31 = vec4(0.0);
        _33 = vec4(0.0);
        _34 = 0u;
        for (;;)
        {
            if (all(equal(uvec2(0u), _68)))
            {
                break;
            }
            _68 -= uvec2(uint(_68.y == 0u), 1u);
            if (!(_34 < 16u))
            {
                break;
            }
            vec4 _92 = vec4(_10._m0[(_29 * 16u) + _34]);
            _31 += _92;
            _33 += _92;
            _34++;
            continue;
        }
        _13._m0[_29 * 2u] = _31.x;
        _13._m0[(_29 * 2u) + 1u] = _33.x;
        _29++;
        continue;
    }
}

