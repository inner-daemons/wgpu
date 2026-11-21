///////////////////////////////////
// Entry point: "ts_main" (task) //
///////////////////////////////////
#version 460
#extension GL_EXT_mesh_shader : require
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

struct _6
{
    vec4 _m0;
    bool _m1;
};

struct _7
{
    vec4 _m0;
    vec4 _m1;
};

struct _10
{
    uvec3 _m0;
    bool _m1;
    vec4 _m2;
};

struct _11
{
    vec4 _m0;
};

struct _16
{
    _7 _m0[3];
    _10 _m1[1];
    uint _m2;
    uint _m3;
};

struct _66
{
    uint _m0;
    uint _m1;
};

taskPayloadSharedEXT _6 _17;
shared float _19;

void _32(bool _31)
{
    _17._m1 = _31;
}

bool _24()
{
    return _17._m1;
}

uvec3 _40()
{
    if (gl_LocalInvocationIndex == 0u)
    {
        _17 = _6(vec4(0.0), false);
        _19 = 0.0;
    }
    barrier();
    _19 = 1.0;
    _17._m0 = vec4(1.0, 1.0, 0.0, 1.0);
    _32(true);
    _17._m1 = _24();
    return uvec3(1u);
}

void main()
{
    uvec3 _63 = _40();
    barrier();
    _66 _73;
    umulExtended(_63.x, _63.y, _73._m1, _73._m0);
    _66 _75;
    umulExtended(_73._m0, _63.z, _75._m1, _75._m0);
    uvec3 _88 = ((((((_75._m0 > 1024u) || (_63.x > 256u)) || (_63.y > 256u)) || (_63.z > 256u)) || (_73._m1 != 0u)) || (_75._m1 != 0u)) ? uvec3(0u) : _63;
    EmitMeshTasksEXT(_88.x, _88.y, _88.z);
}


////////////////////////////////////////
// Entry point: "ts_divergent" (task) //
////////////////////////////////////////
#version 460
#extension GL_EXT_mesh_shader : require
layout(local_size_x = 2, local_size_y = 1, local_size_z = 1) in;

struct _6
{
    vec4 _m0;
    bool _m1;
};

struct _7
{
    vec4 _m0;
    vec4 _m1;
};

struct _10
{
    uvec3 _m0;
    bool _m1;
    vec4 _m2;
};

struct _11
{
    vec4 _m0;
};

struct _16
{
    _7 _m0[3];
    _10 _m1[1];
    uint _m2;
    uint _m3;
};

struct _66
{
    uint _m0;
    uint _m1;
};

taskPayloadSharedEXT _6 _17;

uvec3 _98()
{
    if (gl_LocalInvocationIndex == 0u)
    {
        _17 = _6(vec4(0.0), false);
    }
    barrier();
    if (gl_LocalInvocationID.x == 0u)
    {
        _17._m0 = vec4(1.0, 1.0, 0.0, 1.0);
        _17._m1 = true;
        return uvec3(1u);
    }
    return uvec3(2u);
}

void main()
{
    uvec3 _112 = _98();
    barrier();
    _66 _119;
    umulExtended(_112.x, _112.y, _119._m1, _119._m0);
    _66 _121;
    umulExtended(_119._m0, _112.z, _121._m1, _121._m0);
    uvec3 _134 = ((((((_121._m0 > 1024u) || (_112.x > 256u)) || (_112.y > 256u)) || (_112.z > 256u)) || (_119._m1 != 0u)) || (_121._m1 != 0u)) ? uvec3(0u) : _112;
    EmitMeshTasksEXT(_134.x, _134.y, _134.z);
}


///////////////////////////////////
// Entry point: "ms_main" (mesh) //
///////////////////////////////////
#version 460
#extension GL_EXT_mesh_shader : require
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(max_vertices = 3, max_primitives = 1, triangles) out;

struct _6
{
    vec4 _m0;
    bool _m1;
};

struct _7
{
    vec4 _m0;
    vec4 _m1;
};

struct _10
{
    uvec3 _m0;
    bool _m1;
    vec4 _m2;
};

struct _11
{
    vec4 _m0;
};

struct _16
{
    _7 _m0[3];
    _10 _m1[1];
    uint _m2;
    uint _m3;
};

struct _66
{
    uint _m0;
    uint _m1;
};

layout(location = 0) out vec4 _151[3];
layout(location = 1) perprimitiveEXT out vec4 _157[1];
taskPayloadSharedEXT _6 _17;
shared float _19;
shared _16 _21;

bool _24()
{
    return _17._m1;
}

