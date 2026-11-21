#version 310 es

precision highp float;
precision highp int;

layout(local_size_x = 8, local_size_y = 2, local_size_z = 1) in;

struct Inner {
    vec4 values[5][3];
    uint atoms[19];
    mat3x2 matrix;
    mat2x2 matrices[3];
};
struct Tagged {
    mat2x2 matrix;
};
const uint width = 8u;
const uint height = 2u;
const uint count = 37u;

shared Inner data[3];

shared uint dynamic[37];

shared float floats[37];

shared Tagged tagged;

shared uint scalar;

shared uvec3 vector;

shared mat2x3 matrix;

layout(std430) buffer type_14_block_0Compute { uint _group_0_binding_0_cs[]; };


void main() {
    uint zero_flat_index = gl_LocalInvocationIndex;
    uvec3 group = gl_WorkGroupID;
    bool local;
    bool ok;
    bool local_1;
    bool local_2;
    uint i;
    uint j;
    uint k;
    uint j_1;
    uint j_2;
    uint i_1;
    uint i_2;
    uint j_3;
    uint k_1;
    uint j_4;
    uint i_3;
    bool local_3;
    uint zero_init_index;
    local = false;
    ok = false;
    local_1 = false;
    local_2 = false;
    i = 0u;
    j = 0u;
    k = 0u;
    j_1 = 0u;
    j_2 = 0u;
    i_1 = 0u;
    i_2 = 0u;
    j_3 = 0u;
    k_1 = 0u;
    j_4 = 0u;
    i_3 = 0u;
    local_3 = false;
    zero_init_index = zero_flat_index;
    while(true) {
        uint _e376 = zero_init_index;
        if ((_e376 >= 45u)) {
            break;
        }
        uint _e383 = (_e376 / 3u);
        data[(_e376 / 15u)].values[(_e383 - 5u * (_e383 / 5u))][(_e376 - 3u * (_e376 / 3u))] = vec4(0.0);
        zero_init_index = (_e376 + 16u);
    }
    zero_init_index = zero_flat_index;
    while(true) {
        uint _e396 = zero_init_index;
        if ((_e396 >= 57u)) {
            break;
        }
        atomicExchange(data[(_e396 / 19u)].atoms[(_e396 - 19u * (_e396 / 19u))], 0u);
        zero_init_index = (_e396 + 16u);
    }
    if ((zero_flat_index < 3u)) {
        data[zero_flat_index].matrix = mat3x2(0.0);
    }
    if ((zero_flat_index < 9u)) {
        data[(zero_flat_index / 3u)].matrices[(zero_flat_index - 3u * (zero_flat_index / 3u))] = mat2x2(0.0);
    }
    zero_init_index = zero_flat_index;
    while(true) {
        uint _e428 = zero_init_index;
        if ((_e428 >= 37u)) {
            break;
        }
        dynamic[_e428] = 0u;
        zero_init_index = (_e428 + 16u);
    }
    zero_init_index = zero_flat_index;
    while(true) {
        uint _e437 = zero_init_index;
        if ((_e437 >= 37u)) {
            break;
        }
        floats[_e437] = 0.0;
        zero_init_index = (_e437 + 16u);
    }
    if ((zero_flat_index == 0u)) {
        tagged = Tagged(mat2x2(0.0));
        scalar = 0u;
        vector = uvec3(0u);
        matrix = mat2x3(0.0);
    }
    memoryBarrierShared();
    barrier();
    uint _e3 = scalar;
    if ((_e3 == 0u)) {
        uvec3 _e9 = vector;
        local = all(equal(_e9, uvec3(0u)));
    } else {
        local = false;
    }
    bool _e15 = local;
    ok = _e15;
    bool _e17 = ok;
    vec3 _e20 = matrix[0];
    if (all(equal(_e20, vec3(0.0)))) {
        vec3 _e29 = matrix[1];
        local_1 = all(equal(_e29, vec3(0.0)));
    } else {
        local_1 = false;
    }
    bool _e35 = local_1;
    ok = (_e17 && _e35);
    bool _e37 = ok;
    vec2 _e41 = tagged.matrix[0];
    if (all(equal(_e41, vec2(0.0)))) {
        vec2 _e51 = tagged.matrix[1];
        local_2 = all(equal(_e51, vec2(0.0)));
    } else {
        local_2 = false;
    }
    bool _e57 = local_2;
    ok = (_e37 && _e57);
    bool loop_init = true;
    while(true) {
        if (!loop_init) {
            uint _e164 = i;
            i = (_e164 + 1u);
        }
        loop_init = false;
        uint _e61 = i;
        if ((_e61 < 3u)) {
        } else {
            break;
        }
        {
            j = 0u;
            bool loop_init_1 = true;
            while(true) {
                if (!loop_init_1) {
                    uint _e93 = j;
                    j = (_e93 + 1u);
                }
                loop_init_1 = false;
                uint _e66 = j;
                if ((_e66 < 5u)) {
                } else {
                    break;
                }
                {
                    k = 0u;
                    bool loop_init_2 = true;
                    while(true) {
                        if (!loop_init_2) {
                            uint _e90 = k;
                            k = (_e90 + 1u);
                        }
                        loop_init_2 = false;
                        uint _e71 = k;
                        if ((_e71 < 3u)) {
                        } else {
                            break;
                        }
                        {
                            bool _e74 = ok;
                            uint _e76 = i;
                            uint _e79 = j;
                            uint _e81 = k;
                            vec4 _e83 = data[_e76].values[_e79][_e81];
                            ok = (_e74 && all(equal(_e83, vec4(0.0))));
                        }
                    }
                }
            }
            j_1 = 0u;
            bool loop_init_3 = true;
            while(true) {
                if (!loop_init_3) {
                    uint _e112 = j_1;
                    j_1 = (_e112 + 1u);
                }
                loop_init_3 = false;
                uint _e97 = j_1;
                if ((_e97 < 19u)) {
                } else {
                    break;
                }
                {
                    bool _e100 = ok;
                    uint _e102 = i;
                    uint _e105 = j_1;
                    uint _e107 = atomicOr(data[_e102].atoms[_e105], 0u);
                    ok = (_e100 && (_e107 == 0u));
                }
            }
            j_2 = 0u;
            bool loop_init_4 = true;
            while(true) {
                if (!loop_init_4) {
                    uint _e161 = j_2;
                    j_2 = (_e161 + 1u);
                }
                loop_init_4 = false;
                uint _e116 = j_2;
                if ((_e116 < 3u)) {
                } else {
                    break;
                }
                {
                    bool _e119 = ok;
                    uint _e121 = i;
                    uint _e124 = j_2;
                    vec2 _e126 = data[_e121].matrix[_e124];
                    ok = (_e119 && all(equal(_e126, vec2(0.0))));
                    bool _e132 = ok;
                    uint _e134 = i;
                    uint _e137 = j_2;
                    vec2 _e140 = data[_e134].matrices[_e137][0];
                    ok = (_e132 && all(equal(_e140, vec2(0.0))));
                    bool _e146 = ok;
                    uint _e148 = i;
                    uint _e151 = j_2;
                    vec2 _e154 = data[_e148].matrices[_e151][1];
                    ok = (_e146 && all(equal(_e154, vec2(0.0))));
                }
            }
        }
    }
    bool loop_init_5 = true;
    while(true) {
        if (!loop_init_5) {
            uint _e188 = i_1;
            i_1 = (_e188 + 1u);
        }
        loop_init_5 = false;
        uint _e168 = i_1;
        if ((_e168 < count)) {
        } else {
            break;
        }
        {
            bool _e171 = ok;
            uint _e173 = i_1;
            uint _e175 = dynamic[_e173];
            ok = (_e171 && (_e175 == 0u));
            bool _e179 = ok;
            uint _e181 = i_1;
            float _e183 = floats[_e181];
            ok = (_e179 && (_e183 == 0.0));
        }
    }
    memoryBarrierShared();
    barrier();
    if ((zero_flat_index == 0u)) {
        scalar = 42u;
        vector = uvec3(42u);
        matrix = mat2x3(vec3(42.0), vec3(42.0));
        bool loop_init_6 = true;
        while(true) {
            if (!loop_init_6) {
                uint _e250 = i_2;
                i_2 = (_e250 + 1u);
            }
            loop_init_6 = false;
            uint _e205 = i_2;
            if ((_e205 < 3u)) {
            } else {
                break;
            }
            {
                j_3 = 0u;
                bool loop_init_7 = true;
                while(true) {
                    if (!loop_init_7) {
                        uint _e232 = j_3;
                        j_3 = (_e232 + 1u);
                    }
                    loop_init_7 = false;
                    uint _e210 = j_3;
                    if ((_e210 < 5u)) {
                    } else {
                        break;
                    }
                    {
                        k_1 = 0u;
                        bool loop_init_8 = true;
                        while(true) {
                            if (!loop_init_8) {
                                uint _e229 = k_1;
                                k_1 = (_e229 + 1u);
                            }
                            loop_init_8 = false;
                            uint _e215 = k_1;
                            if ((_e215 < 3u)) {
                            } else {
                                break;
                            }
                            {
                                uint _e219 = i_2;
                                uint _e222 = j_3;
                                uint _e224 = k_1;
                                data[_e219].values[_e222][_e224] = vec4(42.0);
                            }
                        }
                    }
                }
                j_4 = 0u;
                bool loop_init_9 = true;
                while(true) {
                    if (!loop_init_9) {
                        uint _e247 = j_4;
                        j_4 = (_e247 + 1u);
                    }
                    loop_init_9 = false;
                    uint _e236 = j_4;
                    if ((_e236 < 19u)) {
                    } else {
                        break;
                    }
                    {
                        uint _e240 = i_2;
                        uint _e243 = j_4;
                        atomicExchange(data[_e240].atoms[_e243], 42u);
                    }
                }
            }
        }
        bool loop_init_10 = true;
        while(true) {
            if (!loop_init_10) {
                uint _e266 = i_3;
                i_3 = (_e266 + 1u);
            }
            loop_init_10 = false;
            uint _e254 = i_3;
            if ((_e254 < count)) {
            } else {
                break;
            }
            {
                uint _e258 = i_3;
                dynamic[_e258] = 42u;
                uint _e262 = i_3;
                floats[_e262] = 42.0;
            }
        }
    }
    memoryBarrierShared();
    barrier();
    bool _e268 = ok;
    uint _e270 = scalar;
    if ((_e270 == 42u)) {
        uvec3 _e276 = vector;
        local_3 = all(equal(_e276, uvec3(42u)));
    } else {
        local_3 = false;
    }
    bool _e282 = local_3;
    ok = (_e268 && _e282);
    bool _e284 = ok;
    vec3 _e287 = matrix[0];
    ok = (_e284 && all(equal(_e287, vec3(42.0))));
    bool _e293 = ok;
    vec4 _e299 = data[2].values[4][2];
    ok = (_e293 && all(equal(_e299, vec4(42.0))));
    bool _e305 = ok;
    uint _e310 = atomicOr(data[2].atoms[18], 0u);
    ok = (_e305 && (_e310 == 42u));
    bool _e314 = ok;
    uint _e318 = dynamic[36u];
    ok = (_e314 && (_e318 == 42u));
    bool _e322 = ok;
    float _e326 = floats[36u];
    ok = (_e322 && (_e326 == 42.0));
    uint index = (((group.x * width) * height) + zero_flat_index);
    uint _e338 = _group_0_binding_0_cs[index];
    bool _e341 = ok;
    _group_0_binding_0_cs[index] = (_e338 | (_e341 ? 2u : 1u));
    return;
}

