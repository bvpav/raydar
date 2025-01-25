use camera::{Camera, Projection};
use cgmath::{Deg, Point3, Vector3};
use material::Material;
use objects::{Cube, Geometry, Object, Sphere};
use world::World;

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
                Point3::new(0.0, 0.0, 2.0), // Positioned to view the triangle vertices
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
                        center: Point3::new(-0.5, -0.25, 0.0),
                        radius: 0.05,
                    }),
                    material: Material::with_albedo(Vector3::new(1.0, 0.0, 0.0)),
                },
                Object {
                    geometry: Geometry::Sphere(Sphere {
                        center: Point3::new(0.0, 0.5, 0.0),
                        radius: 0.05,
                    }),
                    material: Material::with_albedo(Vector3::new(0.0, 1.0, 0.0)),
                },
                Object {
                    geometry: Geometry::Sphere(Sphere {
                        center: Point3::new(0.25, -0.1, 0.0),
                        radius: 0.05,
                    }),
                    material: Material::with_albedo(Vector3::new(0.0, 0.0, 1.0)),
                },
            ],
        }
    }
}
