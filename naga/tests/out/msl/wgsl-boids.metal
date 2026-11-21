// language: metal1.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size1;
    uint size2;
};

struct Particle {
    metal::float2 pos;
    metal::float2 vel;
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
typedef Particle type_3[1];
struct Particles {
    type_3 particles;
};
constant uint NUM_PARTICLES = 1500u;

struct main_Input {
};
kernel void main_(
  metal::uint3 global_invocation_id [[thread_position_in_grid]]
, constant SimParams& params [[buffer(0)]]
, device Particles const& particlesSrc [[buffer(1)]]
, device Particles& particlesDst [[buffer(2)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(3)]]
) {
    metal::float2 vPos;
    metal::float2 vVel;
    metal::float2 cMass;
    metal::float2 cVel;
    metal::float2 colVel;
    int cMassCount;
    int cVelCount;
    metal::float2 pos;
    metal::float2 vel;
    uint i;
    vPos = metal::float2 {};
    vVel = metal::float2 {};
    cMass = metal::float2(0.0, 0.0);
    cVel = metal::float2(0.0, 0.0);
    colVel = metal::float2(0.0, 0.0);
    cMassCount = 0;
    cVelCount = 0;
    pos = metal::float2 {};
    vel = metal::float2 {};
    i = 0u;
    uint index = global_invocation_id.x;
    if (index >= NUM_PARTICLES) {
        return;
    }
    metal::float2 _e8 = particlesSrc.particles[index].pos;
    vPos = _e8;
    metal::float2 _e14 = particlesSrc.particles[index].vel;
    vVel = _e14;
    uint2 loop_bound = uint2(4294967295u);
    bool loop_init = true;
    while(true) {
        if (metal::all(loop_bound == uint2(0u))) { break; }
        loop_bound -= uint2(loop_bound.y == 0u, 1u);
        if (!loop_init) {
            uint _e82 = i;
            i = _e82 + 1u;
        }
        loop_init = false;
        uint _e27 = i;
        if (_e27 >= NUM_PARTICLES) {
            break;
        }
        uint _e30 = i;
        if (_e30 == index) {
            continue;
        }
        uint _e34 = i;
        metal::float2 _e37 = particlesSrc.particles[_e34].pos;
        pos = _e37;
        uint _e40 = i;
        metal::float2 _e43 = particlesSrc.particles[_e40].vel;
        vel = _e43;
        metal::float2 _e44 = pos;
        metal::float2 _e45 = vPos;
        float _e49 = params.rule1Distance;
        if (metal::distance(_e44, _e45) < _e49) {
            metal::float2 _e51 = cMass;
            metal::float2 _e52 = pos;
            cMass = _e51 + _e52;
            int _e54 = cMassCount;
            cMassCount = as_type<int>(as_type<uint>(_e54) + as_type<uint>(1));
        }
        metal::float2 _e57 = pos;
        metal::float2 _e58 = vPos;
        float _e62 = params.rule2Distance;
        if (metal::distance(_e57, _e58) < _e62) {
            metal::float2 _e64 = colVel;
            metal::float2 _e65 = pos;
            metal::float2 _e66 = vPos;
            colVel = _e64 - (_e65 - _e66);
        }
        metal::float2 _e69 = pos;
        metal::float2 _e70 = vPos;
        float _e74 = params.rule3Distance;
        if (metal::distance(_e69, _e70) < _e74) {
            metal::float2 _e76 = cVel;
            metal::float2 _e77 = vel;
            cVel = _e76 + _e77;
            int _e79 = cVelCount;
            cVelCount = as_type<int>(as_type<uint>(_e79) + as_type<uint>(1));
        }
    }
    int _e85 = cMassCount;
    if (_e85 > 0) {
        metal::float2 _e88 = cMass;
        int _e89 = cMassCount;
        metal::float2 _e93 = vPos;
        cMass = (_e88 / metal::float2(static_cast<float>(_e89))) - _e93;
    }
    int _e95 = cVelCount;
    if (_e95 > 0) {
        metal::float2 _e98 = cVel;
        int _e99 = cVelCount;
        cVel = _e98 / metal::float2(static_cast<float>(_e99));
    }
    metal::float2 _e103 = vVel;
    metal::float2 _e104 = cMass;
    float _e107 = params.rule1Scale;
    metal::float2 _e110 = colVel;
    float _e113 = params.rule2Scale;
    metal::float2 _e116 = cVel;
    float _e119 = params.rule3Scale;
    vVel = ((_e103 + (_e104 * _e107)) + (_e110 * _e113)) + (_e116 * _e119);
    metal::float2 _e122 = vVel;
    metal::float2 _e124 = vVel;
    vVel = metal::normalize(_e122) * metal::clamp(metal::length(_e124), 0.0, 0.1);
    metal::float2 _e130 = vPos;
    metal::float2 _e131 = vVel;
    float _e134 = params.deltaT;
    vPos = _e130 + (_e131 * _e134);
    float _e138 = vPos.x;
    if (_e138 < -1.0) {
        vPos.x = 1.0;
    }
    float _e144 = vPos.x;
    if (_e144 > 1.0) {
        vPos.x = -1.0;
    }
    float _e150 = vPos.y;
    if (_e150 < -1.0) {
        vPos.y = 1.0;
    }
    float _e156 = vPos.y;
    if (_e156 > 1.0) {
        vPos.y = -1.0;
    }
    metal::float2 _e165 = vPos;
    particlesDst.particles[index].pos = _e165;
    metal::float2 _e170 = vVel;
    particlesDst.particles[index].vel = _e170;
    return;
}
