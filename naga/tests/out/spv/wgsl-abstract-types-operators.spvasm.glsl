#version 460
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

shared uint _17[64];

void _20()
{
    float _32 = 42.0;
    int _34 = 43;
    uint _36 = 44u;
    float _38 = 3.0;
    float _39 = 3.0;
    float _40 = 0.0;
    float _41 = 3.0;
    float _42 = 3.0;
    float _43 = 0.0;
    float _44 = 0.0;
    float _45 = 0.0;
    float _46 = 0.0;
    int _47 = 3;
    int _48 = 0;
    int _49 = 0;
    int _50 = 0;
    uint _51 = 3u;
    uint _52 = 0u;
    uint _53 = 0u;
    uint _54 = 0u;
    int _55 = 0;
    int _56 = 0;
    _40 = 1.0 + _32;
    _43 = 1.0 + _32;
    _44 = _32 + 2.0;
    _45 = _32 + 2.0;
    _46 = _32 + _32;
    _48 = 1 + _34;
    _49 = _34 + 2;
    _50 = _34 + _34;
    _52 = 1u + _36;
    _53 = _36 + 2u;
    _54 = _36 + _36;
    _55 = 1 << int(_36);
    _56 = 1 << int(_36);
}

void _88()
{
}

void _93()
{
    int _95 = 1 - 1;
}

void main()
{
    uvec2 _117 = uvec2(4294967295u);
    uint _104 = gl_LocalInvocationIndex;
    for (;;)
    {
        if (all(equal(uvec2(0u), _117)))
        {
            break;
        }
        _117 -= uvec2(uint(_117.y == 0u), 1u);
        if (_104 >= 64u)
        {
            break;
        }
        _17[_104] = 0u;
        _104++;
        continue;
    }
    barrier();
    _20();
    _88();
    _93();
}

