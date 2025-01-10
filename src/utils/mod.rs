use std::ops::{Mul, Sub};

use cgmath::{InnerSpace, Vector3};
use rand::Rng;

pub trait Reflect {
    fn reflect(&self, normal: Self) -> Self;
}

impl<V> Reflect for V
where
    V: InnerSpace + Sub<V, Output = V> + Mul<f32, Output = V>,
{
    fn reflect(&self, normal: Self) -> Self {
        *self - normal * self.dot(normal) * 2.0
    }
}

pub trait Refract {
    fn refract(&self, normal: Self, ior_ratio: f32) -> Self;
    fn can_refract(&self, normal: Self, ior_ratio: f32) -> bool;
}

impl Refract for cgmath::Vector3<f32> {
    fn refract(&self, normal: Self, ior_ratio: f32) -> Self {
        assert!((self.magnitude2() - 1.0).abs() < 1e-6);
        assert!((normal.magnitude2() - 1.0).abs() < 1e-6);

        let cos_theta = self.dot(-normal).min(1.0);

        let perpendicular = ior_ratio * (self + cos_theta * normal);
        let parallel = -(1.0 - perpendicular.magnitude2()).abs().sqrt() * normal;

        perpendicular + parallel
    }

    fn can_refract(&self, normal: Self, ior_ratio: f32) -> bool {
        assert!((self.magnitude2() - 1.0).abs() < 1e-6);

        let cos_theta = self.dot(-normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        ior_ratio * sin_theta <= 1.0
    }
}

pub fn random_in_unit_sphere() -> Vector3<f32> {
    let mut rng = rand::thread_rng();
    Vector3::new(
        rng.gen_range(-1.0..=1.0),
        rng.gen_range(-1.0..=1.0),
        rng.gen_range(-1.0..=1.0),
    )
    .normalize()
}

pub fn _random_in_unit_hemisphere(normal: Vector3<f32>) -> Vector3<f32> {
    let random = random_in_unit_sphere();
    if random.dot(normal) > 0.0 {
        random
    } else {
        -random
    }
}
