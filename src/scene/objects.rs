use cgmath::{Point3, Vector3};

pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
    pub albedo: Vector3<f32>,
}
