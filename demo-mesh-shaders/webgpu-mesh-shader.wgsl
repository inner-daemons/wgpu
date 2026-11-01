enable mesh_shader;

// Unused, left for demonstration
struct TaskPayload {
    colorMask: vec4<f32>,
}
// New address space, similar to workgroup storage but readonly
// on mesh shaders.
var<task_payload> taskPayload: TaskPayload;

@task                 // Task shader stage
@payload(taskPayload) // The only task_payload it's allowed to use
@workgroup_size(3)
fn ts_main(@builtin(local_invocation_index) i: u32) {
    if i == 0 {
        // Visible to all dispatched mesh shader workgroups
        taskPayload.colorMask = vec4(1.0, 1.0, 0.0, 1.0);
    }
    // Set the number of mesh shader workgroups to dispatch
    // More on the semantics soon
    emitMeshTasks(1, 1, 1);
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}
struct PrimitiveOutput {
    // If true the primitive skips rasterization
    @builtin(cull_primitive) cull: bool,
    // Not tied to any specific vertex, not interpolated.
    //
    // Shares a binding space with vertex attributes,
    // so this couldn't use @location(0)
    @per_primitive @location(1) colorMask: vec4<f32>,
}

@mesh
@workgroup_size(3)
@payload(taskPayload)
@topology(triangles) // The mesh shader will produce a triangle meshlet
@vertices(VertexOutput, 3) // Set the vertex/primitive types and max outputs
@primitives(PrimitiveOutput, 1)
fn ms_main(@builtin(local_invocation_index) i: u32) {
    // Set the number of outputs
    // More on the semantics soon
    setMeshOutputs(3, 1);
    if i == 0 {
        // Set per-primitive properties
        var p: PrimitiveOutput;
        p.cull = false;
        p.colorMask = vec4<f32>(1.0, 0.0, 1.0, 1.0);
        setPrimitive(0, p);

        // Set vertex indices for the triangle
        setPrimitiveIndices(0, vec3<u32>(0, 1, 2));
    }

    // Set the vertices
    var v: VertexOutput;
    v.position = vec4(f32(i) - 1.0, f32(i & 1u) * 2.0 - 1.0, 0.0, 1.0);
    v.color = vec4(f32(i == 0u), f32(i == 1u), f32(i == 2u), 1.0);
    setVertex(i, v);
}

struct VertexInput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}
struct PrimitiveInput {
    // A non-interpolated per-primitive value
    @per_primitive @location(1) colorMask: vec4<f32>,
}

@fragment
fn fs_main(vertex: VertexInput, primitive: PrimitiveInput) -> @location(0) vec4<f32> {
    return vertex.color * primitive.colorMask;
}
