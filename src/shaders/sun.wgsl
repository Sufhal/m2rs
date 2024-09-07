struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;


struct TransformUniform {
    transform: mat4x4<f32>,
};
@group(1) @binding(0) var<uniform> transform: TransformUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) position: vec3<f32>,
}

// Paramètres de rotation
const PI: f32 = 3.14159;


fn getRotationXMatrix(angle: f32) -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(1., 0., 0., 0.),
        vec4<f32>(0., cos(angle), -sin(angle), 0.),
        vec4<f32>(0., sin(angle), cos(angle), 0.),
        vec4<f32>(0., 0., 0., 1.)
    );
}

fn getRotationYMatrix(angle: f32) -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(cos(angle), 0., sin(angle), 0.),
		vec4<f32>(0., 1., 0., 0.),
		vec4<f32>(-sin(angle), 0., cos(angle), 0.),
		vec4<f32>(0., 0., 0., 1.)
    );
}

fn getRotationZMatrix(angle: f32) -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(cos(angle), -sin(angle), 0., 0.),
        vec4<f32>(sin(angle), cos(angle), 0., 0.),
        vec4<f32>(0., 0., 1., 0.),
        vec4<f32>(0., 0., 0., 1.)
    );
}

fn rotateX(originalMatrix: mat4x4<f32>, angle: f32) -> mat4x4<f32> {
    return originalMatrix * (
        getRotationXMatrix(angle) * 
        getRotationYMatrix(0.) * 
        getRotationZMatrix(0.)
    );
}

fn rotateY(originalMatrix: mat4x4<f32>, angle: f32) -> mat4x4<f32> {
    return originalMatrix * (
        getRotationXMatrix(0.) * 
        getRotationYMatrix(angle) * 
        getRotationZMatrix(0.)
    );
}

fn rotateZ(originalMatrix: mat4x4<f32>, angle: f32) -> mat4x4<f32> {
    return originalMatrix * (
        getRotationXMatrix(0.) * 
        getRotationYMatrix(0.) * 
        getRotationZMatrix(angle)
    );
}

fn billboardSpherical(modelViewOriginalMatrix: mat4x4<f32>) -> mat4x4<f32> {
    let d: f32 = length(modelViewOriginalMatrix[0].xyz);
    var m = modelViewOriginalMatrix;
    m[0][0] = d;
    m[0][1] = 0.;
    m[0][2] = 0.;
    m[1][0] = 0.; 
    m[1][1] = d; 
    m[1][2] = 0.; 
    m[2][0] = 0.;
    m[2][1] = 0.;
    m[2][2] = d;
    return m;
}

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.world_normal = model.normal;
    
    var transformed_model_matrix = transform.transform;

    out.position = model.position;

    var transformed_model_view_matrix = camera.view_matrix * transformed_model_matrix;
    transformed_model_view_matrix = billboardSpherical(transformed_model_view_matrix);
    transformed_model_view_matrix = rotateX(transformed_model_view_matrix, PI / 2.0);

    let model_view_position: vec4<f32> = transformed_model_view_matrix * vec4<f32>(model.position, 1.0); 
    out.clip_position = camera.projection_matrix *  model_view_position;

    return out;
}

@group(1) @binding(1) var sampler_tex: sampler;
@group(1) @binding(2) var texture_sun: texture_2d<f32>;

fn normalize_value_between(value: f32, min: f32, max: f32) -> f32 {
    return (value - min) / (max - min);
}

fn easeOutExpo(x: f32) -> f32 {
    if x == 1.0 {
        return 1.0;
    } else {
        return 1.0 - pow(2.0, -10.0 * x);
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = textureSample(texture_sun, sampler_tex, in.tex_coords);
    return vec4<f32>(1.0, 0.0, 0.0, t.r);
    // return vec4<f32>(sun.color.rgb, easeOutExpo(t.r));
    // return vec4<f32>(1.0, 0.0, 0.0, easeOutExpo(t.r));
    // return t;
    // return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}