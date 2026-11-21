#version 460
#extension GL_EXT_fragment_shader_barycentric : require

layout(location = 0) pervertexEXT in float _23[3];
layout(location = 0) out vec4 _26;
float _9[3];
vec4 _11;

void _14()
{
    _11 = vec4(_9[0], _9[1], _9[2], 1.0);
}

void main()
{
    _9 = float[](0.0, 0.0, 0.0);
    _11 = vec4(0.0);
    _9 = _23;
    _14();
    _26 = _11;
}

