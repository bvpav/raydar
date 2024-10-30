use std::time::{Duration, Instant};

use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector2, Vector3, Vector4, Zero};
use image::{ImageBuffer, Rgba};

use crate::scene::{objects::Sphere, Scene};

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
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

        let discriminant = k * k - a * c;

        return if discriminant < 0.0 {
            None
        } else {
            let sqrt_discriminant = discriminant.sqrt();
            let t1 = (-k - sqrt_discriminant) / a;
            let t2 = (-k + sqrt_discriminant) / a;

            Some((t1, t2))
        };
    }

    fn at(&self, t: f32) -> Point3<f32> {
        self.origin + self.direction * t
    }
}

#[derive(Default)]
pub struct Renderer {
    pub last_frame_duration: Option<Duration>,
}

impl Renderer {
    pub fn render_frame(&mut self, scene: &Scene) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut image = ImageBuffer::new(scene.camera.resolution_x(), scene.camera.resolution_y());

        let frame_started_at = Instant::now();
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let uv_coord = Vector2::new(
                x as f32 / scene.camera.resolution_x() as f32,
                1.0 - y as f32 / scene.camera.resolution_y() as f32,
            );
            let color = self.per_pixel(uv_coord, scene);
            *pixel = Rgba([
                (color.x * 255.0) as u8,
                (color.y * 255.0) as u8,
                (color.z * 255.0) as u8,
                (color.w * 255.0) as u8,
            ]);
        }
        self.last_frame_duration = Some(frame_started_at.elapsed());

        image
    }

    fn per_pixel(&self, uv_coord: Vector2<f32>, scene: &Scene) -> Vector4<f32> {
        let clip_space_point = (uv_coord * 2.0 - Vector2::new(1.0, 1.0))
            .extend(-1.0)
            .extend(-1.0);
        let camera_space_point = scene.camera.inverse_proj_matrix() * clip_space_point;
        let camera_space_point = camera_space_point / camera_space_point.w;

        let world_space_direction = scene.camera.inverse_view_matrix() * camera_space_point;

        let ray = Ray {
            origin: scene.camera.position(),
            // TODO: maybe use swizzling (needs feature to be enabled)
            direction: -Vector3::new(
                world_space_direction.x,
                world_space_direction.y,
                world_space_direction.z,
            )
            .normalize(),
        };

        return if let Some((t1, _)) = ray.hit(&scene.sphere) {
            // assert!(t1 >= 0.0, "sphere is behind the camera");
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
