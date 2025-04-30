use cgmath::{ElementWise, EuclideanSpace, InnerSpace, Point3, Vector2, Vector3, Vector4, Zero};
use image::{ImageBuffer, Rgba, Rgba32FImage, RgbaImage};

use crate::{
    scene::{
        aabb::{HasBoundingBox, AABB},
        objects::{Cube, Geometry, Object, Sphere},
        Scene,
    },
    utils,
};

use self::utils::{Reflect, Refract};

use super::{timing::Profiler, Renderer, RendererConfig};

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
}

impl Ray {
    fn hit<'a>(&self, bvh_node: &'a BVHNode) -> Option<(f32, &'a Object)> {
        match &bvh_node.kind {
            BVHNodeKind::Internal(left, right) => [left, right]
                .into_iter()
                .filter_map(|n| self.hit_aabb(&n.aabb).and_then(|_| self.hit(n)))
                .min_by_key(|(t, _)| ordered_float::OrderedFloat(*t)),
            BVHNodeKind::Leaf(o) => self
                .hit_aabb(&bvh_node.aabb)
                .and_then(|_| self.hit_object(o).map(|t| (t, o))),
        }
    }

    fn hit_object(&self, object: &Object) -> Option<f32> {
        match &object.geometry {
            Geometry::Sphere(sphere) => self.hit_sphere(sphere),
            Geometry::Cube(cube) => self.hit_cube(cube),
        }
    }

    fn at(&self, t: f32) -> Point3<f32> {
        self.origin + self.direction * t
    }

    fn hit_sphere(&self, sphere: &Sphere) -> Option<f32> {
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

    fn hit_cube(&self, cube: &Cube) -> Option<f32> {
        self.hit_aabb(&cube.aabb())
    }

    fn hit_aabb(&self, aabb: &AABB) -> Option<f32> {
        // Calculate intersection distances for each axis using vector operations
        let t1 = (aabb.min - self.origin).div_element_wise(self.direction);
        let t2 = (aabb.max - self.origin).div_element_wise(self.direction);

        // Find entry and exit points
        let tmin = t1.x.min(t2.x).max(t1.y.min(t2.y)).max(t1.z.min(t2.z));
        let tmax = t1.x.max(t2.x).min(t1.y.max(t2.y)).min(t1.z.max(t2.z));

        // If tmax < 0, ray is intersecting AABB, but entire AABB is behind us
        if tmax < 0.0 {
            return None;
        }

        // If tmin > tmax, ray doesn't intersect AABB
        if tmin > tmax {
            return None;
        }

        // If tmin < 0 then the ray starts inside the box
        // In this case we return the exit point (tmax)
        if tmin < 0.0 {
            Some(tmax)
        } else {
            Some(tmin)
        }
    }
}

#[derive(Clone)]
enum BVHNodeKind {
    Internal(Box<BVHNode>, Box<BVHNode>),
    Leaf(Object),
}

#[derive(Clone)]
struct BVHNode {
    aabb: AABB,
    kind: BVHNodeKind,
}

impl BVHNode {
    fn new(objects: &[Object]) -> Option<Self> {
        match objects {
            [] => None,
            [object] => Some(Self {
                aabb: object.aabb(),
                kind: BVHNodeKind::Leaf(object.clone()),
            }),
            _ => todo!(),
        }
    }
}

struct HitRecord<'a> {
    #[allow(unused)]
    hit_distance: f32,
    is_front_face: bool,
    world_position: Point3<f32>,
    world_normal: Vector3<f32>,
    object: &'a Object,
}

#[derive(Default)]
pub struct CpuRenderer {
    profiler: Profiler,
    frame_buffer: Option<Rgba32FImage>,
    sample_count: u32,
    config: RendererConfig,
    bvh_root: Option<BVHNode>,
}

impl Renderer for CpuRenderer {
    fn render_frame(&mut self, scene: &Scene) -> RgbaImage {
        self.new_frame(scene);
        let mut frame_buffer = self.frame_buffer(scene);

        let mut rendered_frame =
            ImageBuffer::new(scene.camera.resolution_x(), scene.camera.resolution_y());

        let bvh_root = self.bvh_root.take();

        while self.sample_count < self.config.max_sample_count {
            self.render_next_sample(scene, bvh_root.as_ref(), &mut frame_buffer);
        }
        self.print_frame_buffer(&frame_buffer, &mut rendered_frame);

        self.frame_buffer = Some(frame_buffer);
        self.bvh_root = bvh_root;

        rendered_frame
    }

