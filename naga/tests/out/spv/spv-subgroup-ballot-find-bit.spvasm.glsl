#version 460
#extension GL_KHR_shader_subgroup_ballot : require
#extension GL_KHR_shader_subgroup_basic : require
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

struct _4
{
    uint _m0;
};

layout(set = 0, binding = 0, std430) buffer _9_8
{
    _4 _m0;
} _8;

uint _6;

void _12()
{
    uvec4 _22 = subgroupBallot(_6 != 0u);
    _8._m0._m0 = subgroupBallotFindLSB(_22) + subgroupBallotFindMSB(_22);
}

void main()
{
    _6 = 0u;
    _6 = gl_SubgroupInvocationID;
    _12();
}

