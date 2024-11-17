use std::ops::{Mul, Sub};

use cgmath::InnerSpace;

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
