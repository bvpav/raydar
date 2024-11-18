use cgmath::Vector3;

pub struct Material {
    pub albedo: Vector3<f32>,
    pub roughness: f32,
    pub metallic: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Vector3::new(0.0, 0.0, 0.0),
            roughness: 0.5,
            metallic: 0.0,
        }
    }
}

impl Material {
    pub fn with_albedo(albedo: Vector3<f32>) -> Self {
        Self {
            albedo,
            ..Default::default()
        }
    }
}
