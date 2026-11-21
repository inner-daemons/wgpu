// language: metal1.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size7;
};

typedef float T;
struct type_2 {
    metal::float4 inner[3];
};
struct type_3 {
    type_2 inner[5];
};
struct type_5 {
    metal::atomic_uint inner[19];
};
struct type_8 {
    metal::float2x2 inner[3];
};
struct Inner {
    type_3 values;
    type_5 atoms;
    char _pad2[4];
    metal::float3x2 matrix_;
    type_8 matrices;
    char _pad4[8];
};
struct Tagged {
    metal::float2x2 matrix_;
};
struct type_9 {
    Inner inner[3];
};
struct type_10 {
    uint inner[37];
};
struct type_11 {
    T inner[37];
};
typedef uint type_14[1];
constant uint width = 8u;
constant uint height = 2u;
constant uint count = 37u;
uint naga_div(uint lhs, uint rhs) {
    return lhs / metal::select(rhs, 1u, rhs == 0u);
}

uint naga_mod(uint lhs, uint rhs) {
    return lhs % metal::select(rhs, 1u, rhs == 0u);
}


struct main_Input {
};
kernel void main_(
  uint zero_flat_index [[thread_index_in_threadgroup]]
, metal::uint3 group [[threadgroup_position_in_grid]]
, threadgroup type_9& data
, threadgroup type_10& dynamic
, threadgroup type_11& floats
, threadgroup Tagged& tagged
, threadgroup uint& scalar
, threadgroup metal::uint3& vector
, threadgroup metal::float2x3& matrix_
, device type_14& output [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
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
    uint2 loop_bound = uint2(4294967295u);
    while(true) {
        if (metal::all(loop_bound == uint2(0u))) { break; }
        loop_bound -= uint2(loop_bound.y == 0u, 1u);
        uint _e376 = zero_init_index;
        if (_e376 >= 45u) {
            break;
        }
        data.inner[naga_div(_e376, 15u)].values.inner[naga_mod(naga_div(_e376, 3u), 5u)].inner[naga_mod(_e376, 3u)] = metal::float4 {};
        zero_init_index = _e376 + 16u;
    }
    zero_init_index = zero_flat_index;
    uint2 loop_bound_1 = uint2(4294967295u);
    while(true) {
        if (metal::all(loop_bound_1 == uint2(0u))) { break; }
        loop_bound_1 -= uint2(loop_bound_1.y == 0u, 1u);
        uint _e396 = zero_init_index;
        if (_e396 >= 57u) {
            break;
        }
        metal::atomic_store_explicit(&data.inner[naga_div(_e396, 19u)].atoms.inner[naga_mod(_e396, 19u)], 0u, metal::memory_order_relaxed);
        zero_init_index = _e396 + 16u;
    }
    if (zero_flat_index < 3u) {
        data.inner[zero_flat_index].matrix_ = metal::float3x2 {};
    }
    if (zero_flat_index < 9u) {
        data.inner[naga_div(zero_flat_index, 3u)].matrices.inner[naga_mod(zero_flat_index, 3u)] = metal::float2x2 {};
    }
    zero_init_index = zero_flat_index;
    uint2 loop_bound_2 = uint2(4294967295u);
    while(true) {
        if (metal::all(loop_bound_2 == uint2(0u))) { break; }
        loop_bound_2 -= uint2(loop_bound_2.y == 0u, 1u);
        uint _e428 = zero_init_index;
        if (_e428 >= 37u) {
            break;
        }
        dynamic.inner[_e428] = 0u;
        zero_init_index = _e428 + 16u;
    }
    zero_init_index = zero_flat_index;
    uint2 loop_bound_3 = uint2(4294967295u);
    while(true) {
        if (metal::all(loop_bound_3 == uint2(0u))) { break; }
        loop_bound_3 -= uint2(loop_bound_3.y == 0u, 1u);
        uint _e437 = zero_init_index;
        if (_e437 >= 37u) {
            break;
        }
        floats.inner[_e437] = 0.0;
        zero_init_index = _e437 + 16u;
    }
    if (zero_flat_index == 0u) {
        tagged = Tagged();
        scalar = 0u;
        vector = metal::uint3 {};
        matrix_ = metal::float2x3 {};
    }
    metal::threadgroup_barrier(metal::mem_flags::mem_threadgroup);
    uint _e3 = scalar;
    if (_e3 == 0u) {
        metal::uint3 _e9 = vector;
        local = metal::all(_e9 == metal::uint3(0u));
    } else {
        local = false;
    }
    bool _e15 = local;
    ok = _e15;
    bool _e17 = ok;
    metal::float3 _e20 = matrix_[0];
    if (metal::all(_e20 == metal::float3(0.0))) {
        metal::float3 _e29 = matrix_[1];
        local_1 = metal::all(_e29 == metal::float3(0.0));
    } else {
        local_1 = false;
    }
    bool _e35 = local_1;
    ok = _e17 & _e35;
    bool _e37 = ok;
    metal::float2 _e41 = tagged.matrix_[0];
    if (metal::all(_e41 == metal::float2(0.0))) {
        metal::float2 _e51 = tagged.matrix_[1];
        local_2 = metal::all(_e51 == metal::float2(0.0));
    } else {
        local_2 = false;
    }
    bool _e57 = local_2;
    ok = _e37 & _e57;
    uint2 loop_bound_4 = uint2(4294967295u);
    bool loop_init = true;
    while(true) {
        if (metal::all(loop_bound_4 == uint2(0u))) { break; }
        loop_bound_4 -= uint2(loop_bound_4.y == 0u, 1u);
        if (!loop_init) {
            uint _e164 = i;
            i = _e164 + 1u;
        }
        loop_init = false;
        uint _e61 = i;
        if (_e61 < 3u) {
        } else {
            break;
        }
        {
            j = 0u;
            uint2 loop_bound_5 = uint2(4294967295u);
            bool loop_init_1 = true;
            while(true) {
                if (metal::all(loop_bound_5 == uint2(0u))) { break; }
                loop_bound_5 -= uint2(loop_bound_5.y == 0u, 1u);
                if (!loop_init_1) {
                    uint _e93 = j;
                    j = _e93 + 1u;
                }
                loop_init_1 = false;
                uint _e66 = j;
                if (_e66 < 5u) {
                } else {
                    break;
                }
                {
                    k = 0u;
                    uint2 loop_bound_6 = uint2(4294967295u);
                    bool loop_init_2 = true;
                    while(true) {
                        if (metal::all(loop_bound_6 == uint2(0u))) { break; }
                        loop_bound_6 -= uint2(loop_bound_6.y == 0u, 1u);
                        if (!loop_init_2) {
                            uint _e90 = k;
                            k = _e90 + 1u;
                        }
                        loop_init_2 = false;
                        uint _e71 = k;
                        if (_e71 < 3u) {
                        } else {
                            break;
                        }
                        {
                            bool _e74 = ok;
                            uint _e76 = i;
                            uint _e79 = j;
                            uint _e81 = k;
                            metal::float4 _e83 = data.inner[_e76].values.inner[_e79].inner[_e81];
                            ok = _e74 & metal::all(_e83 == metal::float4(0.0));
                        }
                    }
                }
            }
            j_1 = 0u;
            uint2 loop_bound_7 = uint2(4294967295u);
            bool loop_init_3 = true;
            while(true) {
                if (metal::all(loop_bound_7 == uint2(0u))) { break; }
                loop_bound_7 -= uint2(loop_bound_7.y == 0u, 1u);
                if (!loop_init_3) {
                    uint _e112 = j_1;
                    j_1 = _e112 + 1u;
                }
                loop_init_3 = false;
                uint _e97 = j_1;
                if (_e97 < 19u) {
                } else {
                    break;
                }
                {
                    bool _e100 = ok;
                    uint _e102 = i;
                    uint _e105 = j_1;
                    uint _e107 = metal::atomic_load_explicit(&data.inner[_e102].atoms.inner[_e105], metal::memory_order_relaxed);
                    ok = _e100 & (_e107 == 0u);
                }
            }
            j_2 = 0u;
            uint2 loop_bound_8 = uint2(4294967295u);
            bool loop_init_4 = true;
            while(true) {
                if (metal::all(loop_bound_8 == uint2(0u))) { break; }
                loop_bound_8 -= uint2(loop_bound_8.y == 0u, 1u);
                if (!loop_init_4) {
                    uint _e161 = j_2;
                    j_2 = _e161 + 1u;
                }
                loop_init_4 = false;
                uint _e116 = j_2;
                if (_e116 < 3u) {
                } else {
                    break;
                }
                {
                    bool _e119 = ok;
                    uint _e121 = i;
                    uint _e124 = j_2;
                    metal::float2 _e126 = data.inner[_e121].matrix_[_e124];
                    ok = _e119 & metal::all(_e126 == metal::float2(0.0));
                    bool _e132 = ok;
                    uint _e134 = i;
                    uint _e137 = j_2;
                    metal::float2 _e140 = data.inner[_e134].matrices.inner[_e137][0];
                    ok = _e132 & metal::all(_e140 == metal::float2(0.0));
                    bool _e146 = ok;
                    uint _e148 = i;
                    uint _e151 = j_2;
                    metal::float2 _e154 = data.inner[_e148].matrices.inner[_e151][1];
                    ok = _e146 & metal::all(_e154 == metal::float2(0.0));
                }
            }
        }
    }
    uint2 loop_bound_9 = uint2(4294967295u);
    bool loop_init_5 = true;
    while(true) {
        if (metal::all(loop_bound_9 == uint2(0u))) { break; }
        loop_bound_9 -= uint2(loop_bound_9.y == 0u, 1u);
        if (!loop_init_5) {
            uint _e188 = i_1;
            i_1 = _e188 + 1u;
        }
        loop_init_5 = false;
        uint _e168 = i_1;
        if (_e168 < count) {
        } else {
            break;
        }
        {
            bool _e171 = ok;
            uint _e173 = i_1;
            uint _e175 = dynamic.inner[_e173];
            ok = _e171 & (_e175 == 0u);
            bool _e179 = ok;
            uint _e181 = i_1;
            T _e183 = floats.inner[_e181];
            ok = _e179 & (_e183 == 0.0);
        }
    }
    metal::threadgroup_barrier(metal::mem_flags::mem_threadgroup);
    if (zero_flat_index == 0u) {
        scalar = 42u;
        vector = metal::uint3(42u);
        matrix_ = metal::float2x3(metal::float3(42.0), metal::float3(42.0));
        uint2 loop_bound_10 = uint2(4294967295u);
        bool loop_init_6 = true;
        while(true) {
            if (metal::all(loop_bound_10 == uint2(0u))) { break; }
            loop_bound_10 -= uint2(loop_bound_10.y == 0u, 1u);
            if (!loop_init_6) {
                uint _e250 = i_2;
                i_2 = _e250 + 1u;
            }
            loop_init_6 = false;
            uint _e205 = i_2;
            if (_e205 < 3u) {
            } else {
                break;
            }
            {
                j_3 = 0u;
                uint2 loop_bound_11 = uint2(4294967295u);
                bool loop_init_7 = true;
                while(true) {
                    if (metal::all(loop_bound_11 == uint2(0u))) { break; }
                    loop_bound_11 -= uint2(loop_bound_11.y == 0u, 1u);
                    if (!loop_init_7) {
                        uint _e232 = j_3;
                        j_3 = _e232 + 1u;
                    }
                    loop_init_7 = false;
                    uint _e210 = j_3;
                    if (_e210 < 5u) {
                    } else {
                        break;
                    }
                    {
                        k_1 = 0u;
                        uint2 loop_bound_12 = uint2(4294967295u);
                        bool loop_init_8 = true;
                        while(true) {
                            if (metal::all(loop_bound_12 == uint2(0u))) { break; }
                            loop_bound_12 -= uint2(loop_bound_12.y == 0u, 1u);
                            if (!loop_init_8) {
                                uint _e229 = k_1;
                                k_1 = _e229 + 1u;
                            }
                            loop_init_8 = false;
                            uint _e215 = k_1;
                            if (_e215 < 3u) {
                            } else {
                                break;
                            }
                            {
                                uint _e219 = i_2;
                                uint _e222 = j_3;
                                uint _e224 = k_1;
                                data.inner[_e219].values.inner[_e222].inner[_e224] = metal::float4(42.0);
                            }
                        }
                    }
                }
                j_4 = 0u;
                uint2 loop_bound_13 = uint2(4294967295u);
                bool loop_init_9 = true;
                while(true) {
                    if (metal::all(loop_bound_13 == uint2(0u))) { break; }
                    loop_bound_13 -= uint2(loop_bound_13.y == 0u, 1u);
                    if (!loop_init_9) {
                        uint _e247 = j_4;
                        j_4 = _e247 + 1u;
                    }
                    loop_init_9 = false;
                    uint _e236 = j_4;
                    if (_e236 < 19u) {
                    } else {
                        break;
                    }
                    {
                        uint _e240 = i_2;
                        uint _e243 = j_4;
                        metal::atomic_store_explicit(&data.inner[_e240].atoms.inner[_e243], 42u, metal::memory_order_relaxed);
                    }
                }
            }
        }
        uint2 loop_bound_14 = uint2(4294967295u);
        bool loop_init_10 = true;
        while(true) {
            if (metal::all(loop_bound_14 == uint2(0u))) { break; }
            loop_bound_14 -= uint2(loop_bound_14.y == 0u, 1u);
            if (!loop_init_10) {
                uint _e266 = i_3;
                i_3 = _e266 + 1u;
            }
            loop_init_10 = false;
            uint _e254 = i_3;
            if (_e254 < count) {
            } else {
                break;
            }
            {
                uint _e258 = i_3;
                dynamic.inner[_e258] = 42u;
                uint _e262 = i_3;
                floats.inner[_e262] = 42.0;
            }
        }
    }
    metal::threadgroup_barrier(metal::mem_flags::mem_threadgroup);
    bool _e268 = ok;
    uint _e270 = scalar;
    if (_e270 == 42u) {
        metal::uint3 _e276 = vector;
        local_3 = metal::all(_e276 == metal::uint3(42u));
    } else {
        local_3 = false;
    }
    bool _e282 = local_3;
    ok = _e268 & _e282;
    bool _e284 = ok;
    metal::float3 _e287 = matrix_[0];
    ok = _e284 & metal::all(_e287 == metal::float3(42.0));
    bool _e293 = ok;
    metal::float4 _e299 = data.inner[2].values.inner[4].inner[2];
    ok = _e293 & metal::all(_e299 == metal::float4(42.0));
    bool _e305 = ok;
    uint _e310 = metal::atomic_load_explicit(&data.inner[2].atoms.inner[18], metal::memory_order_relaxed);
    ok = _e305 & (_e310 == 42u);
    bool _e314 = ok;
    uint _e318 = dynamic.inner[36u];
    ok = _e314 & (_e318 == 42u);
    bool _e322 = ok;
    T _e326 = floats.inner[36u];
    ok = _e322 & (_e326 == 42.0);
    uint index = ((group.x * width) * height) + zero_flat_index;
    uint _e338 = output[index];
    bool _e341 = ok;
    output[index] = _e338 | (_e341 ? 2u : 1u);
    return;
}


struct singleInput {
};
kernel void single(
  uint local_invocation_index [[thread_index_in_threadgroup]]
, threadgroup uint& scalar
, device type_14& output [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    if (local_invocation_index == 0u) {
        scalar = 0u;
    }
    metal::threadgroup_barrier(metal::mem_flags::mem_threadgroup);
    uint _e3 = scalar;
    output[0] = _e3;
    return;
}