void _158()
{
    uvec2 _183 = uvec2(4294967295u);
    uint _170 = gl_LocalInvocationIndex;
    for (;;)
    {
        if (all(equal(uvec2(0u), _183)))
        {
            break;
        }
        _183 -= uvec2(uint(_183.y == 0u), 1u);
        if (_170 >= 3u)
        {
            break;
        }
        _21._m0[_170] = _7(vec4(0.0), vec4(0.0));
        _170++;
        continue;
    }
    _21._m1[gl_LocalInvocationIndex] = _10(uvec3(0u), false, vec4(0.0));
    if (gl_LocalInvocationIndex == 0u)
    {
        _19 = 0.0;
        _21._m2 = 0u;
        _21._m3 = 0u;
    }
    barrier();
    _21._m2 = 3u;
    _21._m3 = 1u;
    _19 = 2.0;
    _21._m0[0u]._m0 = vec4(0.0, 1.0, 0.0, 1.0);
    _21._m0[0u]._m1 = vec4(0.0, 1.0, 0.0, 1.0) * _17._m0;
    _21._m0[1u]._m0 = vec4(-1.0, -1.0, 0.0, 1.0);
    _21._m0[1u]._m1 = vec4(0.0, 0.0, 1.0, 1.0) * _17._m0;
    _21._m0[2u]._m0 = vec4(1.0, -1.0, 0.0, 1.0);
    _21._m0[2u]._m1 = vec4(1.0, 0.0, 0.0, 1.0) * _17._m0;
    _21._m1[0u]._m0 = uvec3(0u, 1u, 2u);
    _21._m1[0u]._m1 = !_24();
    _21._m1[0u]._m2 = vec4(1.0, 0.0, 1.0, 1.0);
}

void main()
{
    _158();
    barrier();
    uint _244 = min(_21._m2, 3u);
    uint _247 = min(_21._m3, 1u);
    SetMeshOutputsEXT(_244, _247);
    for (uint _238 = gl_LocalInvocationIndex; _238 < _244; _238++)
    {
        gl_MeshVerticesEXT[_238].gl_Position = _21._m0[_238]._m0;
        _151[_238] = _21._m0[_238]._m1;
    }
    for (uint _239 = gl_LocalInvocationIndex; _239 < _247; _239++)
    {
        gl_PrimitiveTriangleIndicesEXT[_239] = _21._m1[_239]._m0;
        gl_MeshPrimitivesEXT[_239].gl_CullPrimitiveEXT = _21._m1[_239]._m1;
        _157[_239] = _21._m1[_239]._m2;
    }
}


////////////////////////////////////
// Entry point: "ms_no_ts" (mesh) //
////////////////////////////////////
#version 460
#extension GL_EXT_mesh_shader : require
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(max_vertices = 3, max_primitives = 1, triangles) out;

struct _6
{
    vec4 _m0;
    bool _m1;
};

struct _7
{
    vec4 _m0;
    vec4 _m1;
};

struct _10
{
    uvec3 _m0;
    bool _m1;
    vec4 _m2;
};

struct _11
{
    vec4 _m0;
};

struct _16
{
    _7 _m0[3];
    _10 _m1[1];
    uint _m2;
    uint _m3;
};

struct _66
{
    uint _m0;
    uint _m1;
};

layout(location = 0) out vec4 _301[3];
layout(location = 1) perprimitiveEXT out vec4 _307[1];
shared float _19;
shared _16 _21;

void _308()
{
    uvec2 _316 = uvec2(4294967295u);
    uint _310 = gl_LocalInvocationIndex;
    for (;;)
    {
        if (all(equal(uvec2(0u), _316)))
        {
            break;
        }
        _316 -= uvec2(uint(_316.y == 0u), 1u);
        if (_310 >= 3u)
        {
            break;
        }
        _21._m0[_310] = _7(vec4(0.0), vec4(0.0));
        _310++;
        continue;
    }
    _21._m1[gl_LocalInvocationIndex] = _10(uvec3(0u), false, vec4(0.0));
    if (gl_LocalInvocationIndex == 0u)
    {
        _19 = 0.0;
        _21._m2 = 0u;
        _21._m3 = 0u;
    }
    barrier();
    _21._m2 = 3u;
    _21._m3 = 1u;
    _19 = 2.0;
    _21._m0[0u]._m0 = vec4(0.0, 1.0, 0.0, 1.0);
    _21._m0[0u]._m1 = vec4(0.0, 1.0, 0.0, 1.0);
    _21._m0[1u]._m0 = vec4(-1.0, -1.0, 0.0, 1.0);
    _21._m0[1u]._m1 = vec4(0.0, 0.0, 1.0, 1.0);
    _21._m0[2u]._m0 = vec4(1.0, -1.0, 0.0, 1.0);
    _21._m0[2u]._m1 = vec4(1.0, 0.0, 0.0, 1.0);
    _21._m1[0u]._m0 = uvec3(0u, 1u, 2u);
    _21._m1[0u]._m1 = false;
    _21._m1[0u]._m2 = vec4(1.0, 0.0, 1.0, 1.0);
}

