// Compile each stage with one of:
//   glslangValidator --target-env vulkan1.3 -S task -DTASK mesh-shader.glsl
//   glslangValidator --target-env vulkan1.3 -S mesh -DMESH mesh-shader.glsl
//   glslangValidator --target-env vulkan1.3 -S frag -DFRAG mesh-shader.glsl
#version 460
#extension GL_EXT_mesh_shader : require

#if defined(TASK) || defined(MESH)
layout(local_size_x = 3) in;

struct TaskPayload {
    vec4 colorMask;
};
taskPayloadSharedEXT TaskPayload taskPayload;
#endif

#ifdef MESH
#define VARYING out
#define PER_OUTPUT []
#else
#define VARYING in
#define PER_OUTPUT
#endif

#if defined(MESH) || defined(FRAG)
layout(location = 0) VARYING vec4 color PER_OUTPUT;
layout(location = 1) perprimitiveEXT VARYING vec4 colorMask PER_OUTPUT;
#endif

#ifdef TASK
void main() {
    uint i = gl_LocalInvocationIndex;
    if (i == 0) {
        taskPayload.colorMask = vec4(1.0, 1.0, 0.0, 1.0);
    }
    EmitMeshTasksEXT(1, 1, 1);
}
#endif

#ifdef MESH
layout(triangles, max_vertices = 3, max_primitives = 1) out;

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
#endif

#ifdef FRAG
layout(location = 0) out vec4 outColor;

void main() {
    outColor = color * colorMask;
}
#endif
