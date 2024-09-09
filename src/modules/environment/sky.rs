use crate::modules::{core::model::CustomMesh, geometry::sphere::Sphere, state::State};

use super::environment::MsEnv;

type Gradient = [[f32; 4]; 5];

pub struct Sky {
    gradient: Gradient,
    pub mesh: CustomMesh,
}

impl Sky {
    pub fn new(msenv: &MsEnv, state: &State<'_>) -> Self {
        let uniform = SkyUniform {
            d_c0: msenv.sky_box.gradient[0],
            d_c1: msenv.sky_box.gradient[1],
            d_c2: msenv.sky_box.gradient[2],
            d_c3: msenv.sky_box.gradient[3],
            d_c4: msenv.sky_box.gradient[4],   
        };
        let sphere = Sphere::new(10000.0, 50, 50);
        let mesh = sphere.to_sky_mesh(
            &state.device, 
            &state.sky_pipeline, 
            // [385.0, 186.0, 641.0],
            [0.0, 0.0, 0.0],
            uniform
        );
        Self {
            gradient: msenv.sky_box.gradient,
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
}