    fn new_frame(&mut self, scene: &Scene) {
        self.profiler.frame_timer.start();
        self.profiler.prepare_timer.start();
        self.frame_buffer = Some(self.blank_frame_buffer(scene));
        self.sample_count = 0;
        self.bvh_root = BVHNode::new(&scene.objects);
    }

    fn render_sample(&mut self, scene: &Scene) -> Option<RgbaImage> {
        if self.sample_count >= self.config.max_sample_count {
            return None;
        }

        let mut frame_buffer = self.frame_buffer(scene);

        let mut rendered_frame =
            ImageBuffer::new(scene.camera.resolution_x(), scene.camera.resolution_y());

        let bvh_root = self.bvh_root.take();

        self.render_next_sample(scene, bvh_root.as_ref(), &mut frame_buffer);
        self.print_frame_buffer(&frame_buffer, &mut rendered_frame);

        self.frame_buffer = Some(frame_buffer);
        self.bvh_root = bvh_root;

        Some(rendered_frame)
    }

    fn max_sample_count(&self) -> u32 {
        self.config.max_sample_count
    }

    fn max_bounces(&self) -> u32 {
        self.config.max_bounces
    }

    fn profiler(&self) -> &Profiler {
        &self.profiler
    }

    fn sample_count(&self) -> u32 {
        self.sample_count
    }

    fn set_max_sample_count(&mut self, count: u32) {
        self.config.max_sample_count = count;
    }

    fn set_max_bounces(&mut self, bounces: u32) {
        self.config.max_bounces = bounces;
    }
}

