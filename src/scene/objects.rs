use cgmath::Point3;
use serde::{Deserialize, Serialize};

use crate::scene::material::Material;

#[derive(Serialize, Deserialize)]
pub enum Geometry {
    Sphere(Sphere),
    Cube(Cube),
}

#[derive(Serialize, Deserialize)]
pub struct Object {
    pub geometry: Geometry,
    pub material: Material,
}

#[derive(Serialize, Deserialize)]
pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Cube {
    pub center: Point3<f32>,
    pub side_length: f32,
}
