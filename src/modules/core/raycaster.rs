use cgmath::{InnerSpace, Vector3};
use super::model::SimpleVertex;

pub struct Raycaster {
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    is_vertical: bool,
}

impl Raycaster {
    pub fn new(origin: [f32; 3], direction: [f32; 3]) -> Self {
        Self {
            origin: origin.into(),
            direction: direction.into(),
            is_vertical: direction[0] == 0.0 && direction[2] == 0.0
        }
    }
    pub fn intersects_first(&self, vertices: &[SimpleVertex], indices: &[u32]) -> Option<f32> {
        for i in (0..indices.len()).step_by(3) {
            let triangle = [
                vertices[indices[i + 0] as usize],
                vertices[indices[i + 1] as usize],
                vertices[indices[i + 2] as usize],
            ];
            if self.is_vertical && !self.is_triangle_near_ray(&triangle, 10.0) { 
                continue;
            }
            let intersection = self.intersects_triangle(triangle);
            if intersection.is_some() {
                return intersection
            }
        }
        None
    }
    /// Based on [Möller–Trumbore intersection algorithm](https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm)
    /// Returns the distance between the origin and the intersection point
    fn intersects_triangle(&self, triangle: [SimpleVertex; 3]) -> Option<f32> {
        let t0 = Vector3::from(triangle[0].position);
        let t1 = Vector3::from(triangle[1].position);
        let t2 = Vector3::from(triangle[2].position);

        let e1 = t1 - t0;
        let e2 = t2 - t0;
    
        let ray_cross_e2 = self.direction.cross(e2);
        let det = e1.dot(ray_cross_e2);
    
        if det > -f32::EPSILON && det < f32::EPSILON {
            return None; // This ray is parallel to this triangle.
        }
    
        let inv_det = 1.0 / det;
        let s = self.origin - t0;
        let u = inv_det * s.dot(ray_cross_e2);
        if u < 0.0 || u > 1.0 {
            return None;
        }
    
        let s_cross_e1 = s.cross(e1);
        let v = inv_det * self.direction.dot(s_cross_e1);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        // At this stage we can compute t to find out where the intersection point is on the line.
        let t = inv_det * e2.dot(s_cross_e1);
    
        if t > f32::EPSILON { // ray intersection
            return Some(t);
        }
        else { // This means that there is a line intersection but not a ray intersection.
            return None;
        }
    }

    fn is_triangle_near_ray(&self, triangle: &[SimpleVertex; 3], threshold: f32) -> bool {
        let ray_x = self.origin.x;
        let ray_z = self.origin.z;
        for vertex in triangle {
            let [vx, _, vz] = vertex.position;
            let dx = (vx - ray_x).abs();
            let dz = (vz - ray_z).abs();
                if dx <= threshold && dz <= threshold {
                return true;
            }
        }
        false
    }
}