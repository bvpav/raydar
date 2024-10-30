use camera::{Camera, Projection};
use cgmath::{Deg, Point3, Vector3};
use objects::Sphere;

pub mod camera;
pub mod objects;

pub struct Scene {
    pub sphere: Sphere,
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
            ),
            sphere: Sphere {
                center: Point3::new(0.0, 0.0, 0.0),
                radius: 0.5,
            },
        }
    }
}
