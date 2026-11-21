#version 460
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

void main()
{
    ivec2 _30 = ivec2(0);
    ivec3 _32 = ivec3(0);
    ivec4 _34 = ivec4(0);
    uvec2 _36 = uvec2(0u);
    uvec3 _38 = uvec3(0u);
    uvec4 _40 = uvec4(0u);
    vec2 _42 = vec2(0.0);
    vec3 _44 = vec3(0.0);
    vec4 _46 = vec4(0.0);
    _36 = uvec2(_30);
    _38 = uvec3(_32);
    _40 = uvec4(_34);
    _30 = ivec2(_36);
    _32 = ivec3(_38);
    _34 = ivec4(_40);
    _42 = intBitsToFloat(_30);
    _44 = intBitsToFloat(_32);
    _46 = intBitsToFloat(_34);
}

