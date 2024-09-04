use cgmath::{InnerSpace, MetricSpace, Vector3};
use super::model::SimpleVertex;

pub struct Raycast {
    origin: Vector3<f32>,
    direction: Vector3<f32>,
}

impl Raycast {
    pub fn new(origin: [f32; 3], direction: [f32; 3]) -> Self {
        Self {
            origin: origin.into(),
            direction: direction.into()
        }
    }
    /// Based on [Möller–Trumbore intersection algorithm](https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm)
    /// Returns the distance between the origin and the intersection point
    pub fn intersects(&self, triangle: [SimpleVertex; 3]) -> Option<f32> {
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
            let intersection_point = self.origin + self.direction * t;
            return Some(self.origin.distance2(intersection_point));
        }
        else { // This means that there is a line intersection but not a ray intersection.
            return None;
        }
    }
}