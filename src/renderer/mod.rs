use std::time::{Duration, Instant};

use cgmath::{ElementWise, EuclideanSpace, InnerSpace, Point3, Vector2, Vector3, Vector4, Zero};
use image::{ImageBuffer, Rgba, Rgba32FImage, RgbaImage};

use crate::{
    scene::{objects::Sphere, Scene},
    utils,
};

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
}

impl Ray {
    fn hit(&self, sphere: &Sphere) -> Option<f32> {
        let origin_vec = self.origin.to_vec();
        let sphere_center_vec = sphere.center.to_vec();

        // Quadratic equation in the form: a*t^2 + 2k*t + c = 0
        let a = self.direction.dot(self.direction);
        let k = origin_vec.dot(self.direction) - self.direction.dot(sphere_center_vec);
        let c = origin_vec.dot(origin_vec) - 2.0 * origin_vec.dot(sphere_center_vec)
            + sphere_center_vec.dot(sphere_center_vec)
            - sphere.radius * sphere.radius;

        let discriminant = k * k - a * c;

        if discriminant < 0.0 {
            None
        } else {
            let sqrt_discriminant = discriminant.sqrt();
            let t1 = (-k - sqrt_discriminant) / a;
            let t2 = (-k + sqrt_discriminant) / a;

            if t1 >= 0.0 {
                Some(t1)
            } else if t2 >= 0.0 {
                Some(t2)
            } else {
                None
            }
        }
    }

    fn at(&self, t: f32) -> Point3<f32> {
        self.origin + self.direction * t
    }
}

struct HitRecord<'a> {
    #[allow(unused)]
    hit_distance: f32,
    world_position: Point3<f32>,
    world_normal: Vector3<f32>,
    sphere: &'a Sphere,
}

const MAX_SAMPLE_COUNT: usize = 1024;
const MAX_BOUNCES: usize = 2;

#[derive(Default)]
pub struct Renderer {
    pub last_frame_start: Option<Instant>,
    pub last_frame_duration: Option<Duration>,
    pub last_sample_duration: Option<Duration>,
    pub sample_count: usize,

    frame_buffer: Option<Rgba32FImage>,
}

impl Renderer {
    pub fn render_frame(&mut self, scene: &Scene) -> RgbaImage {
        let mut rendered_frame =
            ImageBuffer::new(scene.camera.resolution_x(), scene.camera.resolution_y());

        self.new_frame(scene);
        let mut frame_buffer = self.frame_buffer(scene);

        while self.sample_count < MAX_SAMPLE_COUNT {
            self.render_next_sample(scene, &mut frame_buffer);
        }
        self.print_frame_buffer(&frame_buffer, &mut rendered_frame);

        self.frame_buffer = Some(frame_buffer);
        rendered_frame
    }

    pub fn new_frame(&mut self, scene: &Scene) {
        self.frame_buffer = Some(self.blank_frame_buffer(scene));
        self.last_frame_start = Some(Instant::now());
        self.sample_count = 0;
    }

    pub fn render_sample(&mut self, scene: &Scene) -> Option<RgbaImage> {
        if self.sample_count >= MAX_SAMPLE_COUNT {
            return None;
        }

        let mut frame_buffer = self.frame_buffer(scene);

        let mut rendered_frame =
            ImageBuffer::new(scene.camera.resolution_x(), scene.camera.resolution_y());

        self.render_next_sample(scene, &mut frame_buffer);
        self.print_frame_buffer(&frame_buffer, &mut rendered_frame);

        self.frame_buffer = Some(frame_buffer);

        Some(rendered_frame)
    }

    pub fn max_sample_count(&self) -> usize {
        MAX_SAMPLE_COUNT
    }

    fn render_next_sample(&mut self, scene: &Scene, frame_buffer: &mut Rgba32FImage) {
        let sample_started_at = Instant::now();

        for (x, y, pixel) in frame_buffer.enumerate_pixels_mut() {
            let uv_coord = Vector2::new(
                x as f32 / scene.camera.resolution_x() as f32,
                1.0 - y as f32 / scene.camera.resolution_y() as f32,
            );
            let color = self.per_pixel(uv_coord, scene);
            *pixel = Rgba([
                (pixel[0] + color.x),
                (pixel[1] + color.y),
                (pixel[2] + color.z),
                (pixel[3] + color.w),
            ]);
        }

        self.sample_count += 1;
        if self.sample_count == MAX_SAMPLE_COUNT {
            if let Some(last_frame_start) = self.last_frame_start {
                self.last_frame_duration = Some(last_frame_start.elapsed());
            }
        }

        self.last_sample_duration = Some(sample_started_at.elapsed());
    }

