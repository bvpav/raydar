use cgmath::{Vector2, Vector3};
use egui::{Layout, Sense};
use raydar::{
    renderer::Renderer,
    scene::{world::World, Scene},
};

struct EditorApp {
    scene: Scene,
    renderer: Renderer,
    needs_rerender: bool,
    should_constantly_rerender: bool,
    rendered_scene_handle: Option<egui::TextureHandle>,
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("inspector")
            .resizable(true)
            .show(ctx, |ui| {
                ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
                    ui.heading("Inspector");
                    if ui.button("Re-Render").clicked() {
                        self.needs_rerender = true;
                    }
                    if ui
                        .checkbox(&mut self.should_constantly_rerender, "Constantly Re-Render")
                        .changed()
                    {
                        self.needs_rerender = true;
                    }
                    ui.label(format!(
                        "Resolution: {}x{}",
                        self.scene.camera.resolution_x(),
                        self.scene.camera.resolution_y()
                    ));
                    if let Some(last_frame_duration) = self.renderer.last_frame_duration {
                        ui.label(format!(
                            "Last frame took {}ms",
                            last_frame_duration.as_millis()
                        ));
                    } else {
                        ui.label("Frame is rendering...");
                    }
                    if let Some(last_sample_duration) = self.renderer.last_sample_duration {
                        ui.label(format!(
                            "Sample {}/{} took {}ms",
                            self.renderer.sample_count,
                            self.renderer.max_sample_count(),
                            last_sample_duration.as_millis()
                        ));
                    }
                    ui.collapsing("World", |ui| {
                        egui::Grid::new("world_grid")
                            .num_columns(2)
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Type");
                                ui.with_layout(
                                    Layout::top_down_justified(egui::Align::Min),
                                    |ui| {
                                        ui.set_min_width(ui.available_width());
                                        egui::ComboBox::from_id_salt("world_type")
                                            .selected_text(match &self.scene.world {
                                                World::SkyColor { .. } => "Sky",
                                                World::SolidColor(_) => "Solid",
                                                World::Transparent => "Transparent",
                                            })
                                            .width(ui.available_width() - 10.0)
                                            .show_ui(ui, |ui| {
                                                let mut changed = false;
                                                changed |= ui
                                                    .selectable_value(
                                                        &mut self.scene.world,
                                                        World::SkyColor {
                                                            top_color: Vector3::new(
                                                                0.53, 0.8, 0.92,
                                                            ),
                                                            bottom_color: Vector3::new(
                                                                1.0, 1.0, 1.0,
                                                            ),
                                                        },
                                                        "Sky",
                                                    )
                                                    .clicked();
                                                changed |= ui
                                                    .selectable_value(
                                                        &mut self.scene.world,
                                                        World::SolidColor(Vector3::new(
                                                            0.5, 0.5, 0.5,
                                                        )),
                                                        "Solid",
                                                    )
                                                    .clicked();
                                                // changed |= ui
                                                //     .selectable_value(
                                                //         &mut self.scene.world,
                                                //         World::Transparent,
                                                //         "Transparent",
                                                //     )
                                                //     .clicked();
                                                if changed {
                                                    self.needs_rerender = true;
                                                }
                                            });
                                    },
                                );
                                ui.end_row();

                                match &mut self.scene.world {
                                    World::SkyColor {
                                        top_color,
                                        bottom_color,
                                    } => {
                                        ui.label("Top");
                                        ui.with_layout(
                                            Layout::top_down_justified(egui::Align::Min),
                                            |ui| {
                                                if ui
                                                    .color_edit_button_rgb(top_color.as_mut())
                                                    .changed()
                                                {
                                                    self.needs_rerender = true;
                                                }
                                            },
                                        );
                                        ui.end_row();

                                        ui.label("Bottom");
                                        ui.with_layout(
                                            Layout::top_down_justified(egui::Align::Min),
                                            |ui| {
                                                if ui
                                                    .color_edit_button_rgb(bottom_color.as_mut())
                                                    .changed()
                                                {
                                                    self.needs_rerender = true;
                                                }
                                            },
                                        );
                                        ui.end_row();
                                    }
                                    World::SolidColor(color) => {
                                        ui.label("Color");
                                        ui.with_layout(
                                            Layout::top_down_justified(egui::Align::Min),
                                            |ui| {
                                                if ui
                                                    .color_edit_button_rgb(color.as_mut())
                                                    .changed()
                                                {
                                                    self.needs_rerender = true;
                                                }
                                            },
                                        );
                                        ui.end_row();
                                    }
                                    World::Transparent => (),
                                }
                            });
                    });
                    for (idx, sphere) in self.scene.spheres.iter_mut().enumerate() {
                        ui.collapsing(format!("Sphere {}", idx), |ui| {
                            egui::Grid::new("location")
                                .num_columns(2)
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.label("X Location");
                                    ui.with_layout(
                                        Layout::top_down_justified(egui::Align::Min),
                                        |ui| {
                                            if ui
                                                .add(
                                                    egui::DragValue::new(&mut sphere.center.x)
                                                        .speed(0.1),
                                                )
                                                .changed()
                                            {
                                                self.needs_rerender = true;
                                            }
                                        },
                                    );
                                    ui.end_row();

                                    ui.label("Y Location");
                                    ui.with_layout(
                                        Layout::top_down_justified(egui::Align::Min),
                                        |ui| {
                                            if ui
                                                .add(
                                                    egui::DragValue::new(&mut sphere.center.y)
                                                        .speed(0.1),
                                                )
                                                .changed()
                                            {
                                                self.needs_rerender = true;
                                            }
                                        },
                                    );
                                    ui.end_row();

                                    ui.label("Z Location");
                                    ui.with_layout(
                                        Layout::top_down_justified(egui::Align::Min),
                                        |ui| {
                                            if ui
                                                .add(
                                                    egui::DragValue::new(&mut sphere.center.z)
                                                        .speed(0.1),
                                                )
                                                .changed()
                                            {
                                                self.needs_rerender = true;
                                            }
                                        },
                                    );
                                    ui.end_row();

                                    ui.label("Radius");
                                    ui.with_layout(
                                        Layout::top_down_justified(egui::Align::Min),
                                        |ui| {
                                            if ui
                                                .add(
                                                    egui::DragValue::new(&mut sphere.radius)
                                                        .speed(0.1),
                                                )
                                                .changed()
                                            {
                                                self.needs_rerender = true;
                                            }
                                        },
                                    );
                                    ui.end_row();

                                    ui.label("Albedo");
                                    ui.with_layout(
                                        Layout::top_down_justified(egui::Align::Min),
                                        |ui| {
                                            if ui
                                                .color_edit_button_rgb(
                                                    sphere.material.albedo.as_mut(),
                                                )
                                                .changed()
                                            {
                                                self.needs_rerender = true;
                                            }
                                        },
                                    );
                                    ui.end_row();

                                    ui.label("Roughness");
                                    ui.with_layout(
                                        Layout::top_down_justified(egui::Align::Min),
                                        |ui| {
                                            if ui
                                                .add(
                                                    egui::DragValue::new(
                                                        &mut sphere.material.roughness,
                                                    )
                                                    .speed(0.1)
                                                    .range(0.0..=1.0),
                                                )
                                                .changed()
                                            {
                                                self.needs_rerender = true;
                                            }
                                        },
                                    );
                                    ui.end_row();

                                    ui.label("Transmission");
                                    ui.with_layout(
                                        Layout::top_down_justified(egui::Align::Min),
                                        |ui| {
                                            if ui
                                                .add(
                                                    egui::DragValue::new(
                                                        &mut sphere.material.transmission,
                                                    )
                                                    .speed(0.1)
                                                    .range(0.0..=1.0),
                                                )
                                                .changed()
                                            {
                                                self.needs_rerender = true;
                                            }
                                        },
                                    );
                                    ui.end_row();
                                    ui.label("Metallic");
                                    ui.with_layout(
                                        Layout::top_down_justified(egui::Align::Min),
                                        |ui| {
                                            if ui
                                                .add(
                                                    egui::DragValue::new(
                                                        &mut sphere.material.metallic,
                                                    )
                                                    .speed(0.1)
                                                    .range(0.0..=1.0),
                                                )
                                                .changed()
                                            {
                                                self.needs_rerender = true;
                                            }
                                        },
                                    );
                                    ui.end_row();

                                    ui.label("Emission Color");
                                    ui.with_layout(
                                        Layout::top_down_justified(egui::Align::Min),
                                        |ui| {
                                            if ui
                                                .color_edit_button_rgb(
                                                    sphere.material.emission_color.as_mut(),
                                                )
                                                .changed()
                                            {
                                                self.needs_rerender = true;
                                            }
                                        },
                                    );
                                    ui.end_row();

                                    ui.label("Emission Strength");
                                    ui.with_layout(
                                        Layout::top_down_justified(egui::Align::Min),
                                        |ui| {
                                            if ui
                                                .add(
                                                    egui::DragValue::new(
                                                        &mut sphere.material.emission_strength,
                                                    )
                                                    .speed(0.1)
                                                    .range(0.0..=f32::INFINITY),
                                                )
                                                .changed()
                                            {
                                                self.needs_rerender = true;
                                            }
                                        },
                                    );
                                });
                        });
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();
            let available_size_x = available_size.x.round() as u32;
            let available_size_y = available_size.y.round() as u32;
            if available_size_x != self.scene.camera.resolution_x()
                || available_size_y != self.scene.camera.resolution_y()
            {
                self.scene.camera.set_resolution_x(available_size_x);
                self.scene.camera.set_resolution_y(available_size_y);
                self.needs_rerender = true;
            }

            if let Some(texture) = &self.rendered_scene_handle {
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
                    self.needs_rerender = true;
                }
                let scroll_delta = ctx.input(|i| i.smooth_scroll_delta);
                if viewport.hovered() && scroll_delta.y != 0.0 {
                    camera.zoom(-scroll_delta.y * (1.0 / 255.0));
                    self.needs_rerender = true;
                }
            }
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
                renderer: Renderer::default(),
                needs_rerender: true,
                should_constantly_rerender: false,
                rendered_scene_handle: None,
            }))
        }),
    )
}
