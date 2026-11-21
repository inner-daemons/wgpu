#version 460

struct _5
{
    float _m0;
    vec3 _m1;
};

const vec3 _91[12] = vec3[](vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0));

_5 _12(vec3 _10[12], uint _11)
{
    uvec2 _38 = uvec2(4294967295u);
    uvec2 _66 = uvec2(4294967295u);
    vec3 _22 = vec3(0.0);
    uint _24 = 0u;
    vec3 _25 = vec3(0.0);
    vec3 _57[12];
    uint _20 = 0u;
    for (;;)
    {
        if (all(equal(uvec2(0u), _38)))
        {
            break;
        }
        _38 -= uvec2(uint(_38.y == 0u), 1u);
        if (!(_20 < _11))
        {
            break;
        }
        _57 = _10;
        _22 = _57[_20];
        _20++;
        continue;
    }
    for (;;)
    {
        if (all(equal(uvec2(0u), _66)))
        {
            break;
        }
        _66 -= uvec2(uint(_66.y == 0u), 1u);
        if (!(_24 < _11))
        {
            break;
        }
        _57 = _10;
        _25 = _57[_24];
        _24++;
        continue;
    }
    return _5(0.0, vec3(0.0));
}

void main()
{
}