    fn print_frame_buffer(&self, frame_buffer: &Rgba32FImage, image: &mut RgbaImage) {
        for (fb_pixel, rendered_pixel) in frame_buffer.pixels().zip(image.pixels_mut()) {
            *rendered_pixel = Rgba([
                ((fb_pixel[0] / self.sample_count as f32).clamp(0.0, 1.0) * 255.0) as u8,
                ((fb_pixel[1] / self.sample_count as f32).clamp(0.0, 1.0) * 255.0) as u8,
                ((fb_pixel[2] / self.sample_count as f32).clamp(0.0, 1.0) * 255.0) as u8,
                ((fb_pixel[3] / self.sample_count as f32).clamp(0.0, 1.0) * 255.0) as u8,
            ]);
        }
    }

    fn per_pixel(&self, uv_coord: Vector2<f32>, scene: &Scene) -> Vector4<f32> {
        let clip_space_point = (uv_coord * 2.0 - Vector2::new(1.0, 1.0))
            .extend(-1.0)
            .extend(-1.0);
        let camera_space_point = scene.camera.inverse_proj_matrix() * clip_space_point;
        let camera_space_point = camera_space_point / camera_space_point.w;

        let world_space_direction = scene.camera.inverse_view_matrix() * camera_space_point;

        let mut ray = Ray {
            origin: scene.camera.position(),
            // TODO: maybe use swizzling (needs feature to be enabled)
            direction: -Vector3::new(
                world_space_direction.x,
                world_space_direction.y,
                world_space_direction.z,
            )
            .normalize(),
        };

        let mut light = Vector3::zero();
        let mut throughput = Vector3::new(1.0, 1.0, 1.0);

        for _ in 0..MAX_BOUNCES {
            if let Some(hit_record) = self.trace_ray(&ray, scene) {
                throughput = throughput.mul_element_wise(hit_record.sphere.material.albedo);
                light += hit_record.sphere.material.emission_color
                    * hit_record.sphere.material.emission_strength;

                ray = Ray {
                    origin: hit_record.world_position + hit_record.world_normal * 0.0001,
                    direction: utils::random_in_unit_hemisphere(hit_record.world_normal),
                }
            } else {
                light += scene.world.sample(ray).mul_element_wise(throughput);
                break;
            };
        }

        light.extend(1.0)
    }

    fn trace_ray<'a>(&self, ray: &Ray, scene: &'a Scene) -> Option<HitRecord<'a>> {
        scene
            .spheres
            .iter()
            .filter_map(|s| ray.hit(s).map(|t| (s, t)))
            .min_by_key(|(_, t)| ordered_float::OrderedFloat(*t))
            .and_then(|(s, t)| self.closest_hit(ray, t, s))
            .or_else(|| self.miss(ray, scene))
    }

    fn closest_hit<'a>(
        &self,
        ray: &Ray,
        hit_distance: f32,
        sphere: &'a Sphere,
    ) -> Option<HitRecord<'a>> {
        let world_position = ray.at(hit_distance);
        let mut world_normal = (world_position - sphere.center).normalize();
        if world_normal.dot(ray.direction) > 0.0 {
            world_normal = -world_normal;
        }

        Some(HitRecord {
            hit_distance,
            world_position,
            world_normal,
            sphere,
        })
    }

    fn miss<'a>(&self, _ray: &Ray, _scene: &Scene) -> Option<HitRecord<'a>> {
        None
    }

    fn frame_buffer(&mut self, scene: &Scene) -> Rgba32FImage {
        self.frame_buffer
            .take()
            .map(|mut frame_buffer| {
                if frame_buffer.width() != scene.camera.resolution_x()
                    || frame_buffer.height() != scene.camera.resolution_y()
                {
                    frame_buffer =
                        Rgba32FImage::new(scene.camera.resolution_x(), scene.camera.resolution_y());
                }
                frame_buffer
            })
            .unwrap_or_else(|| {
                Rgba32FImage::new(scene.camera.resolution_x(), scene.camera.resolution_y())
            })
    }

    fn blank_frame_buffer(&mut self, scene: &Scene) -> Rgba32FImage {
        let mut frame_buffer = self.frame_buffer(scene);
        frame_buffer.fill(0.0);
        frame_buffer
    }
}
