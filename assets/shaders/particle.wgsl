struct Camera {
    position: vec4<f32>,
    view_matrix: mat4x4<f32>,
    view_projection: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct Instance {
    @location(1) position: vec3<f32>,
    @location(2) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput, instance: Instance) -> VertexOutput {
    let camera_right: vec3<f32> = vec3<f32>(camera.view_matrix[0].x, camera.view_matrix[1].x, camera.view_matrix[2].x);
    let camera_up: vec3<f32> = vec3<f32>(camera.view_matrix[0].y, camera.view_matrix[1].y, camera.view_matrix[2].y);
    let position_world: vec3<f32> = instance.position + camera_right * model.position.x * 0.02 + camera_up * model.position.y * 0.02;
    var out: VertexOutput;
    var screenspace: vec4<f32> = camera.view_projection * vec4<f32>(position_world, 1.0);
    out.clip_position = screenspace;
    out.color = instance.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}