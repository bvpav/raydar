use cgmath::{ElementWise, EuclideanSpace, InnerSpace, Point3, Vector2, Vector3, Vector4, Zero};
use color_eyre::eyre::{Context, Report};
use image::{ImageBuffer, Rgba};

struct Sphere {
    center: Point3<f32>,
    radius: f32,
}

struct Ray {
    origin: Point3<f32>,
    direction: Vector3<f32>,
}

impl Ray {
    fn hit(&self, sphere: &Sphere) -> Option<(f32, f32)> {
        let origin_vec = self.origin.to_vec();
        let sphere_center_vec = sphere.center.to_vec();

        // Quadratic equation in the form: a*t^2 + 2k*t + c = 0
        let a = self.direction.dot(self.direction);
        let k = origin_vec.dot(self.direction) - self.direction.dot(sphere_center_vec);
        let c = origin_vec.dot(origin_vec) - 2.0 * origin_vec.dot(sphere_center_vec)
            + sphere_center_vec.dot(sphere_center_vec)
            - sphere.radius * sphere.radius;

        let discriminant_squared = k * k - a * c;

        return if discriminant_squared < 0.0 {
            None
        } else {
            let discriminant = discriminant_squared.sqrt();
            let t1 = (-k - discriminant) / a;
            let t2 = (-k + discriminant) / a;

            if t1 < t2 {
                Some((t1, t2))
            } else {
                Some((t2, t1))
            }
        };
    }

    fn at(&self, t: f32) -> Point3<f32> {
        self.origin + self.direction * t
    }
}

struct Scene {
    resolution_x: u32,
    resolution_y: u32,

    sphere: Sphere,
}

struct Renderer;

impl Renderer {
    fn render_frame(&self, scene: &Scene) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut image = ImageBuffer::new(scene.resolution_x, scene.resolution_y);

        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let uv_coord = Vector2::new(
                x as f32 / scene.resolution_x as f32,
                1.0 - y as f32 / scene.resolution_y as f32,
            );
            let color = self.per_pixel(uv_coord, scene);
            *pixel = Rgba([
                (color.x * 255.0) as u8,
                (color.y * 255.0) as u8,
                (color.z * 255.0) as u8,
                (color.w * 255.0) as u8,
            ]);
        }

        image
    }

    fn per_pixel(&self, uv_coord: Vector2<f32>, scene: &Scene) -> Vector4<f32> {
        let aspect_ratio = scene.resolution_x as f32 / scene.resolution_y as f32;
        // let ray = Ray {
        //     origin: Point3::new(
        //         uv_coord.x * aspect_ratio * 2.0 - aspect_ratio,
        //         uv_coord.y * 2.0 - 1.0,
        //         -1.0,
        //     ),
        //     direction: Vector3::new(0.0, 0.0, 1.0),
        // };

        let ray = Ray {
            origin: Point3::new(0.0, 0.0, -1.0),
            direction: (uv_coord.mul_element_wise(Vector2::new(2.0 * aspect_ratio, 2.0))
                - Vector2::new(aspect_ratio, 1.0))
            .extend(1.0),
        };

        return if let Some((t1, _)) = ray.hit(&scene.sphere) {
            assert!(t1 >= 0.0, "sphere is behind the camera");
            let hit_point = ray.at(t1);
            let normal = (hit_point - scene.sphere.center).normalize();

            let light_direction = Vector3::new(-1.0, -1.0, 0.6).normalize();
            let cosine_similarity = normal.dot(-light_direction);

            (Vector3::new(1.0, 0.0, 1.0) * (cosine_similarity + 1.0) * 0.5).extend(1.0)
        } else {
            // let up = Vector3::unit_y();
            // let cosine_similarity =
            //     ray.direction.dot(up) / (ray.direction.magnitude() * up.magnitude());

            // let top_color = Vector4::new(0.53, 0.8, 0.92, 1.0);
            // let bottom_color = Vector4::new(1.0, 1.0, 1.0, 1.0);

            // bottom_color.lerp(top_color, (cosine_similarity + 1.0) * 0.5)

            Vector3::zero().extend(1.0)
        };
    }
}

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let scene = Scene {
        resolution_x: 854,
        resolution_y: 480,
        sphere: Sphere {
            center: Point3::new(0.0, 0.0, 0.0),
            radius: 0.5,
        },
    };

    let renderer = Renderer;

    let image = renderer.render_frame(&scene);
    image.save("output.png").wrap_err("Cannot save image")?;

    Ok(())
}
