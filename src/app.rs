use crate::implicit::ImplicitCurveShape;
use crate::polygon::PolygonShape;
use crate::renderer::{
    DistanceFieldRenderer, ShapeRenderer, SolidBackgroundRenderer, WalkOnSphereRenderer,
};
use crate::shape::Shape;
use eframe::egui::{self, Color32, TextureOptions};
use egui_plot::{Plot, PlotImage, PlotPoint};
use std::fs;
use std::path::Path;

pub enum ActiveShapeVariant {
    Polygon(PolygonShape),
    ImplicitCurve(ImplicitCurveShape),
}

pub struct ShapeApp {
    pub shape_type_index: usize,
    pub renderer_index: usize,
    pub shape: ActiveShapeVariant,
    pub status_message: String,
    pub current_filename: String,
    pub new_file_name: String,
    pub available_files: Vec<String>,
    // Caching and manual render trigger fields
    texture: Option<egui::TextureHandle>,
    last_renderer_index: usize,
    last_shape_type_index: usize,
    render_requested: bool,
}

impl ShapeApp {
    fn active_shape_trait(&self) -> &dyn Shape {
        match &self.shape {
            ActiveShapeVariant::Polygon(s) => s,
            ActiveShapeVariant::ImplicitCurve(s) => s,
        }
    }

    fn shape_dir(&self) -> &'static str {
        match self.shape_type_index {
            0 => "shapes/polygons",
            _ => "shapes/implicit_curves",
        }
    }

    fn default_filename_for_type(idx: usize) -> &'static str {
        match idx {
            0 => "default.txt",
            _ => "default_curve.txt",
        }
    }

    fn refresh_file_list(&mut self) {
        self.available_files.clear();
        let dir = self.shape_dir();
        let _ = fs::create_dir_all(dir);
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".txt") {
                        self.available_files.push(name.to_string());
                    }
                }
            }
        }
        self.available_files.sort();
    }

    fn switch_shape_type(&mut self, new_idx: usize) {
        self.shape_type_index = new_idx;
        let dir = self.shape_dir();
        let _ = fs::create_dir_all(dir);
        let def_file = Self::default_filename_for_type(new_idx).to_string();
        let path = format!("{}/{}", dir, def_file);

        match new_idx {
            0 => {
                let poly = if Path::new(&path).exists() {
                    PolygonShape::load_from(&path).unwrap_or_else(|_| PolygonShape::default_shape())
                } else {
                    let p = PolygonShape::default_shape();
                    let _ = p.save_to(&path);
                    p
                };
                self.shape = ActiveShapeVariant::Polygon(poly);
            }
            _ => {
                let curve = if Path::new(&path).exists() {
                    ImplicitCurveShape::load_from(&path)
                        .unwrap_or_else(|_| ImplicitCurveShape::default_shape())
                } else {
                    let c = ImplicitCurveShape::default_shape();
                    let _ = c.save_to(&path);
                    c
                };
                self.shape = ActiveShapeVariant::ImplicitCurve(curve);
            }
        }

        self.current_filename = def_file;
        self.new_file_name = "my_shape.txt".to_string();
        self.refresh_file_list();
        self.status_message = "Switched shape type".to_string();
        self.render_requested = true; // Request render on switch
    }

    fn render_plot(&mut self, plot_ui: &mut egui_plot::PlotUi, ctx: &egui::Context) {
        match &mut self.shape {
            ActiveShapeVariant::Polygon(s) => s.render_plot(plot_ui, ctx),
            ActiveShapeVariant::ImplicitCurve(s) => s.render_plot(plot_ui, ctx),
        }
    }

    fn render_sidebar_controls(&mut self, ui: &mut egui::Ui) {
        match &mut self.shape {
            ActiveShapeVariant::Polygon(s) => s.render_sidebar_controls(ui),
            ActiveShapeVariant::ImplicitCurve(s) => s.render_sidebar_controls(ui),
        }
    }

    fn save_current(&self, path: &str) -> std::io::Result<()> {
        match &self.shape {
            ActiveShapeVariant::Polygon(s) => s.save_to(path),
            ActiveShapeVariant::ImplicitCurve(s) => s.save_to(path),
        }
    }

    fn load_into(&mut self, path: &str) -> Result<(), String> {
        match self.shape_type_index {
            0 => {
                let poly = PolygonShape::load_from(path)?;
                self.shape = ActiveShapeVariant::Polygon(poly);
                self.render_requested = true;
                Ok(())
            }
            _ => {
                let curve = ImplicitCurveShape::load_from(path)?;
                self.shape = ActiveShapeVariant::ImplicitCurve(curve);
                self.render_requested = true;
                Ok(())
            }
        }
    }

    fn reset_default(&mut self) {
        match self.shape_type_index {
            0 => self.shape = ActiveShapeVariant::Polygon(PolygonShape::default_shape()),
            _ => {
                self.shape = ActiveShapeVariant::ImplicitCurve(ImplicitCurveShape::default_shape())
            }
        }
        self.render_requested = true;
    }
}

