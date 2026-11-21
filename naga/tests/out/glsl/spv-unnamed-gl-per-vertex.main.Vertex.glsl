#version 310 es

precision highp float;
precision highp int;

struct type_4 {
    vec4 member;
    float member_1;
    float member_2[1];
    float member_3[1];
};
type_4 global;

int global_1;


void function() {
    int _e9 = global_1;
    global.member = vec4(((_e9 == 0) ? -4.0 : 1.0), ((_e9 == 2) ? 4.0 : -1.0), 0.0, 1.0);
    return;
}

void main() {
    uint param = uint(gl_VertexID);
    global = type_4(vec4(0.0, 0.0, 0.0, 1.0), 1.0, float[1](0.0), float[1](0.0));
    global_1 = 0;
    global_1 = int(param);
    function();
    float _e6 = global.member.y;
    global.member.y = -(_e6);
    vec4 _e8 = global.member;
    gl_Position = _e8;
    gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);
    return;
}

