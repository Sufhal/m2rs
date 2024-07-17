use super::buffer::ToMesh;

struct Plane {
    width: f32,
    height: f32,
    width_segments: f32,
    height_segments: f32,
    indices: Vec<u32>,
    vertices: Vec<f32>,
    normals: Vec<f32>,
}

impl Plane {
    
    pub fn new(width: f32, height: f32, width_segments: f32, height_segments: f32) -> Plane {

        let width_half = width / 2.0;
		let height_half = height / 2.0;

		let grid_x = width_segments.floor() as u32;
		let grid_y = height_segments.floor() as u32;

		let grid_x1 = grid_x + 1;
		let grid_y1 = grid_y + 1;

		let segment_width = width / grid_x as f32;
		let segment_height = height / grid_y as f32;

        let mut indices = Vec::<u32>::new();
        let mut vertices = Vec::<f32>::new();
        let mut normals = Vec::<f32>::new();
        let mut uvs = Vec::<f32>::new();

        for iy in 0..grid_y1 {
			let y = iy as f32 * segment_height - height_half;
            for ix in 0..grid_x1 {
				let x = ix as f32 * segment_width - width_half;
                vertices.push(x);
                vertices.push(-y);
                vertices.push(0.0);
				normals.push(0.0);
				normals.push(0.0);
				normals.push(1.0);
				uvs.push((ix / grid_x) as f32);
				uvs.push((1 - ( iy / grid_y)) as f32);
			}
		}

        for iy in 0..grid_y {
            for ix in 0..grid_x {
				let a = ix + grid_x1 * iy;
				let b = ix + grid_x1 * ( iy + 1 );
				let c = ( ix + 1 ) + grid_x1 * ( iy + 1 );
				let d = ( ix + 1 ) + grid_x1 * iy;
                indices.push(a);
                indices.push(b);
                indices.push(c);
                indices.push(b);
                indices.push(c);
                indices.push(d);
			}
		}

        Plane {
            width,
            height,
            width_segments,
            height_segments,
            indices,
            vertices,
            normals
        }

    }

}

impl ToMesh for Plane {}