impl Default for ShapeApp {
    fn default() -> Self {
        let shape_idx = 0;
        let dir = "shapes/polygons";
        let _ = fs::create_dir_all(dir);
        let default_filename = "default.txt".to_string();
        let default_path = format!("{}/{}", dir, default_filename);

        let polygon = if Path::new(&default_path).exists() {
            PolygonShape::load_from(&default_path).unwrap_or_else(|_| PolygonShape::default_shape())
        } else {
            let p = PolygonShape::default_shape();
            let _ = p.save_to(&default_path);
            p
        };

        let mut app = Self {
            shape_type_index: shape_idx,
            renderer_index: 0,
            shape: ActiveShapeVariant::Polygon(polygon),
            status_message: "Ready".to_string(),
            current_filename: default_filename,
            new_file_name: "my_shape.txt".to_string(),
            available_files: Vec::new(),
            texture: None,
            last_renderer_index: 0,
            last_shape_type_index: 0,
            render_requested: true, // Render initially on startup
        };
        app.refresh_file_list();
        app
    }
}

impl eframe::App for ShapeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- 1. Sidebar Panel ---
        egui::SidePanel::left("controls_panel").show(ctx, |ui| {
            ui.heading("Distance Field Studio");
            ui.separator();

            // Explicit Render Button prominently at the top
            if ui.button("▶ Render / Update View").clicked() {
                self.render_requested = true;
                self.status_message = "Rendering updated view...".to_string();
            }

            ui.add_space(8.0);
            ui.label("📐 **Shape Type:**");
            let mut selected_type = self.shape_type_index;
            egui::ComboBox::from_id_salt("shape_type_combo")
                .selected_text(match selected_type {
                    0 => "Polygon",
                    _ => "Implicit Curve (Ellipse)",
                })
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(&mut selected_type, 0, "Polygon")
                        .clicked()
                    {
                        self.switch_shape_type(0);
                    }
                    if ui
                        .selectable_value(&mut selected_type, 1, "Implicit Curve (Ellipse)")
                        .clicked()
                    {
                        self.switch_shape_type(1);
                    }
                });

            ui.add_space(8.0);
            ui.label("🎨 **Background Renderer:**");
            let mut selected_renderer = self.renderer_index;
            egui::ComboBox::from_id_salt("renderer_combo")
                .selected_text(match selected_renderer {
                    0 => "Distance Field",
                    1 => "Solid Background (None)",
                    _ => "Walk on Spheres",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected_renderer, 0, "Distance Field");
                    ui.selectable_value(&mut selected_renderer, 1, "Solid Background (None)");
                    ui.selectable_value(&mut selected_renderer, 2, "Walk on Spheres");
                });
            if selected_renderer != self.renderer_index {
                self.renderer_index = selected_renderer;
                self.status_message = "Switched renderer (press Render to apply)".to_string();
            }

            ui.add_space(8.0);
            ui.separator();

            self.render_sidebar_controls(ui);

            ui.add_space(12.0);
            ui.separator();
            ui.heading("File Management");
            ui.add_space(4.0);

            ui.label(format!("📁 Active: {}", self.current_filename));

            if ui.button("💾 Save Current Shape").clicked() {
                let dir = self.shape_dir();
                let path = format!("{}/{}", dir, self.current_filename);
                match self.save_current(&path) {
                    Ok(_) => self.status_message = format!("Saved {}", self.current_filename),
                    Err(e) => self.status_message = format!("Save error: {}", e),
                }
            }

            ui.add_space(8.0);
            ui.strong("Create New File:");
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.new_file_name).desired_width(120.0));
                if ui.button("Create").clicked() {
                    let mut filename = self.new_file_name.trim().to_string();
                    if !filename.ends_with(".txt") {
                        filename.push_str(".txt");
                    }
                    if !filename.is_empty() {
                        let dir = self.shape_dir();
                        let _ = fs::create_dir_all(dir);
                        let path = format!("{}/{}", dir, filename);

                        let save_result = match self.shape_type_index {
                            0 => PolygonShape::default_shape().save_to(&path),
                            _ => ImplicitCurveShape::default_shape().save_to(&path),
                        };

                        match save_result {
                            Ok(_) => {
                                let _ = self.load_into(&path);
                                self.current_filename = filename;
                                self.refresh_file_list();
                                self.status_message =
                                    format!("Created & loaded {}", self.current_filename);
                            }
                            Err(e) => self.status_message = format!("Error creating file: {}", e),
                        }
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.heading("Saved Files (Click to Switch)");

            self.refresh_file_list();
            egui::ScrollArea::vertical()
                .max_height(140.0)
                .show(ui, |ui| {
                    for filename in self.available_files.clone() {
                        let is_active = filename == self.current_filename;
                        let label_text = if is_active {
                            format!("▶ {}", filename)
                        } else {
                            filename.clone()
                        };

                        if ui.selectable_label(is_active, label_text).clicked() {
                            let dir = self.shape_dir();
                            let path = format!("{}/{}", dir, filename);
                            match self.load_into(&path) {
                                Ok(_) => {
                                    self.current_filename = filename;
                                    self.status_message =
                                        format!("Loaded {}", self.current_filename);
                                }
                                Err(e) => self.status_message = format!("Load error: {}", e),
                            }
                        }
                    }
                });

            ui.add_space(8.0);
            ui.colored_label(egui::Color32::LIGHT_BLUE, &self.status_message);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(8.0);
                if ui.button("🔄 Reset Current Shape to Default").clicked() {
                    self.reset_default();
                    self.status_message = "Reset shape to default".to_string();
                }
            });
        });

        // --- 2. Central Panel & Manual Rendering Logic ---
        // --- 2. Central Panel & Manual Rendering Logic ---
        egui::CentralPanel::default().show(ctx, |ui| {
            let width = 200;
            let height = 200;
            let min_val = -5.0;
            let max_val = 5.0;

            // Only re-render when the button has been explicitly clicked
            if self.render_requested || self.texture.is_none() {
                let renderer: Box<dyn ShapeRenderer> = match self.renderer_index {
                    0 => Box::new(DistanceFieldRenderer::new(7.0)),
                    1 => Box::new(SolidBackgroundRenderer::new(Color32::from_rgb(25, 25, 30))),
                    _ => Box::new(WalkOnSphereRenderer::new(1000.0)),
                };

                let color_image =
                    renderer.render_image(self.active_shape_trait(), width, height, min_val, max_val);

                let texture = ctx.load_texture(
                    "interactive_shape_dist",
                    color_image,
                    TextureOptions::LINEAR,
                );

                self.texture = Some(texture);
                self.last_renderer_index = self.renderer_index;
                self.last_shape_type_index = self.shape_type_index;
                self.render_requested = false; // Reset flag after rendering completes
            }

            // Clone the texture handle so it doesn't hold an immutable borrow on `self.texture`
            // while `self.render_plot` requires a mutable borrow (`&mut self`).
            if let Some(texture) = self.texture.clone() {
                Plot::new("interactive_distance_plot")
                    .view_aspect(1.0)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .allow_zoom(true)
                    .show(ui, |plot_ui| {
                        let center = PlotPoint::new(0.0, 0.0);
                        let size =
                            egui::Vec2::new((max_val - min_val) as f32, (max_val - min_val) as f32);
                        plot_ui.image(PlotImage::new(&texture, center, size));

                        self.render_plot(plot_ui, ctx);
                    });
            }
        });
    }
}
