use cgmath::Point3;
use egui::Layout;
use raydar::*;

struct EditorApp {
    scene: Scene,
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("image will be here...");
        });

        egui::SidePanel::right("inspector")
            .resizable(true)
            .show(ctx, |ui| {
                ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
                    ui.heading("Inspector");
                    ui.collapsing("Sphere", |ui| {
                        egui::Grid::new("location")
                            .num_columns(2)
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("X Location");
                                ui.with_layout(
                                    Layout::top_down_justified(egui::Align::Min),
                                    |ui| {
                                        ui.add(
                                            egui::DragValue::new(&mut self.scene.sphere.center.x)
                                                .speed(0.1),
                                        );
                                    },
                                );
                                ui.end_row();

                                ui.label("Y Location");
                                ui.with_layout(
                                    Layout::top_down_justified(egui::Align::Min),
                                    |ui| {
                                        ui.add(
                                            egui::DragValue::new(&mut self.scene.sphere.center.y)
                                                .speed(0.1),
                                        );
                                    },
                                );
                                ui.end_row();

                                ui.label("Z Location");
                                ui.with_layout(
                                    Layout::top_down_justified(egui::Align::Min),
                                    |ui| {
                                        ui.add(
                                            egui::DragValue::new(&mut self.scene.sphere.center.z)
                                                .speed(0.1),
                                        );
                                    },
                                );
                                ui.end_row();

                                ui.label("Radius");
                                ui.with_layout(
                                    Layout::top_down_justified(egui::Align::Min),
                                    |ui| {
                                        ui.add(
                                            egui::DragValue::new(&mut self.scene.sphere.radius)
                                                .speed(0.1),
                                        );
                                    },
                                );
                            });
                    });
                });
            });
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();

    let scene = Scene {
        resolution_x: 854,
        resolution_y: 480,
        sphere: Sphere {
            center: Point3::new(0.0, 0.0, 0.0),
            radius: 0.5,
        },
    };

    eframe::run_native(
        "Raydar Editor",
        native_options,
        Box::new(|_cc| Ok(Box::new(EditorApp { scene }))),
    )
}
