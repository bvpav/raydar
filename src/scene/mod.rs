use camera::{Camera, Projection};
use cgmath::{Deg, Point3, Vector3};
use material::Material;
use objects::Sphere;
use world::World;

pub mod camera;
pub mod material;
pub mod objects;
pub mod world;

pub struct Scene {
    pub camera: Camera,
    pub world: World,
    pub spheres: Vec<Sphere>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            camera: Camera::new(
                Point3::new(-2.0, 0.0, -2.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::unit_y(),
                854,
                480,
                0.01,
                1000.0,
                Projection::Perspective { fov: Deg(90.0) },
                // Projection::Orthographic { size: 1000.0 },
            ),
            world: World::SkyColor {
                top_color: Vector3::new(0.53, 0.8, 0.92),
                bottom_color: Vector3::new(1.0, 1.0, 1.0),
            },
            spheres: vec![
                Sphere {
                    center: Point3::new(0.0, 0.0, 0.0),
                    radius: 1.0,
                    material: Material::with_albedo(Vector3::new(1.0, 0.0, 0.16)),
                },
                Sphere {
                    center: Point3::new(0.0, -101.0, 0.0),
                    radius: 100.0,
                    material: Material::with_albedo(Vector3::new(0.1, 0.1, 0.5)),
                },
                Sphere {
                    center: Point3::new(2.0, 0.0, 0.0),
                    radius: 1.0,
                    material: Material::with_emission(Vector3::new(0.8, 0.5, 0.2), 3.0),
                },
            ],
        }
    }
}
