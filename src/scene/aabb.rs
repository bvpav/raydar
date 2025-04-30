use cgmath::Point3;

pub trait HasBoundingBox {
    fn aabb(&self) -> AABB;
}

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl AABB {
    pub fn union(&self, other: &AABB) -> AABB {
        AABB {
            min: self.min.zip(other.min, |a, b| a.min(b)),
            max: self.max.zip(other.max, |a, b| a.max(b)),
        }
    }
}
