use cgmath::Vector3;

pub struct Material {
    pub albedo: Vector3<f32>,
    pub roughness: f32,
    pub metallic: f32,
    pub emission_color: Vector3<f32>,
    pub emission_strength: f32,
    pub transmission: f32,
    pub ior: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Vector3::new(0.5, 0.5, 0.5),
            roughness: 0.5,
            metallic: 0.0,
            emission_color: Vector3::new(0.0, 0.0, 0.0),
            emission_strength: 0.0,
            transmission: 0.0,
            ior: 1.5,
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

    pub fn with_emission(emission_color: Vector3<f32>, emission_strength: f32) -> Self {
        Self {
            emission_color,
            emission_strength,
            ..Default::default()
        }
    }

    pub fn with_transmission(transmission: f32, ior: f32) -> Self {
        Self {
            transmission,
            ior,
            ..Default::default()
        }
    }
}
