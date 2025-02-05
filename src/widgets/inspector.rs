use cgmath::Vector3;
use egui::{Grid, Layout};

use crate::{
    renderer::Renderer,
    scene::{
        material::Material,
        objects::{Cube, Geometry, Object, Sphere},
        world::World,
        Scene,
    },
};

pub struct Inspector<'a> {
    scene: &'a mut Scene,
    renderer: &'a dyn Renderer,
    needs_rerender: &'a mut bool,
    should_constantly_rerender: &'a mut bool,
}

impl<'a> Inspector<'a> {
    pub fn new(
        scene: &'a mut Scene,
        renderer: &'a dyn Renderer,
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

                    if let Some(last_frame_duration) =
                        self.renderer.profiler().frame_timer().duration()
                    {
                        ui.label(format!(
                            "Last frame took {}ms",
                            last_frame_duration.as_millis()
                        ));
                    } else {
                        ui.label("Frame is rendering...");
                    }

                    if let Some(last_sample_duration) =
                        self.renderer.profiler().sample_timer().duration()
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

pub struct MaterialEditor<'a> {
    material: &'a mut Material,
    needs_rerender: &'a mut bool,
}

impl<'a> MaterialEditor<'a> {
    pub fn new(material: &'a mut Material, needs_rerender: &'a mut bool) -> Self {
        Self {
            material,
            needs_rerender,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.label("Albedo");
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            if ui
                .color_edit_button_rgb(self.material.albedo.as_mut())
                .changed()
            {
                *self.needs_rerender = true;
            }
        });
        ui.end_row();

        ui.label("Roughness");
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            if ui
                .add(
                    egui::DragValue::new(&mut self.material.roughness)
                        .speed(0.1)
                        .range(0.0..=1.0),
                )
                .changed()
            {
                *self.needs_rerender = true;
            }
        });
        ui.end_row();

        ui.label("Transmission");
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            if ui
                .add(
                    egui::DragValue::new(&mut self.material.transmission)
                        .speed(0.1)
                        .range(0.0..=1.0),
                )
                .changed()
            {
                *self.needs_rerender = true;
            }
        });
        ui.end_row();

        ui.label("IOR");
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            if ui
                .add(egui::DragValue::new(&mut self.material.ior).speed(0.1))
                .changed()
            {
                *self.needs_rerender = true;
            }
        });
        ui.end_row();

        ui.label("Metallic");
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            if ui
                .add(
                    egui::DragValue::new(&mut self.material.metallic)
                        .speed(0.1)
                        .range(0.0..=1.0),
                )
                .changed()
            {
                *self.needs_rerender = true;
            }
        });
        ui.end_row();

        ui.label("Emission Color");
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            if ui
                .color_edit_button_rgb(self.material.emission_color.as_mut())
                .changed()
            {
                *self.needs_rerender = true;
            }
        });
        ui.end_row();

        ui.label("Emission Strength");
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            if ui
                .add(
                    egui::DragValue::new(&mut self.material.emission_strength)
                        .speed(0.1)
                        .range(0.0..=f32::INFINITY),
                )
                .changed()
            {
                *self.needs_rerender = true;
            }
        });
        ui.end_row();
    }
}

pub struct ObjectEditor<'a> {
    object: &'a mut Object,
    index: usize,
    needs_rerender: &'a mut bool,
}

impl<'a> ObjectEditor<'a> {
    pub fn new(object: &'a mut Object, index: usize, needs_rerender: &'a mut bool) -> Self {
        Self {
            object,
            index,
            needs_rerender,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let object_string = match &self.object.geometry {
            Geometry::Sphere(_) => "Sphere",
            Geometry::Cube(_) => "Cube",
        };
        ui.collapsing(format!("{} {}", object_string, self.index), |ui| {
            Grid::new(format!("object_{}_grid", self.index))
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    match &mut self.object.geometry {
                        Geometry::Sphere(sphere) => {
                            SphereEditor::new(sphere, self.needs_rerender).show(ui)
                        }
                        Geometry::Cube(cube) => CubeEditor::new(cube, self.needs_rerender).show(ui),
                    }

                    MaterialEditor::new(&mut self.object.material, self.needs_rerender).show(ui);
                });
        });
    }
}

pub struct SphereEditor<'a> {
    sphere: &'a mut Sphere,
    needs_rerender: &'a mut bool,
}

impl<'a> SphereEditor<'a> {
    pub fn new(sphere: &'a mut Sphere, needs_rerender: &'a mut bool) -> Self {
        Self {
            sphere,
            needs_rerender,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.label("X Location");
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            if ui
                .add(egui::DragValue::new(&mut self.sphere.center.x).speed(0.1))
                .changed()
            {
                *self.needs_rerender = true;
            }
        });
        ui.end_row();

        ui.label("Y Location");
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            if ui
                .add(egui::DragValue::new(&mut self.sphere.center.y).speed(0.1))
                .changed()
            {
                *self.needs_rerender = true;
            }
        });
        ui.end_row();

        ui.label("Z Location");
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            if ui
                .add(egui::DragValue::new(&mut self.sphere.center.z).speed(0.1))
                .changed()
            {
                *self.needs_rerender = true;
            }
        });
        ui.end_row();

        ui.label("Radius");
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            if ui
                .add(egui::DragValue::new(&mut self.sphere.radius).speed(0.1))
                .changed()
            {
                *self.needs_rerender = true;
            }
        });
        ui.end_row();
    }
}

pub struct CubeEditor<'a> {
    cube: &'a mut Cube,
    needs_rerender: &'a mut bool,
}

impl<'a> CubeEditor<'a> {
    pub fn new(cube: &'a mut Cube, needs_rerender: &'a mut bool) -> Self {
        Self {
            cube,
            needs_rerender,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.label("X Location");
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            if ui
                .add(egui::DragValue::new(&mut self.cube.center.x).speed(0.1))
                .changed()
            {
                *self.needs_rerender = true;
            }
        });
        ui.end_row();

        ui.label("Y Location");
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            if ui
                .add(egui::DragValue::new(&mut self.cube.center.y).speed(0.1))
                .changed()
            {
                *self.needs_rerender = true;
            }
        });
        ui.end_row();

        ui.label("Z Location");
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            if ui
                .add(egui::DragValue::new(&mut self.cube.center.z).speed(0.1))
                .changed()
            {
                *self.needs_rerender = true;
            }
        });
        ui.end_row();

        ui.label("Side Length");
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            if ui
                .add(egui::DragValue::new(&mut self.cube.side_length).speed(0.1))
                .changed()
            {
                *self.needs_rerender = true;
            }
        });
        ui.end_row();
    }
}

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
