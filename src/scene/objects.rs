use cgmath::{Point3, Vector3};
use serde::{Deserialize, Serialize};

use crate::scene::material::Material;

use super::aabb::{HasBoundingBox, AABB};

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

impl HasBoundingBox for Object {
    fn aabb(&self) -> AABB {
        match &self.geometry {
            Geometry::Sphere(sphere) => sphere.aabb(),
            Geometry::Cube(cube) => cube.aabb(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
}

impl HasBoundingBox for Sphere {
    fn aabb(&self) -> AABB {
        AABB {
            min: self.center - Vector3::new(self.radius, self.radius, self.radius),
            max: self.center + Vector3::new(self.radius, self.radius, self.radius),
        }
    }
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Cube {
    pub center: Point3<f32>,
    pub side_length: f32,
}

impl HasBoundingBox for Cube {
    fn aabb(&self) -> AABB {
        let half_size = Vector3::new(
            self.side_length * 0.5,
            self.side_length * 0.5,
            self.side_length * 0.5,
        );
        AABB {
            min: self.center - half_size,
            max: self.center + half_size,
        }
    }
}
