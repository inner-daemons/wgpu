////////////////////////////////
// Entry point: "main" (comp) //
////////////////////////////////
#version 460
layout(local_size_x = 8, local_size_y = 2, local_size_z = 1) in;

struct _16
{
    vec4 _m0[5][3];
    uint _m1[19];
    mat3x2 _m2;
    mat2 _m3[3];
};

struct _17
{
    mat2 _m0;
};

layout(set = 0, binding = 0, std430) buffer _44_43
{
    uint _m0[];
} _43;

shared _16 _29[3];
shared uint _31[37];
shared float _33[37];
shared _17 _35;
shared uint _37;
shared uvec3 _39;
shared mat2x3 _41;

uint _46(uint _48, uint _49)
{
    return _48 / ((_49 == 0u) ? 1u : _49);
}

uint _56(uint _57, uint _58)
{
    return _57 % ((_58 == 0u) ? 1u : _58);
}

void main()
{
    uvec2 _128 = uvec2(4294967295u);
    uvec2 _157 = uvec2(4294967295u);
    uvec2 _196 = uvec2(4294967295u);
    uvec2 _217 = uvec2(4294967295u);
    uvec2 _283 = uvec2(4294967295u);
    uvec2 _304 = uvec2(4294967295u);
    uvec2 _325 = uvec2(4294967295u);
    uvec2 _360 = uvec2(4294967295u);
    uvec2 _390 = uvec2(4294967295u);
    uvec2 _439 = uvec2(4294967295u);
    uvec2 _477 = uvec2(4294967295u);
    uvec2 _498 = uvec2(4294967295u);
    uvec2 _519 = uvec2(4294967295u);
    uvec2 _548 = uvec2(4294967295u);
    uvec2 _576 = uvec2(4294967295u);
    bool _98 = false;
    bool _100 = false;
    bool _101 = false;
    bool _102 = false;
    uint _103 = 0u;
    uint _105 = 0u;
    uint _106 = 0u;
    uint _107 = 0u;
    uint _108 = 0u;
    uint _109 = 0u;
    uint _110 = 0u;
    uint _111 = 0u;
    uint _112 = 0u;
    uint _113 = 0u;
    uint _114 = 0u;
    bool _115 = false;
    uint _116 = gl_LocalInvocationIndex;
    for (;;)
    {
        if (all(equal(uvec2(0u), _128)))
        {
            break;
        }
        _128 -= uvec2(uint(_128.y == 0u), 1u);
        if (_116 >= 45u)
        {
            break;
        }
        _29[_46(_116, 15u)]._m0[_56(_46(_116, 3u), 5u)][_56(_116, 3u)] = vec4(0.0);
        _116 += 16u;
        continue;
    }
    _116 = gl_LocalInvocationIndex;
    for (;;)
    {
        if (all(equal(uvec2(0u), _157)))
        {
            break;
        }
        _157 -= uvec2(uint(_157.y == 0u), 1u);
        if (_116 >= 57u)
        {
            break;
        }
        atomicExchange(_29[_46(_116, 19u)]._m1[_56(_116, 19u)], 0u);
        _116 += 16u;
        continue;
    }
    if (gl_LocalInvocationIndex < 3u)
    {
        _29[gl_LocalInvocationIndex]._m2 = mat3x2(vec2(0.0), vec2(0.0), vec2(0.0));
    }
    if (gl_LocalInvocationIndex < 9u)
    {
        _29[_46(gl_LocalInvocationIndex, 3u)]._m3[_56(gl_LocalInvocationIndex, 3u)] = mat2(vec2(0.0), vec2(0.0));
    }
    _116 = gl_LocalInvocationIndex;
    for (;;)
    {
        if (all(equal(uvec2(0u), _196)))
        {
            break;
        }
        _196 -= uvec2(uint(_196.y == 0u), 1u);
        if (_116 >= 37u)
        {
            break;
        }
        _31[_116] = 0u;
        _116 += 16u;
        continue;
    }
    _116 = gl_LocalInvocationIndex;
    for (;;)
    {
        if (all(equal(uvec2(0u), _217)))
        {
            break;
        }
        _217 -= uvec2(uint(_217.y == 0u), 1u);
        if (_116 >= 37u)
        {
            break;
        }
        _33[_116] = 0.0;
        _116 += 16u;
        continue;
    }
    if (gl_LocalInvocationIndex == 0u)
    {
        _35 = _17(mat2(vec2(0.0), vec2(0.0)));
        _37 = 0u;
        _39 = uvec3(0u);
        _41 = mat2x3(vec3(0.0), vec3(0.0));
    }
    barrier();
    if (_37 == 0u)
    {
        _98 = all(equal(_39, uvec3(0u)));
    }
    else
    {
        _98 = false;
    }
    _100 = _98;
    if (all(equal(_41[0u], vec3(0.0))))
    {
        _101 = all(equal(_41[1u], vec3(0.0)));
    }
    else
    {
        _101 = false;
    }
    _100 = _100 && _101;
    if (all(equal(_35._m0[0u], vec2(0.0))))
    {
        _102 = all(equal(_35._m0[1u], vec2(0.0)));
    }
    else
    {
        _102 = false;
    }
    _100 = _100 && _102;
    for (;;)
    {
        if (all(equal(uvec2(0u), _283)))
        {
            break;
        }
        _283 -= uvec2(uint(_283.y == 0u), 1u);
        if (!(_103 < 3u))
        {
            break;
        }
        _105 = 0u;
        for (;;)
        {
            if (all(equal(uvec2(0u), _304)))
            {
                break;
            }
            _304 -= uvec2(uint(_304.y == 0u), 1u);
            if (!(_105 < 5u))
            {
                break;
            }
            _106 = 0u;
            for (;;)
            {
                if (all(equal(uvec2(0u), _325)))
                {
                    break;
                }
                _325 -= uvec2(uint(_325.y == 0u), 1u);
                if (!(_106 < 3u))
                {
                    break;
                }
                _100 = _100 && all(equal(_29[_103]._m0[_105][_106], vec4(0.0)));
                _106++;
                continue;
            }
            _105++;
            continue;
        }
        _107 = 0u;
        for (;;)
        {
            if (all(equal(uvec2(0u), _360)))
            {
                break;
            }
            _360 -= uvec2(uint(_360.y == 0u), 1u);
            if (!(_107 < 19u))
            {
                break;
            }
            uint _381 = atomicAdd(_29[_103]._m1[_107], 0u);
            _100 = _100 && (_381 == 0u);
            _107++;
            continue;
        }
        _108 = 0u;
        for (;;)
        {
            if (all(equal(uvec2(0u), _390)))
            {
                break;
            }
            _390 -= uvec2(uint(_390.y == 0u), 1u);
            if (!(_108 < 3u))
            {
                break;
            }
            _100 = _100 && all(equal(_29[_103]._m2[_108], vec2(0.0)));
            _100 = _100 && all(equal(_29[_103]._m3[_108][0u], vec2(0.0)));
            _100 = _100 && all(equal(_29[_103]._m3[_108][1u], vec2(0.0)));
            _108++;
            continue;
        }
        _103++;
        continue;
    }
    for (;;)
    {
        if (all(equal(uvec2(0u), _439)))
        {
            break;
        }
        _439 -= uvec2(uint(_439.y == 0u), 1u);
        if (!(_109 < 37u))
        {
            break;
        }
        _100 = _100 && (_31[_109] == 0u);
        _100 = _100 && (_33[_109] == 0.0);
        _109++;
        continue;
    }
    barrier();
    if (gl_LocalInvocationIndex == 0u)
    {
        _37 = 42u;
        _39 = uvec3(42u);
        _41 = mat2x3(vec3(42.0), vec3(42.0));
        for (;;)
        {
            if (all(equal(uvec2(0u), _477)))
            {
                break;
            }
            _477 -= uvec2(uint(_477.y == 0u), 1u);
            if (!(_110 < 3u))
            {
                break;
            }
            _111 = 0u;
            for (;;)
            {
                if (all(equal(uvec2(0u), _498)))
                {
                    break;
                }
                _498 -= uvec2(uint(_498.y == 0u), 1u);
                if (!(_111 < 5u))
                {
                    break;
                }
                _112 = 0u;
                for (;;)
                {
                    if (all(equal(uvec2(0u), _519)))
                    {
                        break;
                    }
                    _519 -= uvec2(uint(_519.y == 0u), 1u);
                    if (!(_112 < 3u))
                    {
                        break;
                    }
                    _29[_110]._m0[_111][_112] = vec4(42.0);
                    _112++;
                    continue;
                }
                _111++;
                continue;
            }
            _113 = 0u;
            for (;;)
            {
                if (all(equal(uvec2(0u), _548)))
                {
                    break;
                }
                _548 -= uvec2(uint(_548.y == 0u), 1u);
                if (!(_113 < 19u))
                {
                    break;
                }
                atomicExchange(_29[_110]._m1[_113], 42u);
                _113++;
                continue;
            }
            _110++;
            continue;
        }
        for (;;)
        {
            if (all(equal(uvec2(0u), _576)))
            {
                break;
            }
            _576 -= uvec2(uint(_576.y == 0u), 1u);
            if (!(_114 < 37u))
            {
                break;
            }
            _31[_114] = 42u;
            _33[_114] = 42.0;
            _114++;
            continue;
        }
    }
    barrier();
    if (_37 == 42u)
    {
        _115 = all(equal(_39, uvec3(42u)));
    }
    else
    {
        _115 = false;
    }
    _100 = _100 && _115;
    _100 = _100 && all(equal(_41[0u], vec3(42.0)));
    _100 = _100 && all(equal(_29[2u]._m0[4u][2u], vec4(42.0)));
    uint _626 = atomicAdd(_29[2u]._m1[18u], 0u);
    _100 = _100 && (_626 == 42u);
    _100 = _100 && (_31[36u] == 42u);
    _100 = _100 && (_33[36u] == 42.0);
    uint _642 = ((gl_WorkGroupID.x * 8u) * 2u) + gl_LocalInvocationIndex;
    _43._m0[_642] |= (_100 ? 2u : 1u);
}


//////////////////////////////////
// Entry point: "single" (comp) //
//////////////////////////////////
#version 460
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

struct _16
{
    vec4 _m0[5][3];
    uint _m1[19];
    mat3x2 _m2;
    mat2 _m3[3];
};

struct _17
{
    mat2 _m0;
};

layout(set = 0, binding = 0, std430) buffer _44_43
{
    uint _m0[];
} _43;

shared _16 _29[3];
shared uint _31[37];
shared float _33[37];
shared _17 _35;
shared uint _37;
shared uvec3 _39;
shared mat2x3 _41;

void main()
{
    if (gl_LocalInvocationIndex == 0u)
    {
        _37 = 0u;
    }
    barrier();
    _43._m0[0u] = _37;
}

