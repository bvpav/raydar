use camera::{Camera, Projection};
use cgmath::{Deg, Point3, Vector3};
use material::Material;
use objects::{Cube, Geometry, Object, Sphere};
use world::World;

pub mod benchmark;
pub mod camera;
pub mod material;
pub mod objects;
pub mod world;

pub struct Scene {
    pub camera: Camera,
    pub world: World,
    pub objects: Vec<Object>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            camera: Camera::new(
                Point3::new(-3.09, 0.03, -1.16),
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
            objects: vec![
                Object {
                    geometry: Geometry::Sphere(Sphere {
                        center: Point3::new(0.0, 0.001, 0.0),
                        radius: 1.0,
                    }),
                    material: Material {
                        albedo: Vector3::new(1.0, 1.0, 1.0),
                        roughness: 0.2,
                        transmission: 1.0,
                        ..Default::default()
                    },
                },
                Object {
                    geometry: Geometry::Cube(Cube {
                        center: Point3::new(0.0, -101.0, 0.0),
                        side_length: 200.0,
                    }),
                    material: Material::with_albedo(Vector3::new(0.34, 0.34, 0.44)),
                },
                Object {
                    geometry: Geometry::Cube(Cube {
                        center: Point3::new(7.0, 3.0, 0.0),
                        side_length: 1.8,
                    }),
                    material: Material::with_emission(Vector3::new(0.8, 0.5, 0.2), 30.0),
                },
            ],
        }
    }
}
