#version 460
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

vec4 _26()
{
    vec4 _36 = mix(vec4(0.0), vec4(1.0), vec4(0.5));
    return ((((vec4(ivec4(int(true)) + ivec4(0)) + vec4(bvec4(true))) + _36) + mix(vec4(0.0), vec4(1.0), vec4(0.100000001490116119384765625))) + vec4(intBitsToFloat(1))) + intBitsToFloat(ivec4(1));
}

ivec4 _50(ivec4 _52, ivec4 _53)
{
    bvec4 _56 = equal(_53, ivec4(0));
    bvec4 _61 = equal(_52, ivec4(int(0x80000000)));
    bvec4 _62 = equal(_53, ivec4(-1));
    bvec4 _63 = bvec4(_61.x && _62.x, _61.y && _62.y, _61.z && _62.z, _61.w && _62.w);
    ivec4 _66 = mix(_53, ivec4(1), bvec4(_56.x || _63.x, _56.y || _63.y, _56.z || _63.z, _56.w || _63.w));
    return _52 - ((_52 / _66) * _66);
}

vec4 _73(float _71, int _72)
{
    return (((vec2(2.0) + vec2(_71)) - vec2(4.0)) / vec2(8.0)).xyxy + vec4(_50(ivec4(_72), ivec4(2)));
}

vec2 _94()
{
    vec2 _99 = vec2(2.0);
    _99 += vec2(1.0);
    _99 -= vec2(3.0);
    _99 /= vec2(4.0);
    return _99;
}

vec3 _111(vec3 _110)
{
    return vec3(notEqual(_110, vec3(0.0)));
}

bool _120()
{
    return true;
}

bool _124()
{
    return false;
}

bool _127()
{
    return true;
}

bool _130()
{
    return false;
}

void _133()
{
    bool _141 = false;
    bool _143 = false;
    bool _144 = false;
    bool _145 = false;
    bool _146 = false;
    bool _147 = false;
    bool _148 = false;
    if (!true)
    {
        _141 = false;
    }
    else
    {
        _141 = true;
    }
    if (true)
    {
        _143 = false;
    }
    else
    {
        _143 = false;
    }
    if (!false)
    {
        _144 = false;
    }
    else
    {
        _144 = true;
    }
    if (!_120())
    {
        _145 = _124();
    }
    else
    {
        _145 = true;
    }
    if (_145)
    {
        if (!_127())
        {
            _147 = _130();
        }
        else
        {
            _147 = true;
        }
        _146 = _147;
    }
    else
    {
        _146 = false;
    }
    if (false)
    {
        _148 = _124();
    }
    else
    {
        _148 = true;
    }
}

int _194(int _196, int _197)
{
    return _196 / (((_197 == 0) || ((_196 == int(0x80000000)) && (_197 == (-1)))) ? 1 : _197);
}

uint _206(uint _208, uint _209)
{
    return _208 / ((_209 == 0u) ? 1u : _209);
}

ivec2 _217(ivec2 _219, ivec2 _220)
{
    bvec2 _223 = equal(_220, ivec2(0));
    bvec2 _226 = equal(_219, ivec2(int(0x80000000)));
    bvec2 _227 = equal(_220, ivec2(-1));
    bvec2 _228 = bvec2(_226.x && _227.x, _226.y && _227.y);
    return _219 / mix(_220, ivec2(1), bvec2(_223.x || _228.x, _223.y || _228.y));
}

uvec3 _233(uvec3 _235, uvec3 _236)
{
    return _235 / mix(_236, uvec3(1u), equal(_236, uvec3(0u)));
}

int _243(int _244, int _245)
{
    int _252 = ((_245 == 0) || ((_244 == int(0x80000000)) && (_245 == (-1)))) ? 1 : _245;
    return _244 - ((_244 / _252) * _252);
}

uint _256(uint _257, uint _258)
{
    return _257 % ((_258 == 0u) ? 1u : _258);
}

ivec2 _263(ivec2 _264, ivec2 _265)
{
    bvec2 _267 = equal(_265, ivec2(0));
    bvec2 _268 = equal(_264, ivec2(int(0x80000000)));
    bvec2 _269 = equal(_265, ivec2(-1));
    bvec2 _270 = bvec2(_268.x && _269.x, _268.y && _269.y);
    ivec2 _272 = mix(_265, ivec2(1), bvec2(_267.x || _270.x, _267.y || _270.y));
    return _264 - ((_264 / _272) * _272);
}

uvec3 _276(uvec3 _277, uvec3 _278)
{
    return _277 % mix(_278, uvec3(1u), equal(_278, uvec3(0u)));
}

uvec2 _284(uvec2 _286, uvec2 _287)
{
    return _286 / mix(_287, uvec2(1u), equal(_287, uvec2(0u)));
}

uvec2 _294(uvec2 _295, uvec2 _296)
{
    return _295 % mix(_296, uvec2(1u), equal(_296, uvec2(0u)));
}

