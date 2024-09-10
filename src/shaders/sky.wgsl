struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct Cycle {
    day_factor: f32,
    night_factor: f32,
}
@group(0) @binding(2) var<uniform> cycle: Cycle;

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
    let model_view_position: vec4<f32> = transformed_model_view_matrix * vec4<f32>(model.position, 1.0); 
    out.clip_position = camera.projection_matrix *  model_view_position;

    return out;
}

fn normalize_value_between(value: f32, min: f32, max: f32) -> f32 {
    return (value - min) / (max - min);
}

fn ease_out_expo(x: f32) -> f32 {
    if x == 1.0 {
        return 1.0;
    } else {
        return 1.0 - pow(2.0, -10.0 * x);
    }
}

fn ease_out_quart(x: f32) -> f32 {
    return 1.0 - pow(1.0 - x, 4.0);
}


struct Sky {
    d_c0: vec4<f32>,
    d_c1: vec4<f32>,
    d_c2: vec4<f32>,
    d_c3: vec4<f32>,
    d_c4: vec4<f32>,
    d_c5: vec4<f32>,
    n_c0: vec4<f32>,
    n_c1: vec4<f32>,
    n_c2: vec4<f32>,
    n_c3: vec4<f32>,
    n_c4: vec4<f32>,
    n_c5: vec4<f32>,
}
@group(1) @binding(1) var<uniform> sky: Sky;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    var sun_light_factor: f32 = 0.0;

    if cycle.day_factor > 0.0 && cycle.day_factor <= 0.5  {
        sun_light_factor = ease_out_expo(normalize_value_between(cycle.day_factor, 0.0, 0.5));
    } 
    else if cycle.day_factor > 0.5 && cycle.day_factor <= 1.0 {
        sun_light_factor = ease_out_expo(normalize_value_between(1.0 - cycle.day_factor, 0.0, 0.5));
    }

	let p = 1.0 / 5.0;
	let n = ((in.tex_coords.y) * 1.6) / p;
	let c1 = n - 1.0;
	let c2 = n - 2.0;
	let c3 = n - 3.0;
	let c4 = n - 4.0;

    // return sky.d_c0;

	if (n >= 0.0 && n <= 1.0) {
		return mix(
            mix(sky.n_c0, sky.n_c1, n),
            mix(sky.d_c0, sky.d_c1, n),
            sun_light_factor
        );
    }
	if (c1 >= 0.0 && c1 <= 1.0) {
		return mix(
            mix(sky.n_c1, sky.n_c2, c1),
            mix(sky.d_c1, sky.d_c2, c1),
            sun_light_factor
        );
    }
	if (c2 >= 0.0 && c2 <= 1.0) {
		return mix(
            mix(sky.n_c2, sky.n_c3, c2),
            mix(sky.d_c2, sky.d_c3, c2),
            sun_light_factor
        );
    }
	if (c3 >= 0.0 && c3 <= 1.0) {
		return mix(
            mix(sky.n_c3, sky.n_c4, c3),
            mix(sky.d_c3, sky.d_c4, c3),
            sun_light_factor
        );
    }
	if (c4 >= 0.0 && c4 <= 1.0) {
		return mix(
            mix(sky.n_c4, sky.n_c5, c4),
            mix(sky.d_c4, sky.d_c5, c4),
            sun_light_factor
        );
    }

    return vec4<f32>(mix(sky.n_c5, sky.d_c5, sun_light_factor).rgb, 1.0);
}