use cgmath::Point3;

use crate::scene::material::Material;

pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
    pub material: Material,
}