impl CpuRenderer {
    pub fn new(config: RendererConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    fn render_next_sample(
        &mut self,
        scene: &Scene,
        bvh_root: Option<&BVHNode>,
        frame_buffer: &mut Rgba32FImage,
    ) {
        self.profiler.prepare_timer.end_if_not_ended();
        self.profiler.render_timer.start_if_not_started();
        self.profiler.sample_timer.start();

        for (x, y, pixel) in frame_buffer.enumerate_pixels_mut() {
            let uv_coord = Vector2::new(
                x as f32 / scene.camera.resolution_x() as f32,
                1.0 - y as f32 / scene.camera.resolution_y() as f32,
            );
            let color = self.per_pixel(uv_coord, scene, bvh_root);
            *pixel = Rgba([
                (pixel[0] + color.x),
                (pixel[1] + color.y),
                (pixel[2] + color.z),
                (pixel[3] + color.w),
            ]);
        }

        self.sample_count += 1;
        if self.sample_count == self.config.max_sample_count {
            self.profiler.render_timer.end();
            self.profiler.frame_timer.end();
        }

        self.profiler.sample_timer.end();
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

    /// Performs Monte Carlo path tracing for a single pixel by solving the rendering equation.
    fn per_pixel(
        &self,
        uv_coord: Vector2<f32>,
        scene: &Scene,
        bvh_root: Option<&BVHNode>,
    ) -> Vector4<f32> {
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
        let mut attenuation = Vector3::new(1.0, 1.0, 1.0);

        for _ in 0..self.config.max_bounces {
            if let Some(hit_record) = self.trace_ray(&ray, bvh_root, scene) {
                // The roughness is squared to achieve perceptual linearity.
                // (based on https://www.pbr-book.org/3ed-2018/Reflection_Models/Microfacet_Models.html
                //           https://www.pbr-book.org/4ed/Reflection_Models/Roughness_Using_Microfacet_Theory
                //           and research by Disney)
                let roughness =
                    hit_record.object.material.roughness * hit_record.object.material.roughness;
                let metallic = hit_record.object.material.metallic;
                let transmission = hit_record.object.material.transmission;

                let mut diffuse_direction =
                    hit_record.world_normal + utils::random_in_unit_sphere();
                if diffuse_direction.dot(hit_record.world_normal) < 0.0 {
                    diffuse_direction = -diffuse_direction;
                }

                let perfect_reflection = ray.direction.reflect(hit_record.world_normal);

                // We perturb the reflection direction to achieve a more realistic reflection.
                // TODO: use a GGX (Trowbridge-Reitz) microfacet distribution.
                let random_offset = utils::random_in_unit_sphere() * roughness;
                let specular_direction = (perfect_reflection + random_offset).normalize();

                let transmission_ray = rand::random::<f32>() < transmission;
                let direction = if transmission_ray {
                    let mut ior = hit_record.object.material.ior;
                    if hit_record.is_front_face {
                        ior = 1.0 / ior;
                    }

                    let ray_direction = ray.direction.normalize();

                    // Apply Schlick's approximation for the Fresnel effect.
                    let cos_theta = ray_direction.dot(-hit_record.world_normal).min(1.0);
                    let reflection_coefficient = {
                        let r0 = ((ior - 1.0) / (ior + 1.0)).powi(2);
                        r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5)
                    };

                    if reflection_coefficient < rand::random::<f32>()
                        && ray_direction.can_refract(hit_record.world_normal, ior)
                    {
                        let refracted = ray_direction.refract(hit_record.world_normal, ior);
                        // Add roughness perturbation to refracted direction
                        let random_offset = utils::random_in_unit_sphere() * roughness;
                        (refracted + random_offset).normalize()
                    } else {
                        specular_direction
                    }
                } else if rand::random::<f32>() < metallic {
                    specular_direction
                } else {
                    if rand::random::<f32>() < roughness {
                        diffuse_direction
                    } else {
                        specular_direction
                    }
                };

                // Move the ray origin slightly along the direction of travel to avoid self-intersections
                let offset = if transmission_ray {
                    direction
                } else {
                    hit_record.world_normal
                };
                ray = Ray {
                    origin: hit_record.world_position + offset * 0.0001,
                    direction,
                };
                if ray.direction.magnitude2() < 1e-10 {
                    ray.direction = hit_record.world_normal;
                }

                attenuation = attenuation.mul_element_wise(hit_record.object.material.albedo);

                light += hit_record.object.material.emission_color
                    * hit_record.object.material.emission_strength;
            } else {
                // Add environment light contribution
                light += scene.world.sample(ray).mul_element_wise(attenuation);
                break;
            };
        }

        light.extend(1.0)
    }

    fn trace_ray<'a>(
        &self,
        ray: &Ray,
        bvh_root: Option<&'a BVHNode>,
        scene: &'a Scene,
    ) -> Option<HitRecord<'a>> {
        bvh_root
            .and_then(|bvh_root| ray.hit(bvh_root))
            .and_then(|(t, o)| self.closest_hit(ray, t, o))
            .or_else(|| self.miss(ray, scene))
    }

    fn closest_hit<'a>(
        &self,
        ray: &Ray,
        hit_distance: f32,
        object: &'a Object,
    ) -> Option<HitRecord<'a>> {
        let world_position = ray.at(hit_distance);
        let mut world_normal = match &object.geometry {
            Geometry::Sphere(sphere) => (world_position - sphere.center).normalize(),
            Geometry::Cube(cube) => {
                let local_position = world_position - cube.center;
                let half_side = cube.side_length / 2.0;

                // Find which face was hit by comparing the hit position with the bounds
                // and checking which one is closest to the bounds
                let x_dist = (local_position.x.abs() - half_side).abs();
                let y_dist = (local_position.y.abs() - half_side).abs();
                let z_dist = (local_position.z.abs() - half_side).abs();

                if x_dist < y_dist && x_dist < z_dist {
                    Vector3::new(local_position.x.signum(), 0.0, 0.0)
                } else if y_dist < z_dist {
                    Vector3::new(0.0, local_position.y.signum(), 0.0)
                } else {
                    Vector3::new(0.0, 0.0, local_position.z.signum())
                }
            }
        };
        let is_front_face = world_normal.dot(ray.direction) <= 0.0;
        if !is_front_face {
            world_normal = -world_normal;
        };

        Some(HitRecord {
            hit_distance,
            is_front_face,
            world_position,
            world_normal,
            object,
        })
    }

    fn miss<'a>(&self, _ray: &Ray, _scene: &Scene) -> Option<HitRecord<'a>> {
        None
    }

    fn frame_buffer(&mut self, scene: &Scene) -> Rgba32FImage {
        self.frame_buffer
            .take()
            .map(|frame_buffer| {
                if frame_buffer.width() != scene.camera.resolution_x()
                    || frame_buffer.height() != scene.camera.resolution_y()
                {
                    return Rgba32FImage::new(
                        scene.camera.resolution_x(),
                        scene.camera.resolution_y(),
                    );
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
