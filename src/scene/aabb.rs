use cgmath::Point3;

pub trait HasBoundingBox {
    fn aabb(&self) -> AABB;
}

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}
