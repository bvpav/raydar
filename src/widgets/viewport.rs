use crate::scene::Scene;
use cgmath::Vector2;
use egui::Sense;

pub struct Viewport<'a> {
    scene: &'a mut Scene,
    texture_handle: &'a Option<egui::TextureHandle>,
    needs_rerender: &'a mut bool,
}

impl<'a> Viewport<'a> {
    pub fn new(
        scene: &'a mut Scene,
        texture_handle: &'a Option<egui::TextureHandle>,
        needs_rerender: &'a mut bool,
    ) -> Self {
        Self {
            scene,
            texture_handle,
            needs_rerender,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let available_size_x = available_size.x.round() as u32;
        let available_size_y = available_size.y.round() as u32;
        if available_size_x != self.scene.camera.resolution_x()
            || available_size_y != self.scene.camera.resolution_y()
        {
            self.scene.camera.set_resolution_x(available_size_x);
            self.scene.camera.set_resolution_y(available_size_y);
            *self.needs_rerender = true;
        }

        if let Some(texture) = self.texture_handle {
            let viewport = ui.add(egui::Image::new(texture).sense(Sense::drag()));
            let camera = &mut self.scene.camera;
            // if viewport.dragged() && viewport.dragged_by(egui::PointerButton::Middle) {
            if viewport.dragged() {
                let delta = viewport.drag_delta();
                if ctx.input(|i| i.modifiers.ctrl) {
                    let delta = egui::vec2(
                        delta.x / camera.resolution_x() as f32 * 2.0,
                        -delta.y / camera.resolution_y() as f32 * 2.0,
                    );
                    let direction = -delta.y.signum() * 3.0;
                    camera.zoom(delta.length() * direction);
                } else if ctx.input(|i| i.modifiers.shift) {
                    camera.pan(Vector2::new(delta.x, delta.y));
                } else {
                    camera.orbit(Vector2::new(delta.x, delta.y));
                }
                *self.needs_rerender = true;
            }
            let scroll_delta = ctx.input(|i| i.smooth_scroll_delta);
            if viewport.hovered() && scroll_delta.y != 0.0 {
                camera.zoom(-scroll_delta.y * (1.0 / 255.0));
                *self.needs_rerender = true;
            }
        }
    }
}
