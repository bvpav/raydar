use raydar::{
    renderer::{cpu::CpuRenderer, vulkan::VulkanRenderer, Renderer},
    scene::{benchmark, Scene},
    widgets::{Inspector, Viewport},
};
use std::{fs::File, io::Write};

struct EditorApp {
    scene: Scene,
    renderer: Box<dyn Renderer>,

    original_resolution_x: u32,
    original_resolution_y: u32,

    needs_rerender: bool,
    should_constantly_rerender: bool,
    rendered_scene_handle: Option<egui::TextureHandle>,
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            if let Err(err) = self.save_scene() {
                eprintln!("Failed to save scene: {err}");
            }
        }

        Inspector::new(
            &mut self.scene,
            Box::as_ref(&self.renderer),
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
    fn new(scene: Scene, renderer: Box<dyn Renderer>) -> Self {
        let original_resolution_x = scene.camera.resolution_x();
        let original_resolution_y = scene.camera.resolution_y();

        Self {
            scene,
            renderer,

            original_resolution_x,
            original_resolution_y,

            needs_rerender: true,
            should_constantly_rerender: false,
            rendered_scene_handle: None,
        }
    }

    fn save_scene(&self) -> Result<(), std::io::Error> {
        // Clone the scene to restore the original resolution
        let mut scene = self.scene.clone();
        let camera = &mut scene.camera;
        camera.set_resolution_x(self.original_resolution_x);
        camera.set_resolution_y(self.original_resolution_y);

        let json = serde_json::to_string_pretty(&scene)?;
        let file_name = "scene.rscn";
        let mut file = File::create(file_name)?;
        file.write_all(json.as_bytes())?;
        println!("Scene saved to {file_name}");
        Ok(())
    }

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

    let scene = benchmark::benchmark_scene();

    let cpu_arg = std::env::args().any(|arg| arg == "--cpu");
    let renderer: Box<dyn Renderer> = if cpu_arg {
        Box::new(CpuRenderer::default())
    } else {
        Box::new(VulkanRenderer::default())
    };

    eframe::run_native(
        "Raydar Editor",
        native_options,
        Box::new(|_cc| Ok(Box::new(EditorApp::new(scene, renderer)))),
    )
}
