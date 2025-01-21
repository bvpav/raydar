use crate::scene::{material::Material, objects::Sphere};
use egui::{Grid, Layout};

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

pub struct SphereEditor<'a> {
    sphere: &'a mut Sphere,
    index: usize,
    needs_rerender: &'a mut bool,
}

impl<'a> SphereEditor<'a> {
    pub fn new(sphere: &'a mut Sphere, index: usize, needs_rerender: &'a mut bool) -> Self {
        Self {
            sphere,
            index,
            needs_rerender,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.collapsing(format!("Sphere {}", self.index), |ui| {
            Grid::new(format!("sphere_{}_grid", self.index))
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
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

                    MaterialEditor::new(&mut self.sphere.material, self.needs_rerender).show(ui);
                });
        });
    }
}
