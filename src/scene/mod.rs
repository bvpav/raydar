use camera::Camera;
use objects::Sphere;

pub mod camera;
pub mod objects;

pub struct Scene {
    pub sphere: Sphere,
    pub camera: Camera,
}
