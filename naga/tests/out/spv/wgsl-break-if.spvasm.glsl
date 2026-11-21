#version 460
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

void _6()
{
    uvec2 _22 = uvec2(4294967295u);
    do
    {
        if (all(equal(uvec2(0u), _22)))
        {
            break;
        }
        _22 -= uvec2(uint(_22.y == 0u), 1u);
    } while (!true);
}

void _35(bool _34)
{
    uvec2 _46 = uvec2(4294967295u);
    bool _38 = false;
    bool _40 = false;
    for (;;)
    {
        if (all(equal(uvec2(0u), _46)))
        {
            break;
        }
        _46 -= uvec2(uint(_46.y == 0u), 1u);
        _38 = _34;
        _40 = _34 != _38;
        if (_34 == _40)
        {
            break;
        }
        else
        {
            continue;
        }
    }
}

void _63(bool _62)
{
    uvec2 _71 = uvec2(4294967295u);
    bool _64 = false;
    bool _65 = false;
    do
    {
        if (all(equal(uvec2(0u), _71)))
        {
            break;
        }
        _71 -= uvec2(uint(_71.y == 0u), 1u);
        _64 = _62;
        _65 = _62 != _64;
    } while (!(_62 == _65));
}

void _87()
{
    uvec2 _96 = uvec2(4294967295u);
    uint _89 = 0u;
    do
    {
        if (all(equal(uvec2(0u), _96)))
        {
            break;
        }
        _96 -= uvec2(uint(_96.y == 0u), 1u);
        _89++;
    } while (!(_89 == 5u));
}

void main()
{
    _6();
    _35(false);
    _63(false);
    _87();
}

