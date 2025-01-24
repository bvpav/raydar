use image::RgbaImage;

use crate::scene::Scene;

use super::{timing::FrameTimer, Renderer};

pub struct VulkanRenderer {
    timer: FrameTimer,
}

impl Renderer for VulkanRenderer {
    fn render_frame(&mut self, _scene: &Scene) -> RgbaImage {
        todo!()
    }

    fn render_sample(&mut self, _scene: &Scene) -> Option<RgbaImage> {
        todo!()
    }

    fn new_frame(&mut self, _scene: &Scene) {
        todo!()
    }

    fn max_sample_count(&self) -> usize {
        todo!()
    }

    fn timer(&self) -> &FrameTimer {
        &self.timer
    }

    fn sample_count(&self) -> usize {
        0
    }
}

impl VulkanRenderer {
    pub fn new() -> Self {
        // TODO: Initialize Vulkan and do the rest of the setup
        Self {
            timer: FrameTimer::default(),
        }
    }
}
