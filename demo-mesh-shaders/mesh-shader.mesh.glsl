// Compile with: glslangValidator --target-env vulkan1.3 -S mesh mesh-shader.mesh.glsl
#version 460
#extension GL_EXT_mesh_shader : require

layout(local_size_x = 3) in;
layout(triangles, max_vertices = 3, max_primitives = 1) out;

struct TaskPayload {
    vec4 colorMask;
};
taskPayloadSharedEXT TaskPayload taskPayload;

layout(location = 0) out vec4 color[];
layout(location = 1) perprimitiveEXT out vec4 colorMask[];

void main() {
    uint i = gl_LocalInvocationIndex;
    SetMeshOutputsEXT(3, 1);
    if (i == 0) {
        gl_PrimitiveTriangleIndicesEXT[0] = uvec3(0, 1, 2);
        gl_MeshPrimitivesEXT[0].gl_CullPrimitiveEXT = false;
        colorMask[0] = vec4(1.0, 0.0, 1.0, 1.0);
    }
    gl_MeshVerticesEXT[i].gl_Position = vec4(float(i) - 1.0, float(i & 1u) * 2.0 - 1.0, 0.0, 1.0);
    color[i] = vec4(float(i == 0), float(i == 1), float(i == 2), 1.0);
}
