use image::RgbaImage;
use timing::Profiler;

use crate::scene::Scene;

pub mod cpu;
pub mod vulkan;

pub mod timing;

pub struct RendererConfig {
    pub max_sample_count: u32,
    pub max_bounces: u32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            max_sample_count: 1024,
            max_bounces: 12,
        }
    }
}

pub trait Renderer {
    fn render_frame(&mut self, scene: &Scene) -> RgbaImage;
    fn render_sample(&mut self, scene: &Scene) -> Option<RgbaImage>;
    fn new_frame(&mut self, scene: &Scene);
    fn max_sample_count(&self) -> u32;
    fn profiler(&self) -> &Profiler;
    fn sample_count(&self) -> u32;
}
