#version 460
#extension GL_EXT_samplerless_texture_functions : require
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

struct InStorage
{
    vec4 a[10];
};

struct InUniform
{
    vec4 a[20];
};

const vec4 _51[2] = vec4[](vec4(0.7070000171661376953125, 0.0, 0.0, 1.0), vec4(0.0, 0.7070000171661376953125, 0.0, 1.0));

layout(set = 0, binding = 0, std430) readonly buffer in_storage
{
    InStorage _m0;
} in_storage_1;

layout(set = 0, binding = 1, std140) uniform in_uniform
{
    InUniform _m0;
} in_uniform_1;

layout(set = 0, binding = 2) uniform texture2DArray image_2d_array;

shared float in_workgroup[30];
float in_private[40];

vec4 mock_function(ivec2 c, int i, int l)
{
    ivec3 _65 = ivec3(c, i);
    vec4 _78;
    if (uint(l) < uint(textureQueryLevels(image_2d_array)))
    {
        if (all(lessThan(uvec3(_65), uvec3(textureSize(image_2d_array, l)))))
        {
            _78 = texelFetch(image_2d_array, _65, l);
        }
        else
        {
            _78 = vec4(0.0);
        }
    }
    else
    {
        _78 = vec4(0.0);
    }
    return ((((in_storage_1._m0.a[i] + in_uniform_1._m0.a[i]) + _78) + vec4(in_workgroup[min(uint(i), 29u)])) + vec4(in_private[min(uint(i), 39u)])) + _51[min(uint(i), 1u)];
}

void main()
{
    uvec2 loop_bound = uvec2(4294967295u);
    in_private = float[](0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    uint zero_init_index = gl_LocalInvocationIndex;
    for (;;)
    {
        if (all(equal(uvec2(0u), loop_bound)))
        {
            break;
        }
        loop_bound -= uvec2(uint(loop_bound.y == 0u), 1u);
        if (zero_init_index >= 30u)
        {
            break;
        }
        in_workgroup[min(zero_init_index, 29u)] = 0.0;
        zero_init_index++;
        continue;
    }
    barrier();
}

