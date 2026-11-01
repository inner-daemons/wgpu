enable wgpu_mesh_shader;

struct TaskPayload {
    colorMask: vec4<f32>,
}
var<task_payload> taskPayload: TaskPayload;

@task
@payload(taskPayload)
@workgroup_size(3)
fn ts_main(@builtin(local_invocation_index) i: u32) -> @builtin(mesh_task_size) vec3<u32> {
    if i == 0 {
        taskPayload.colorMask = vec4(1.0, 1.0, 0.0, 1.0);
    }
    // Return directly rather than using a special function
    return vec3(1, 1, 1);
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}
struct PrimitiveOutput {
    @builtin(triangle_indices) indices: vec3<u32>,
    @builtin(cull_primitive) cull: bool,
    @per_primitive @location(1) colorMask: vec4<f32>,
}

// Write outputs to a workgroup variable, more flexible
struct MeshOutput {
    @builtin(vertices) vertices: array<VertexOutput, 3>,
    @builtin(primitives) primitives: array<PrimitiveOutput, 1>,
    @builtin(vertex_count) vertex_count: u32,
    @builtin(primitive_count) primitive_count: u32,
}
var<workgroup> mesh_output: MeshOutput;

@mesh(mesh_output)
@payload(taskPayload)
@workgroup_size(3)
fn ms_main(@builtin(local_invocation_index) i: u32) {
    if i == 0 {
        // These can be set at any point, no ordering rules
        mesh_output.vertex_count = 3;
        mesh_output.primitive_count = 1;

        mesh_output.primitives[0].indices = vec3<u32>(0, 1, 2);
        mesh_output.primitives[0].cull = false;
        mesh_output.primitives[0].colorMask = vec4<f32>(1.0, 0.0, 1.0, 1.0);
    }
    mesh_output.vertices[i].position = vec4(f32(i) - 1.0, f32(i & 1u) * 2.0 - 1.0, 0.0, 1.0);
    mesh_output.vertices[i].color = vec4(f32(i == 0u), f32(i == 1u), f32(i == 2u), 1.0);
}

// Fragment shader stuff is the same
struct VertexInput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}
struct PrimitiveInput {
    @per_primitive @location(1) colorMask: vec4<f32>,
}

@fragment
fn fs_main(vertex: VertexInput, primitive: PrimitiveInput) -> @location(0) vec4<f32> {
    return vertex.color * primitive.colorMask;
}
