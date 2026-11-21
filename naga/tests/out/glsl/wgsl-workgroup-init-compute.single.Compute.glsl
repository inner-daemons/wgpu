#version 310 es

precision highp float;
precision highp int;

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

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

shared uint scalar;

layout(std430) buffer type_14_block_0Compute { uint _group_0_binding_0_cs[]; };


void main() {
    uint local_invocation_index = gl_LocalInvocationIndex;
    if ((local_invocation_index == 0u)) {
        scalar = 0u;
    }
    memoryBarrierShared();
    barrier();
    uint _e3 = scalar;
    _group_0_binding_0_cs[0] = _e3;
    return;
}

