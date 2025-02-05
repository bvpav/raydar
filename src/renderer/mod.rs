use image::RgbaImage;
use timing::Profiler;

use crate::scene::Scene;

pub mod cpu;
pub mod vulkan;

pub mod timing;

pub trait Renderer {
    fn render_frame(&mut self, scene: &Scene) -> RgbaImage;
    fn render_sample(&mut self, scene: &Scene) -> Option<RgbaImage>;
    fn new_frame(&mut self, scene: &Scene);
    fn max_sample_count(&self) -> usize;
    fn profiler(&self) -> &Profiler;
    fn sample_count(&self) -> usize;
}

pub(crate) const MAX_SAMPLE_COUNT: usize = 1024;
pub(crate) const MAX_BOUNCES: usize = 12;
