use crate::{
    renderer::{CpuRenderer, Renderer},
    scene::{world::World, Scene},
};
use cgmath::Vector3;
use egui::{Grid, Layout};

use super::scene::ObjectEditor;

pub struct WorldEditor<'a> {
    world: &'a mut World,
    needs_rerender: &'a mut bool,
}

impl<'a> WorldEditor<'a> {
    pub fn new(world: &'a mut World, needs_rerender: &'a mut bool) -> Self {
        Self {
            world,
            needs_rerender,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("World", |ui| {
            Grid::new("world_grid")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Type");
                    ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
                        ui.set_min_width(ui.available_width());
                        egui::ComboBox::from_id_salt("world_type")
                            .selected_text(match &self.world {
                                World::SkyColor { .. } => "Sky",
                                World::SolidColor(_) => "Solid",
                                World::Transparent => "Transparent",
                            })
                            .width(ui.available_width() - 10.0)
                            .show_ui(ui, |ui| {
                                let mut changed = false;
                                changed |= ui
                                    .selectable_value(
                                        self.world,
                                        World::SkyColor {
                                            top_color: Vector3::new(0.53, 0.8, 0.92),
                                            bottom_color: Vector3::new(1.0, 1.0, 1.0),
                                        },
                                        "Sky",
                                    )
                                    .clicked();

                                // changed |= ui
                                //     .selectable_value(
                                //         &mut self.scene.world,
                                //         World::Transparent,
                                //         "Transparent",
                                //     )
                                //     .clicked();
                                changed |= ui
                                    .selectable_value(
                                        self.world,
                                        World::SolidColor(Vector3::new(0.5, 0.5, 0.5)),
                                        "Solid",
                                    )
                                    .clicked();
                                if changed {
                                    *self.needs_rerender = true;
                                }
                            });
                    });
                    ui.end_row();

                    match self.world {
                        World::SkyColor {
                            top_color,
                            bottom_color,
                        } => {
                            ui.label("Top");
                            ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
                                if ui.color_edit_button_rgb(top_color.as_mut()).changed() {
                                    *self.needs_rerender = true;
                                }
                            });
                            ui.end_row();

                            ui.label("Bottom");
                            ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
                                if ui.color_edit_button_rgb(bottom_color.as_mut()).changed() {
                                    *self.needs_rerender = true;
                                }
                            });
                            ui.end_row();
                        }
                        World::SolidColor(color) => {
                            ui.label("Color");
                            ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
                                if ui.color_edit_button_rgb(color.as_mut()).changed() {
                                    *self.needs_rerender = true;
                                }
                            });
                            ui.end_row();
                        }
                        World::Transparent => (),
                    }
                });
        });
    }
}

pub struct Inspector<'a> {
    scene: &'a mut Scene,
    renderer: &'a CpuRenderer,
    needs_rerender: &'a mut bool,
    should_constantly_rerender: &'a mut bool,
}

impl<'a> Inspector<'a> {
    pub fn new(
        scene: &'a mut Scene,
        renderer: &'a CpuRenderer,
        needs_rerender: &'a mut bool,
        should_constantly_rerender: &'a mut bool,
    ) -> Self {
        Self {
            scene,
            renderer,
            needs_rerender,
            should_constantly_rerender,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("inspector")
            .resizable(true)
            .show(ctx, |ui| {
                ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
                    ui.heading("Inspector");
                    if ui.button("Re-Render").clicked() {
                        *self.needs_rerender = true;
                    }
                    if ui
                        .checkbox(self.should_constantly_rerender, "Constantly Re-Render")
                        .changed()
                    {
                        *self.needs_rerender = true;
                    }
                    ui.label(format!(
                        "Resolution: {}x{}",
                        self.scene.camera.resolution_x(),
                        self.scene.camera.resolution_y()
                    ));

                    if let Some(last_frame_duration) = self.renderer.timer().last_frame_duration() {
                        ui.label(format!(
                            "Last frame took {}ms",
                            last_frame_duration.as_millis()
                        ));
                    } else {
                        ui.label("Frame is rendering...");
                    }

                    if let Some(last_sample_duration) = self.renderer.timer().last_sample_duration()
                    {
                        ui.label(format!(
                            "Sample {}/{} took {}ms",
                            self.renderer.sample_count(),
                            self.renderer.max_sample_count(),
                            last_sample_duration.as_millis()
                        ));
                    }

                    WorldEditor::new(&mut self.scene.world, self.needs_rerender).show(ui);

                    for (idx, sphere) in self.scene.objects.iter_mut().enumerate() {
                        ObjectEditor::new(sphere, idx, self.needs_rerender).show(ui);
                    }
                });
            });
    }
}
