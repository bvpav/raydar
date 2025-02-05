use super::camera::{Camera, Projection};
use super::material::Material;
use super::objects::{Cube, Geometry, Object, Sphere};
use super::world::World;
use super::Scene;
use cgmath::{Deg, Point3, Vector3};

/// Creates a complex benchmark scene for testing the renderer's capabilities.
///
/// The scene consists of:
/// - A large dark floor
/// - A central glass sphere
/// - A spiral of metallic spheres that get rougher as they spiral outward
/// - A grid of rainbow-colored cubes arranged in a circle, with alternating metallic properties
/// - Small glass spheres arranged in a diamond pattern, getting slightly frosted towards the edges
/// - Three light sources: a warm light, a cool light, and a white top light
///
/// The scene demonstrates various material properties:
/// - Transmission (glass spheres)
/// - Metallic reflections (spiral spheres and some cubes)
/// - Roughness variation based on distance from center
/// - Rainbow colors with distance-based saturation
/// - Emission (light sources)
pub fn benchmark_scene() -> Scene {
    let camera = Camera::new(
        Point3::new(-8.0, 4.0, -8.0),
        Point3::new(0.0, 1.0, 0.0), // Look at point slightly above ground
        Vector3::unit_y(),
        3840,
        2160,
        0.01,
        1000.0,
        Projection::Perspective { fov: Deg(70.0) },
    );

    let world = World::SkyColor {
        top_color: Vector3::new(0.53, 0.8, 0.92),
        bottom_color: Vector3::new(1.0, 1.0, 1.0),
    };

    let mut objects = Vec::new();

    // Add floor at y=0
    objects.push(Object {
        geometry: Geometry::Cube(Cube {
            center: Point3::new(0.0, -10.0, 0.0),
            side_length: 20.0,
        }),
        material: Material {
            albedo: Vector3::new(0.2, 0.2, 0.25),
            roughness: 0.7,
            metallic: 0.0,
            ..Default::default()
        },
    });

    // Add central glass sphere - just above floor
    objects.push(Object {
        geometry: Geometry::Sphere(Sphere {
            center: Point3::new(0.0, 1.0, 0.0),
            radius: 1.0,
        }),
        material: Material {
            albedo: Vector3::new(1.0, 1.0, 1.0),
            roughness: 0.0,
            transmission: 1.0,
            ior: 1.5,
            ..Default::default()
        },
    });

    // Add spiral of metallic spheres - hovering just above floor
    for i in 0..50 {
        let angle = i as f32 * std::f32::consts::PI * 0.25;
        let radius = 3.0 + (i as f32 * 0.15);
        let height = 0.3 + (i as f32 * 0.05);

        let roughness = (radius / 10.0).min(0.9);

        objects.push(Object {
            geometry: Geometry::Sphere(Sphere {
                center: Point3::new(angle.cos() * radius, height, angle.sin() * radius),
                radius: 0.3,
            }),
            material: Material {
                albedo: Vector3::new(0.95, 0.95, 0.95),
                roughness,
                metallic: 1.0,
                ..Default::default()
            },
        });
    }

    // Add grid of colored cubes - sitting on floor
    for x in -5..=5 {
        for z in -5..=5 {
            if (x as i32).abs() <= 1 && (z as i32).abs() <= 1 {
                continue;
            }

            let pos_x = x as f32 * 1.5;
            let pos_z = z as f32 * 1.5;
            let height = ((x * x + z * z) as f32).sqrt() * 0.05;

            let angle = pos_z.atan2(pos_x);
            let distance = (pos_x * pos_x + pos_z * pos_z).sqrt();
            let distance_factor = (distance / 10.0).min(1.0);

            let hue = (angle / (std::f32::consts::PI * 2.0) + 0.5) % 1.0;
            let (r, g, b) = hsv_to_rgb(hue, 0.8 + distance_factor * 0.2, 0.9);
            let color = Vector3::new(r, g, b);

            let roughness = (distance / 15.0).min(0.8);

            objects.push(Object {
                geometry: Geometry::Cube(Cube {
                    center: Point3::new(pos_x, 0.25 + height, pos_z),
                    side_length: 0.5,
                }),
                material: Material {
                    albedo: color,
                    roughness,
                    metallic: if (x + z) % 2 == 0 { 0.8 } else { 0.0 },
                    ..Default::default()
                },
            });
        }
    }

    // Add small glass spheres in a diamond pattern - hovering low
    for i in -3..=3 {
        for j in -3..=3 {
            if (i as i32).abs() + (j as i32).abs() > 3 {
                continue;
            }
            if (i as i32).abs() <= 1 && (j as i32).abs() <= 1 {
                continue;
            }

            let pos_x = i as f32;
            let pos_z = j as f32;
            let distance = (pos_x * pos_x + pos_z * pos_z).sqrt();

            // Glass gets slightly rougher towards the edges
            let roughness = (distance / 6.0).min(0.1); // Keep it mostly clear but add slight variation

            objects.push(Object {
                geometry: Geometry::Sphere(Sphere {
                    center: Point3::new(pos_x, 0.4, pos_z),
                    radius: 0.2,
                }),
                material: Material {
                    albedo: Vector3::new(1.0, 1.0, 1.0),
                    roughness,
                    transmission: 1.0,
                    ior: 1.5,
                    ..Default::default()
                },
            });
        }
    }

    let light_positions = [
        (5.0, 5.0, -5.0, Vector3::new(1.0, 0.9, 0.7), 15.0), // Warm color
        (-5.0, 4.0, 5.0, Vector3::new(0.7, 0.8, 1.0), 12.0), // Cool color
        (0.0, 6.0, 0.0, Vector3::new(1.0, 1.0, 1.0), 20.0),  // White
    ];

    for (x, y, z, color, strength) in light_positions.iter() {
        objects.push(Object {
            geometry: Geometry::Sphere(Sphere {
                center: Point3::new(*x, *y, *z),
                radius: 0.5,
            }),
            material: Material::with_emission(*color, *strength),
        });
    }

    Scene {
        camera,
        world,
        objects,
    }
}

/// Helper function to convert HSV to RGB
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let h = h * 6.0;
    let i = h.floor();
    let f = h - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    match i as i32 % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
