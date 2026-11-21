#version 310 es

precision highp float;
precision highp int;

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;


void main() {
    ivec2 i2_;
    ivec3 i3_;
    ivec4 i4_;
    uvec2 u2_;
    uvec3 u3_;
    uvec4 u4_;
    vec2 f2_;
    vec3 f3_;
    vec4 f4_;
    i2_ = ivec2(0);
    i3_ = ivec3(0);
    i4_ = ivec4(0);
    u2_ = uvec2(0u);
    u3_ = uvec3(0u);
    u4_ = uvec4(0u);
    f2_ = vec2(0.0);
    f3_ = vec3(0.0);
    f4_ = vec4(0.0);
    ivec2 _e9 = i2_;
    u2_ = uvec2(_e9);
    ivec3 _e11 = i3_;
    u3_ = uvec3(_e11);
    ivec4 _e13 = i4_;
    u4_ = uvec4(_e13);
    uvec2 _e15 = u2_;
    i2_ = ivec2(_e15);
    uvec3 _e17 = u3_;
    i3_ = ivec3(_e17);
    uvec4 _e19 = u4_;
    i4_ = ivec4(_e19);
    ivec2 _e21 = i2_;
    f2_ = intBitsToFloat(_e21);
    ivec3 _e23 = i3_;
    f3_ = intBitsToFloat(_e23);
    ivec4 _e25 = i4_;
    f4_ = intBitsToFloat(_e25);
    return;
}

