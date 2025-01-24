use raydar::{
    renderer::{cpu::CpuRenderer, Renderer},
    scene::Scene,
    widgets::{Inspector, Viewport},
};

struct EditorApp {
    scene: Scene,
    renderer: CpuRenderer,
    needs_rerender: bool,
    should_constantly_rerender: bool,
    rendered_scene_handle: Option<egui::TextureHandle>,
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        Inspector::new(
            &mut self.scene,
            &self.renderer,
            &mut self.needs_rerender,
            &mut self.should_constantly_rerender,
        )
        .show(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            Viewport::new(
                &mut self.scene,
                &self.rendered_scene_handle,
                &mut self.needs_rerender,
            )
            .show(ctx, ui);
        });

        self.rerender(ctx);
    }
}

impl EditorApp {
    fn rerender(&mut self, ctx: &eframe::egui::Context) {
        if self.needs_rerender || self.should_constantly_rerender {
            self.renderer.new_frame(&self.scene);
            self.needs_rerender = false;
        }

        let image = self.renderer.render_sample(&self.scene);
        if let Some(image) = image {
            let size = [image.width() as _, image.height() as _];
            let pixels = image.into_raw();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);

            self.rendered_scene_handle = Some(ctx.load_texture(
                "rendered_scene",
                color_image,
                egui::TextureOptions::default(),
            ));

            ctx.request_repaint();
        }
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();

    let scene = Scene::default();

    eframe::run_native(
        "Raydar Editor",
        native_options,
        Box::new(|_cc| {
            Ok(Box::new(EditorApp {
                scene,
                renderer: CpuRenderer::default(),
                needs_rerender: true,
                should_constantly_rerender: false,
                rendered_scene_handle: None,
            }))
        }),
    )
}
