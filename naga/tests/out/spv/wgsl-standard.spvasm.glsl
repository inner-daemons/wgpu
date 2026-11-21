#version 460

layout(location = 0) out vec4 _15;

bool _7()
{
    return true;
}

void main()
{
    vec4 _20 = vec4(0.0);
    vec4 _22 = vec4(0.0);
    vec4 _23 = vec4(0.0);
    _20 = dFdxCoarse(gl_FragCoord);
    _22 = dFdyCoarse(gl_FragCoord);
    _23 = fwidthCoarse(gl_FragCoord);
    _20 = dFdxFine(gl_FragCoord);
    _22 = dFdyFine(gl_FragCoord);
    _23 = fwidthFine(gl_FragCoord);
    _20 = dFdx(gl_FragCoord);
    _22 = dFdy(gl_FragCoord);
    _23 = fwidth(gl_FragCoord);
    _15 = (_20 + _22) * _23;
}

