#version 460
layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

struct _7
{
    uint _m0;
    int _m1[2];
};

shared uint _10;
shared int _12;
shared _7 _14;

void main()
{
    bool _34 = false;
    bool _36 = false;
    bool _37 = false;
    if (gl_LocalInvocationIndex < 2u)
    {
        atomicExchange(_14._m1[gl_LocalInvocationIndex], 0);
    }
    if (gl_LocalInvocationIndex == 0u)
    {
        atomicExchange(_10, 0u);
        atomicExchange(_12, 0);
        atomicExchange(_14._m0, 0u);
    }
    barrier();
    uint _56 = atomicOr(_10, uint((gl_WorkGroupID.x + (gl_WorkGroupID.y * 32768u)) >= 64u));
    int _57 = atomicAdd(_12, 1);
    atomicExchange(_14._m0, 1u);
    int _59 = atomicAdd(_14._m1[0u], 1);
    barrier();
    barrier();
    uint _61 = atomicAdd(_10, 0u);
    barrier();
    barrier();
    int _62 = atomicAdd(_12, 0);
    barrier();
    barrier();
    uint _64 = atomicAdd(_14._m0, 0u);
    barrier();
    barrier();
    int _66 = atomicAdd(_14._m1[0u], 0);
    barrier();
    if (_61 == 0u)
    {
        _34 = _62 > 0;
    }
    else
    {
        _34 = false;
    }
    if (_34)
    {
        _36 = _64 > 0u;
    }
    else
    {
        _36 = false;
    }
    if (_36)
    {
        _37 = _66 > 0;
    }
    else
    {
        _37 = false;
    }
    if (_37)
    {
        return;
    }
    else
    {
        return;
    }
}

