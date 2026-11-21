#version 460

out float gl_ClipDistance[1];

struct _8
{
    vec4 _m0;
    float _m1[1];
};

void main()
{
    _8 _18 = _8(vec4(0.0), float[](0.0));
    _18._m1[0u] = 0.5;
    gl_Position = _18._m0;
    gl_ClipDistance = _18._m1;
}

