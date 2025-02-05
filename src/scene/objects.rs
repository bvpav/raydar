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

impl Object {
    pub fn default_sphere() -> Self {
        Self {
            geometry: Geometry::Sphere(Sphere {
                center: Point3::new(0.0, 0.0, 0.0),
                radius: 1.0,
            }),
            material: Material::default(),
        }
    }

    pub fn default_cube() -> Self {
        Self {
            geometry: Geometry::Cube(Cube {
                center: Point3::new(0.0, 0.0, 0.0),
                side_length: 2.0,
            }),
            material: Material::default(),
        }
    }
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
