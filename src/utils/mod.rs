use std::ops::{Mul, Sub};

use cgmath::{InnerSpace, Vector3};
use rand::Rng;

#[allow(dead_code)]
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

pub fn random_in_unit_sphere() -> Vector3<f32> {
    let mut rng = rand::thread_rng();
    Vector3::new(
        rng.gen_range(-1.0..=1.0),
        rng.gen_range(-1.0..=1.0),
        rng.gen_range(-1.0..=1.0),
    )
    .normalize()
}

pub fn random_in_unit_hemisphere(normal: Vector3<f32>) -> Vector3<f32> {
    let random = random_in_unit_sphere();
    if random.dot(normal) > 0.0 {
        random
    } else {
        -random
    }
}
