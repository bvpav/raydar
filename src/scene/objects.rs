use cgmath::Point3;

use crate::scene::material::Material;

pub enum Geometry {
    Sphere(Sphere),
}

pub struct Object {
    pub geometry: Geometry,
    pub material: Material,
}

pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
}
