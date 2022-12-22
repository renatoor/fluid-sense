struct Camera {
    position: vec4<f32>,
    view_matrix: mat4x4<f32>,
    view_projection: mat4x4<f32>,
}

struct Light {
    position: vec4<f32>,
    color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> light: Light;

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(2) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct Instance {
    @location(3) model_matrix_0: vec4<f32>,
    @location(4) model_matrix_1: vec4<f32>,
    @location(5) model_matrix_2: vec4<f32>,
    @location(6) model_matrix_3: vec4<f32>,
    @location(7) normal_matrix_0: vec3<f32>,
    @location(8) normal_matrix_1: vec3<f32>,
    @location(9) normal_matrix_2: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) model_scale: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: Instance,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );

    var out: VertexOutput;

    let scale = vec3<f32>(
        length(vec3<f32>(model_matrix[0].x, model_matrix[1].x, model_matrix[2].x)),
        length(vec3<f32>(model_matrix[0].y, model_matrix[1].y, model_matrix[2].y)),
        length(vec3<f32>(model_matrix[0].z, model_matrix[1].z, model_matrix[2].z)),
    );

    out.tex_coords = model.tex_coords;
    out.world_normal = normal_matrix * model.normal;
    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_projection * world_position;
    out.model_scale = scale;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords * in.model_scale.xy);

    let light_color = light.color.xyz;
    let light_position = light.position.xyz;
    let camera_position = camera.position.xyz;

    let light_dir = normalize(light_position - in.world_position);
    let view_dir = normalize(camera_position - in.world_position);
    let half_dir = normalize(view_dir + light_dir);

    let ambient = light_color * 0.1;

    let diffuse = light_color * max(dot(in.world_normal, light_dir), 0.0);

    //let specular = light_color * pow(max(dot(in.world_normal, half_dir), 0.0), 32.0);

    let result = (ambient + diffuse) * color.xyz;

    return vec4<f32>(result, 1.0);
}