void main()
{
    _308();
    barrier();
    uint _358 = min(_21._m2, 3u);
    uint _361 = min(_21._m3, 1u);
    SetMeshOutputsEXT(_358, _361);
    for (uint _352 = gl_LocalInvocationIndex; _352 < _358; _352++)
    {
        gl_MeshVerticesEXT[_352].gl_Position = _21._m0[_352]._m0;
        _301[_352] = _21._m0[_352]._m1;
    }
    for (uint _353 = gl_LocalInvocationIndex; _353 < _361; _353++)
    {
        gl_PrimitiveTriangleIndicesEXT[_353] = _21._m1[_353]._m0;
        gl_MeshPrimitivesEXT[_353].gl_CullPrimitiveEXT = _21._m1[_353]._m1;
        _307[_353] = _21._m1[_353]._m2;
    }
}


////////////////////////////////////////
// Entry point: "ms_divergent" (mesh) //
////////////////////////////////////////
#version 460
#extension GL_EXT_mesh_shader : require
layout(local_size_x = 2, local_size_y = 1, local_size_z = 1) in;
layout(max_vertices = 3, max_primitives = 1, triangles) out;

struct _6
{
    vec4 _m0;
    bool _m1;
};

struct _7
{
    vec4 _m0;
    vec4 _m1;
};

struct _10
{
    uvec3 _m0;
    bool _m1;
    vec4 _m2;
};

struct _11
{
    vec4 _m0;
};

struct _16
{
    _7 _m0[3];
    _10 _m1[1];
    uint _m2;
    uint _m3;
};

struct _66
{
    uint _m0;
    uint _m1;
};

layout(location = 0) out vec4 _414[3];
layout(location = 1) perprimitiveEXT out vec4 _420[1];
shared float _19;
shared _16 _21;

void _421()
{
    uvec2 _428 = uvec2(4294967295u);
    uint _422 = gl_LocalInvocationIndex;
    for (;;)
    {
        if (all(equal(uvec2(0u), _428)))
        {
            break;
        }
        _428 -= uvec2(uint(_428.y == 0u), 1u);
        if (_422 >= 3u)
        {
            break;
        }
        _21._m0[_422] = _7(vec4(0.0), vec4(0.0));
        _422 += 2u;
        continue;
    }
    if (gl_LocalInvocationIndex < 1u)
    {
        _21._m1[gl_LocalInvocationIndex] = _10(uvec3(0u), false, vec4(0.0));
    }
    if (gl_LocalInvocationIndex == 0u)
    {
        _19 = 0.0;
        _21._m2 = 0u;
        _21._m3 = 0u;
    }
    barrier();
    if (gl_LocalInvocationID.x == 0u)
    {
        _21._m2 = 3u;
        _21._m3 = 1u;
        _19 = 2.0;
        _21._m0[0u]._m0 = vec4(0.0, 1.0, 0.0, 1.0);
        _21._m0[0u]._m1 = vec4(0.0, 1.0, 0.0, 1.0);
        _21._m0[1u]._m0 = vec4(-1.0, -1.0, 0.0, 1.0);
        _21._m0[1u]._m1 = vec4(0.0, 0.0, 1.0, 1.0);
        _21._m0[2u]._m0 = vec4(1.0, -1.0, 0.0, 1.0);
        _21._m0[2u]._m1 = vec4(1.0, 0.0, 0.0, 1.0);
        _21._m1[0u]._m0 = uvec3(0u, 1u, 2u);
        _21._m1[0u]._m1 = false;
        _21._m1[0u]._m2 = vec4(1.0, 0.0, 1.0, 1.0);
        return;
    }
    else
    {
        return;
    }
}

void main()
{
    _421();
    barrier();
    uint _478 = min(_21._m2, 3u);
    uint _481 = min(_21._m3, 1u);
    SetMeshOutputsEXT(_478, _481);
    for (uint _472 = gl_LocalInvocationIndex; _472 < _478; _472 += 2u)
    {
        gl_MeshVerticesEXT[_472].gl_Position = _21._m0[_472]._m0;
        _414[_472] = _21._m0[_472]._m1;
    }
    for (uint _473 = gl_LocalInvocationIndex; _473 < _481; _473 += 2u)
    {
        gl_PrimitiveTriangleIndicesEXT[_473] = _21._m1[_473]._m0;
        gl_MeshPrimitivesEXT[_473].gl_CullPrimitiveEXT = _21._m1[_473]._m1;
        _420[_473] = _21._m1[_473]._m2;
    }
}


///////////////////////////////////
// Entry point: "fs_main" (frag) //
///////////////////////////////////
#version 460
#extension GL_EXT_mesh_shader : require

struct _6
{
    vec4 _m0;
    bool _m1;
};

struct _7
{
    vec4 _m0;
    vec4 _m1;
};

struct _10
{
    uvec3 _m0;
    bool _m1;
    vec4 _m2;
};

struct _11
{
    vec4 _m0;
};

struct _16
{
    _7 _m0[3];
    _10 _m1[1];
    uint _m2;
    uint _m3;
};

struct _66
{
    uint _m0;
    uint _m1;
};

layout(location = 0) in vec4 _524;
layout(location = 1) perprimitiveEXT in vec4 _527;
layout(location = 0) out vec4 _529;

void main()
{
    _529 = _7(gl_FragCoord, _524)._m1 * _11(_527)._m0;
}

