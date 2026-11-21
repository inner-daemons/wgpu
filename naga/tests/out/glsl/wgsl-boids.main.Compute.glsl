#version 310 es

precision highp float;
precision highp int;

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

struct Particle {
    vec2 pos;
    vec2 vel;
};
struct SimParams {
    float deltaT;
    float rule1Distance;
    float rule2Distance;
    float rule3Distance;
    float rule1Scale;
    float rule2Scale;
    float rule3Scale;
};
const uint NUM_PARTICLES = 1500u;

layout(std140) uniform SimParams_block_0Compute { SimParams _group_0_binding_0_cs; };

layout(std430) readonly buffer Particles_block_1Compute {
    Particle particles[];
} _group_0_binding_1_cs;

layout(std430) buffer Particles_block_2Compute {
    Particle particles[];
} _group_0_binding_2_cs;


void main() {
    uvec3 global_invocation_id = gl_GlobalInvocationID;
    vec2 vPos;
    vec2 vVel;
    vec2 cMass;
    vec2 cVel;
    vec2 colVel;
    int cMassCount;
    int cVelCount;
    vec2 pos;
    vec2 vel;
    uint i;
    vPos = vec2(0.0);
    vVel = vec2(0.0);
    cMass = vec2(0.0, 0.0);
    cVel = vec2(0.0, 0.0);
    colVel = vec2(0.0, 0.0);
    cMassCount = 0;
    cVelCount = 0;
    pos = vec2(0.0);
    vel = vec2(0.0);
    i = 0u;
    uint index = global_invocation_id.x;
    if ((index >= NUM_PARTICLES)) {
        return;
    }
    vec2 _e8 = _group_0_binding_1_cs.particles[index].pos;
    vPos = _e8;
    vec2 _e14 = _group_0_binding_1_cs.particles[index].vel;
    vVel = _e14;
    bool loop_init = true;
    while(true) {
        if (!loop_init) {
            uint _e82 = i;
            i = (_e82 + 1u);
        }
        loop_init = false;
        uint _e27 = i;
        if ((_e27 >= NUM_PARTICLES)) {
            break;
        }
        uint _e30 = i;
        if ((_e30 == index)) {
            continue;
        }
        uint _e34 = i;
        vec2 _e37 = _group_0_binding_1_cs.particles[_e34].pos;
        pos = _e37;
        uint _e40 = i;
        vec2 _e43 = _group_0_binding_1_cs.particles[_e40].vel;
        vel = _e43;
        vec2 _e44 = pos;
        vec2 _e45 = vPos;
        float _e49 = _group_0_binding_0_cs.rule1Distance;
        if ((distance(_e44, _e45) < _e49)) {
            vec2 _e51 = cMass;
            vec2 _e52 = pos;
            cMass = (_e51 + _e52);
            int _e54 = cMassCount;
            cMassCount = (_e54 + 1);
        }
        vec2 _e57 = pos;
        vec2 _e58 = vPos;
        float _e62 = _group_0_binding_0_cs.rule2Distance;
        if ((distance(_e57, _e58) < _e62)) {
            vec2 _e64 = colVel;
            vec2 _e65 = pos;
            vec2 _e66 = vPos;
            colVel = (_e64 - (_e65 - _e66));
        }
        vec2 _e69 = pos;
        vec2 _e70 = vPos;
        float _e74 = _group_0_binding_0_cs.rule3Distance;
        if ((distance(_e69, _e70) < _e74)) {
            vec2 _e76 = cVel;
            vec2 _e77 = vel;
            cVel = (_e76 + _e77);
            int _e79 = cVelCount;
            cVelCount = (_e79 + 1);
        }
    }
    int _e85 = cMassCount;
    if ((_e85 > 0)) {
        vec2 _e88 = cMass;
        int _e89 = cMassCount;
        vec2 _e93 = vPos;
        cMass = ((_e88 / vec2(float(_e89))) - _e93);
    }
    int _e95 = cVelCount;
    if ((_e95 > 0)) {
        vec2 _e98 = cVel;
        int _e99 = cVelCount;
        cVel = (_e98 / vec2(float(_e99)));
    }
    vec2 _e103 = vVel;
    vec2 _e104 = cMass;
    float _e107 = _group_0_binding_0_cs.rule1Scale;
    vec2 _e110 = colVel;
    float _e113 = _group_0_binding_0_cs.rule2Scale;
    vec2 _e116 = cVel;
    float _e119 = _group_0_binding_0_cs.rule3Scale;
    vVel = (((_e103 + (_e104 * _e107)) + (_e110 * _e113)) + (_e116 * _e119));
    vec2 _e122 = vVel;
    vec2 _e124 = vVel;
    vVel = (normalize(_e122) * clamp(length(_e124), 0.0, 0.1));
    vec2 _e130 = vPos;
    vec2 _e131 = vVel;
    float _e134 = _group_0_binding_0_cs.deltaT;
    vPos = (_e130 + (_e131 * _e134));
    float _e138 = vPos.x;
    if ((_e138 < -1.0)) {
        vPos.x = 1.0;
    }
    float _e144 = vPos.x;
    if ((_e144 > 1.0)) {
        vPos.x = -1.0;
    }
    float _e150 = vPos.y;
    if ((_e150 < -1.0)) {
        vPos.y = 1.0;
    }
    float _e156 = vPos.y;
    if ((_e156 > 1.0)) {
        vPos.y = -1.0;
    }
    vec2 _e165 = vPos;
    _group_0_binding_2_cs.particles[index].pos = _e165;
    vec2 _e170 = vVel;
    _group_0_binding_2_cs.particles[index].vel = _e170;
    return;
}

