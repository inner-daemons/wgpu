#version 300 es

precision highp float;
precision highp int;


void main() {
    int i;
    ivec2 i2_;
    ivec3 i3_;
    ivec4 i4_;
    uint u;
    uvec2 u2_;
    uvec3 u3_;
    uvec4 u4_;
    vec2 f2_;
    vec4 f4_;
    i = 0;
    i2_ = ivec2(0);
    i3_ = ivec3(0);
    i4_ = ivec4(0);
    u = 0u;
    u2_ = uvec2(0u);
    u3_ = uvec3(0u);
    u4_ = uvec4(0u);
    f2_ = vec2(0.0);
    f4_ = vec4(0.0);
    ivec4 _e7 = i4_;
    u = uint((_e7[0] & 0xFF) | ((_e7[1] & 0xFF) << 8) | ((_e7[2] & 0xFF) << 16) | ((_e7[3] & 0xFF) << 24));
    uvec4 _e9 = u4_;
    u = (_e9[0] & 0xFFu) | ((_e9[1] & 0xFFu) << 8) | ((_e9[2] & 0xFFu) << 16) | ((_e9[3] & 0xFFu) << 24);
    uint _e11 = u;
    f4_ = (vec4(ivec4(_e11 << 24, _e11 << 16, _e11 << 8, _e11) >> 24) / 127.0);
    uint _e13 = u;
    f4_ = (vec4(_e13 & 0xFFu, _e13 >> 8 & 0xFFu, _e13 >> 16 & 0xFFu, _e13 >> 24) / 255.0);
    uint _e15 = u;
    f2_ = unpackSnorm2x16(_e15);
    uint _e17 = u;
    f2_ = unpackUnorm2x16(_e17);
    return;
}

