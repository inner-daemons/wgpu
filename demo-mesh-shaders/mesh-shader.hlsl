// Compile with: dxc -T as_6_5 -E ts_main, dxc -T ms_6_5 -E ms_main, dxc -T ps_6_5 -E fs_main

struct TaskPayload {
    float4 colorMask;
};
struct VertexOutput {
    float4 position : SV_Position;
    float4 color : LOC0;
};
struct PrimitiveOutput {
    bool cull : SV_CullPrimitive;
    float4 colorMask : LOC1;
};
struct PrimitiveInput {
    nointerpolation float4 colorMask : LOC1;
};

groupshared TaskPayload taskPayload;

[numthreads(3, 1, 1)]
void ts_main(uint i : SV_GroupIndex) {
    if (i == 0) {
        taskPayload.colorMask = float4(1.0, 1.0, 0.0, 1.0);
    }
    DispatchMesh(1, 1, 1, taskPayload);
}

[numthreads(3, 1, 1)]
[outputtopology("triangle")]
void ms_main(
    uint i : SV_GroupIndex,
    in payload TaskPayload payload,
    out vertices VertexOutput vertices[3],
    out indices uint3 indices[1],
    out primitives PrimitiveOutput primitives[1]
) {
    SetMeshOutputCounts(3, 1);
    if (i == 0) {
        indices[0] = uint3(0, 1, 2);
        primitives[0].cull = false;
        primitives[0].colorMask = float4(1.0, 0.0, 1.0, 1.0);
    }
    vertices[i].position = float4(float(i) - 1.0, float(i & 1) * 2.0 - 1.0, 0.0, 1.0);
    vertices[i].color = float4(float(i == 0), float(i == 1), float(i == 2), 1.0);
}

float4 fs_main(VertexOutput vertex, PrimitiveInput primitive) : SV_Target0 {
    return vertex.color * primitive.colorMask;
}
