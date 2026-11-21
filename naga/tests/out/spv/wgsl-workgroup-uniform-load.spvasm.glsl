#version 460
layout(local_size_x = 4, local_size_y = 1, local_size_z = 1) in;

shared int _8[128];

void main()
{
    uvec2 _38 = uvec2(4294967295u);
    uint _22 = gl_LocalInvocationIndex;
    for (;;)
    {
        if (all(equal(uvec2(0u), _38)))
        {
            break;
        }
        _38 -= uvec2(uint(_38.y == 0u), 1u);
        if (_22 >= 128u)
        {
            break;
        }
        _8[_22] = 0;
        _22 += 4u;
        continue;
    }
    barrier();
    barrier();
    int _60 = _8[gl_WorkGroupID.x];
    barrier();
    if (_60 > 10)
    {
        barrier();
        return;
    }
    else
    {
        return;
    }
}

