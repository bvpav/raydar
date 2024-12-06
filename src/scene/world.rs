use cgmath::{InnerSpace, Vector3, VectorSpace};

use crate::renderer::Ray;

pub enum World {
    SkyColor {
        top_color: Vector3<f32>,
        bottom_color: Vector3<f32>,
    },
    SolidColor(Vector3<f32>),
    Transparent,
}

impl World {
    pub fn sample(&self, ray: Ray) -> Vector3<f32> {
        match self {
            World::SkyColor {
                top_color,
                bottom_color,
            } => {
                let up = Vector3::unit_y();
                let cosine_similarity =
                    ray.direction.dot(up) / (ray.direction.magnitude() * up.magnitude());

                let sky_color = bottom_color.lerp(*top_color, (cosine_similarity + 1.0) * 0.5);

                sky_color
            }
            World::SolidColor(color) => *color,
            World::Transparent => todo!("Transparent world support"),
        }
    }
}
