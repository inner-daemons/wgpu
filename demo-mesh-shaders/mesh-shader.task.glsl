// Compile with: glslangValidator --target-env vulkan1.3 -S task mesh-shader.task.glsl
#version 460
#extension GL_EXT_mesh_shader : require

layout(local_size_x = 3) in;

struct TaskPayload {
    vec4 colorMask;
};
taskPayloadSharedEXT TaskPayload taskPayload;

void main() {
    uint i = gl_LocalInvocationIndex;
    if (i == 0) {
        taskPayload.colorMask = vec4(1.0, 1.0, 0.0, 1.0);
    }
    EmitMeshTasksEXT(1, 1, 1);
}
