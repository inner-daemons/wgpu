// Compile with: xcrun -sdk macosx metal -std=metal3.0 -c mesh-shader.metal
// Threads per object and mesh threadgroup (3) are set on the draw call.

#include <metal_stdlib>
using namespace metal;

struct TaskPayload {
    float4 colorMask;
};
struct VertexOutput {
    float4 position [[position]];
    float4 color [[user(loc0)]];
};
struct PrimitiveOutput {
    bool cull [[primitive_culled]];
    float4 colorMask [[user(loc1), flat]];
};
struct FragmentInput {
    float4 color [[user(loc0)]];
    float4 colorMask [[user(loc1), flat]];
};

using MeshOutput = metal::mesh<VertexOutput, PrimitiveOutput, 3, 1, metal::topology::triangle>;

[[object]] void ts_main(
    object_data TaskPayload& taskPayload [[payload]],
    mesh_grid_properties grid,
    uint i [[thread_index_in_threadgroup]]
) {
    if (i == 0) {
        taskPayload.colorMask = float4(1.0, 1.0, 0.0, 1.0);
        grid.set_threadgroups_per_grid(uint3(1, 1, 1));
    }
}

[[mesh]] void ms_main(
    MeshOutput output,
    const object_data TaskPayload& taskPayload [[payload]],
    uint i [[thread_index_in_threadgroup]]
) {
    if (i == 0) {
        output.set_primitive_count(1);

        PrimitiveOutput p;
        p.cull = false;
        p.colorMask = float4(1.0, 0.0, 1.0, 1.0);
        output.set_primitive(0, p);

        output.set_index(0, 0);
        output.set_index(1, 1);
        output.set_index(2, 2);
    }
    VertexOutput v;
    v.position = float4(float(i) - 1.0, float(i & 1) * 2.0 - 1.0, 0.0, 1.0);
    v.color = float4(float(i == 0), float(i == 1), float(i == 2), 1.0);
    output.set_vertex(i, v);
}

fragment float4 fs_main(FragmentInput in [[stage_in]]) {
    return in.color * in.colorMask;
}
