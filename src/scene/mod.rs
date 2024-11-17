use camera::{Camera, Projection};
use cgmath::{Deg, Point3, Vector3};
use material::Material;
use objects::Sphere;

pub mod camera;
pub mod material;
pub mod objects;

pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub camera: Camera,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            camera: Camera::new(
                Point3::new(0.0, 0.0, -1.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::unit_y(),
                854,
                480,
                0.01,
                1000.0,
                Projection::Perspective { fov: Deg(90.0) },
                // Projection::Orthographic { size: 1000.0 },
            ),
            spheres: vec![
                Sphere {
                    center: Point3::new(-0.5, 0.0, 0.0),
                    radius: 0.5,
                    material: Material::with_albedo(Vector3::new(0.8, 0.3, 0.3)),
                },
                Sphere {
                    center: Point3::new(0.5, 0.0, 0.0),
                    radius: 0.5,
                    material: Material::with_albedo(Vector3::new(0.3, 0.2, 0.8)),
                },
            ],
        }
    }
}