void _302()
{
    int _314 = 0;
    int _316 = 0;
    float _318 = -1.0;
    ivec2 _319 = -ivec2(1);
    vec2 _320 = -vec2(1.0);
    int _321 = 2 + 1;
    uint _322 = 2u + 1u;
    float _323 = 2.0 + 1.0;
    ivec2 _324 = ivec2(2) + ivec2(1);
    uvec3 _325 = uvec3(2u) + uvec3(1u);
    vec4 _326 = vec4(2.0) + vec4(1.0);
    int _327 = 2 - 1;
    uint _328 = 2u - 1u;
    float _329 = 2.0 - 1.0;
    ivec2 _330 = ivec2(2) - ivec2(1);
    uvec3 _331 = uvec3(2u) - uvec3(1u);
    vec4 _332 = vec4(2.0) - vec4(1.0);
    int _333 = 2 * 1;
    uint _334 = 2u * 1u;
    float _335 = 2.0 * 1.0;
    ivec2 _336 = ivec2(2) * ivec2(1);
    uvec3 _337 = uvec3(2u) * uvec3(1u);
    vec4 _338 = vec4(2.0) * vec4(1.0);
    float _341 = 2.0 / 1.0;
    vec4 _344 = vec4(2.0) / vec4(1.0);
    float _347 = 2.0 - 1.0 * trunc(2.0 / 1.0);
    vec4 _350 = vec4(2.0) - vec4(1.0) * trunc(vec4(2.0) / vec4(1.0));
    ivec2 _353 = ivec2(2) + ivec2(1);
    ivec2 _354 = ivec2(2) + ivec2(1);
    uvec2 _355 = uvec2(2u) + uvec2(1u);
    uvec2 _356 = uvec2(2u) + uvec2(1u);
    vec2 _357 = vec2(2.0) + vec2(1.0);
    vec2 _358 = vec2(2.0) + vec2(1.0);
    ivec2 _359 = ivec2(2) - ivec2(1);
    ivec2 _360 = ivec2(2) - ivec2(1);
    uvec2 _361 = uvec2(2u) - uvec2(1u);
    uvec2 _362 = uvec2(2u) - uvec2(1u);
    vec2 _363 = vec2(2.0) - vec2(1.0);
    vec2 _364 = vec2(2.0) - vec2(1.0);
    vec2 _373 = vec2(2.0) * 1.0;
    vec2 _374 = vec2(1.0) * 2.0;
    vec2 _379 = vec2(2.0) / vec2(1.0);
    vec2 _380 = vec2(2.0) / vec2(1.0);
    vec2 _385 = vec2(2.0) - vec2(1.0) * trunc(vec2(2.0) / vec2(1.0));
    vec2 _386 = vec2(2.0) - vec2(1.0) * trunc(vec2(2.0) / vec2(1.0));
    mat3 _387 = mat3(vec3(0.0), vec3(0.0), vec3(0.0)) * 1.0;
    mat3 _388 = mat3(vec3(0.0), vec3(0.0), vec3(0.0)) * 2.0;
    vec3 _389 = mat4x3(vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0)) * vec4(1.0);
    vec4 _390 = vec3(2.0) * mat4x3(vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0));
    _316 = _314 + int(0x80000000);
}

void _394()
{
    int _396 = ~1;
    uint _397 = ~1u;
    ivec2 _398 = ~ivec2(1);
    uvec3 _399 = ~uvec3(1u);
    int _400 = 2 | 1;
    uint _401 = 2u | 1u;
    ivec2 _402 = ivec2(2) | ivec2(1);
    uvec3 _403 = uvec3(2u) | uvec3(1u);
    int _404 = 2 & 1;
    uint _405 = 2u & 1u;
    ivec2 _406 = ivec2(2) & ivec2(1);
    uvec3 _407 = uvec3(2u) & uvec3(1u);
    int _408 = 2 ^ 1;
    uint _409 = 2u ^ 1u;
    ivec2 _410 = ivec2(2) ^ ivec2(1);
    uvec3 _411 = uvec3(2u) ^ uvec3(1u);
    int _412 = 2 << int(1u);
    uint _413 = 2u << 1u;
    ivec2 _414 = ivec2(2) << ivec2(uvec2(1u));
    uvec3 _415 = uvec3(2u) << uvec3(1u);
    int _416 = 2 >> int(1u);
    uint _417 = 2u >> 1u;
    ivec2 _418 = ivec2(2) >> ivec2(uvec2(1u));
    uvec3 _419 = uvec3(2u) >> uvec3(1u);
}

void _421()
{
}

void _460()
{
    int _462 = 0;
    ivec3 _463 = ivec3(0);
    _462 = 1;
    _462++;
    _462--;
    _462 *= _462;
    _462 = _194(_462, _462);
    _462 = _243(_462, 1);
    _462 &= 0;
    _462 |= 0;
    _462 ^= 0;
    _462 = _462 << int(2u);
    _462 = _462 >> int(1u);
    _462++;
    _462--;
    _463.y++;
    _463.y--;
}

void _501()
{
    int _503 = -1;
    int _504 = -1;
    int _506 = -1;
    int _508 = -1;
    int _510 = -1;
    int _513 = -1;
    int _517 = -1;
    int _522 = -1;
    float _527 = -1.0;
    float _528 = -1.0;
    float _530 = -1.0;
    float _532 = -1.0;
    float _534 = -1.0;
    float _537 = -1.0;
    float _541 = -1.0;
    float _546 = -1.0;
}

void main()
{
    _133();
    _302();
    _394();
    _421();
    _460();
    _501();
}

