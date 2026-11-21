#version 460
#extension GL_KHR_shader_subgroup_ballot : require
#extension GL_KHR_shader_subgroup_basic : require
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

struct _4
{
    uint _m0;
};

layout(set = 0, binding = 0, std430) buffer _7_6
{
    _4 _m0;
} _6;

uint _9;
uint _11;

void _13()
{
    uvec4 _21 = uvec4(0u);
    uint _23 = 0u;
    uint _25 = 0u;
    _21 = uvec4(_9, _11, 3u, 4u);
    _23 = subgroupBallotFindLSB(_21);
    _25 = subgroupBallotFindMSB(_21);
    _6._m0._m0 = _23 + _25;
}

void main()
{
    _9 = 0u;
    _11 = 0u;
    _9 = gl_SubgroupInvocationID;
    _11 = gl_SubgroupSize;
    _13();
}

