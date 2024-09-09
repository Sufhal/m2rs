type Gradient = [[f32; 4]; 5];

pub struct Sky {
    gradient: Gradient,
}

impl Sky {
    pub fn new(colors: Gradient) -> Self {
        Self {
            gradient: colors
        }
    }
    pub fn uniform(&self) -> SkyUniform {
        SkyUniform {
            d_c0: self.gradient[0],
            d_c1: self.gradient[1],
            d_c2: self.gradient[2],
            d_c3: self.gradient[3],
            d_c4: self.gradient[4],
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