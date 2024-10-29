use objects::Sphere;

pub mod camera;
pub mod objects;

pub struct Scene {
    pub resolution_x: u32,
    pub resolution_y: u32,

    pub sphere: Sphere,
}
