// Compile with: glslangValidator --target-env vulkan1.3 -S frag mesh-shader.frag.glsl
#version 460
#extension GL_EXT_mesh_shader : require

layout(location = 0) in vec4 color;
layout(location = 1) perprimitiveEXT in vec4 colorMask;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = color * colorMask;
}
