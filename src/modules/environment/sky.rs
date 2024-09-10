use crate::modules::{core::model::CustomMesh, geometry::sphere::Sphere, state::State};

use super::environment::MsEnv;

type Gradient = [[f32; 4]; 6];

pub struct Sky {
    uniform: SkyUniform,
    pub mesh: CustomMesh,
}

impl Sky {
    pub fn new(day_msenv: &MsEnv, night_msenv: &MsEnv, state: &State<'_>) -> Self {
        let uniform = SkyUniform {
            d_c0: day_msenv.sky_box.gradient[0],
            d_c1: day_msenv.sky_box.gradient[1],
            d_c2: day_msenv.sky_box.gradient[2],
            d_c3: day_msenv.sky_box.gradient[3],
            d_c4: day_msenv.sky_box.gradient[4],
            d_c5: day_msenv.sky_box.gradient[5],   
            n_c0: night_msenv.sky_box.gradient[0],
            n_c1: night_msenv.sky_box.gradient[1],
            n_c2: night_msenv.sky_box.gradient[2],
            n_c3: night_msenv.sky_box.gradient[3],
            n_c4: night_msenv.sky_box.gradient[4],
            n_c5: night_msenv.sky_box.gradient[5], 
        };
        let sphere = Sphere::new(10000.0, 50, 50);
        let mesh = sphere.to_sky_mesh(
            &state.device, 
            &state.sky_pipeline, 
            // [385.0, 186.0, 641.0],
            [0.0, 0.0, 0.0],
            uniform.clone()
        );
        Self {
            uniform,
            mesh,
        }
    }
}


#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct SkyUniform {
    pub d_c0: [f32; 4],
    pub d_c1: [f32; 4],
    pub d_c2: [f32; 4],
    pub d_c3: [f32; 4],
    pub d_c4: [f32; 4],
    pub d_c5: [f32; 4],
    pub n_c0: [f32; 4],
    pub n_c1: [f32; 4],
    pub n_c2: [f32; 4],
    pub n_c3: [f32; 4],
    pub n_c4: [f32; 4],
    pub n_c5: [f32; 4],
}