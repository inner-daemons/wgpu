struct Particle {
    float2 pos;
    float2 vel;
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

static const uint NUM_PARTICLES = 1500u;

cbuffer params : register(b0) { SimParams params; }
ByteAddressBuffer particlesSrc : register(t1);
RWByteAddressBuffer particlesDst : register(u2);

float2 ZeroValuefloat2() {
    return (float2)0;
}

[numthreads(64, 1, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID)
{
    float2 vPos;
    float2 vVel;
    float2 cMass;
    float2 cVel;
    float2 colVel;
    int cMassCount;
    int cVelCount;
    float2 pos;
    float2 vel;
    uint i;

    vPos = ZeroValuefloat2();
    vVel = ZeroValuefloat2();
    cMass = float2(0.0, 0.0);
    cVel = float2(0.0, 0.0);
    colVel = float2(0.0, 0.0);
    cMassCount = int(0);
    cVelCount = int(0);
    pos = ZeroValuefloat2();
    vel = ZeroValuefloat2();
    i = 0u;
    uint index = global_invocation_id.x;
    if ((index >= NUM_PARTICLES)) {
        return;
    }
    float2 _e8 = asfloat(particlesSrc.Load2(0+index*16+0));
    vPos = _e8;
    float2 _e14 = asfloat(particlesSrc.Load2(8+index*16+0));
    vVel = _e14;
    uint2 loop_bound = uint2(4294967295u, 4294967295u);
    bool loop_init = true;
    while(true) {
        if (all(loop_bound == uint2(0u, 0u))) { break; }
        loop_bound -= uint2(loop_bound.y == 0u, 1u);
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
        float2 _e37 = asfloat(particlesSrc.Load2(0+_e34*16+0));
        pos = _e37;
        uint _e40 = i;
        float2 _e43 = asfloat(particlesSrc.Load2(8+_e40*16+0));
        vel = _e43;
        float2 _e44 = pos;
        float2 _e45 = vPos;
        float _e49 = params.rule1Distance;
        if ((distance(_e44, _e45) < _e49)) {
            float2 _e51 = cMass;
            float2 _e52 = pos;
            cMass = (_e51 + _e52);
            int _e54 = cMassCount;
            cMassCount = asint(asuint(_e54) + asuint(int(1)));
        }
        float2 _e57 = pos;
        float2 _e58 = vPos;
        float _e62 = params.rule2Distance;
        if ((distance(_e57, _e58) < _e62)) {
            float2 _e64 = colVel;
            float2 _e65 = pos;
            float2 _e66 = vPos;
            colVel = (_e64 - (_e65 - _e66));
        }
        float2 _e69 = pos;
        float2 _e70 = vPos;
        float _e74 = params.rule3Distance;
        if ((distance(_e69, _e70) < _e74)) {
            float2 _e76 = cVel;
            float2 _e77 = vel;
            cVel = (_e76 + _e77);
            int _e79 = cVelCount;
            cVelCount = asint(asuint(_e79) + asuint(int(1)));
        }
    }
    int _e85 = cMassCount;
    if ((_e85 > int(0))) {
        float2 _e88 = cMass;
        int _e89 = cMassCount;
        float2 _e93 = vPos;
        cMass = ((_e88 / (float(_e89)).xx) - _e93);
    }
    int _e95 = cVelCount;
    if ((_e95 > int(0))) {
        float2 _e98 = cVel;
        int _e99 = cVelCount;
        cVel = (_e98 / (float(_e99)).xx);
    }
    float2 _e103 = vVel;
    float2 _e104 = cMass;
    float _e107 = params.rule1Scale;
    float2 _e110 = colVel;
    float _e113 = params.rule2Scale;
    float2 _e116 = cVel;
    float _e119 = params.rule3Scale;
    vVel = (((_e103 + (_e104 * _e107)) + (_e110 * _e113)) + (_e116 * _e119));
    float2 _e122 = vVel;
    float2 _e124 = vVel;
    vVel = (normalize(_e122) * clamp(length(_e124), 0.0, 0.1));
    float2 _e130 = vPos;
    float2 _e131 = vVel;
    float _e134 = params.deltaT;
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
    float2 _e165 = vPos;
    particlesDst.Store2(0+index*16+0, asuint(_e165));
    float2 _e170 = vVel;
    particlesDst.Store2(8+index*16+0, asuint(_e170));
    return;
}
