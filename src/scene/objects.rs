use cgmath::Point3;
use serde::{Deserialize, Serialize};

use crate::scene::material::Material;

#[derive(Serialize, Deserialize, Clone)]
pub enum Geometry {
    Sphere(Sphere),
    Cube(Cube),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Object {
    pub geometry: Geometry,
    pub material: Material,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cube {
    pub center: Point3<f32>,
    pub side_length: f32,
}
