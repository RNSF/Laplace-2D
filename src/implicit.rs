use crate::shape::Shape;
use eframe::egui;
use egui_plot::{Line, PlotPoints, Points};
use std::fs;

pub struct ImplicitCurveShape {
    pub center: [f64; 2],
    pub radius_x: f64,
    pub radius_y: f64,
    pub dragging: bool,
}

impl Shape for ImplicitCurveShape {
    fn distance_to(&self, p: [f64; 2]) -> f64 {
        let dx = p[0] - self.center[0];
        let dy = p[1] - self.center[1];
        let rx = self.radius_x.max(0.01);
        let ry = self.radius_y.max(0.01);

        let val = (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry) - 1.0;
        let grad_x = 2.0 * dx / (rx * rx);
        let grad_y = 2.0 * dy / (ry * ry);
        let grad_norm = (grad_x * grad_x + grad_y * grad_y).sqrt();

        if grad_norm == 0.0 {
            val.abs()
        } else {
            val.abs() / grad_norm
        }
    }

    fn render_plot(&mut self, plot_ui: &mut egui_plot::PlotUi, ctx: &egui::Context) {
        let n_points = 100;
        let mut points = Vec::with_capacity(n_points + 1);
        for i in 0..=n_points {
            let theta = (i as f64 / n_points as f64) * std::f64::consts::TAU;
            let x = self.center[0] + self.radius_x * theta.cos();
            let y = self.center[1] + self.radius_y * theta.sin();
            points.push([x, y]);
        }

        plot_ui.line(
            Line::new(PlotPoints::from(points))
                .color(egui::Color32::LIGHT_GREEN)
                .width(2.5),
        );

        plot_ui.points(
            Points::new(PlotPoints::from(vec![self.center]))
                .radius(6.0)
                .color(egui::Color32::YELLOW),
        );

        let primary_down = ctx.input(|i| i.pointer.primary_down());
        let primary_pressed = ctx.input(|i| i.pointer.primary_pressed());

        if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
            let p = [pointer_pos.x, pointer_pos.y];

            if primary_pressed {
                let dx = p[0] - self.center[0];
                let dy = p[1] - self.center[1];
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < 0.8 {
                    self.dragging = true;
                }
            }

            if primary_down && self.dragging {
                self.center = p;
            } else if !primary_down {
                self.dragging = false;
            }
        } else if !primary_down {
            self.dragging = false;
        }
    }

    fn render_sidebar_controls(&mut self, ui: &mut egui::Ui) {
        ui.label("🖱️ **Implicit Curve Interactions:**");
        ui.label("• Left-click & drag center point: Move curve");
        ui.add_space(8.0);
        ui.add(egui::Slider::new(&mut self.radius_x, 0.5..=5.0).text("Radius X"));
        ui.add(egui::Slider::new(&mut self.radius_y, 0.5..=5.0).text("Radius Y"));
    }

    fn save_to(&self, path: &str) -> std::io::Result<()> {
        let data = format!(
            "{},{},{},{}\n",
            self.center[0], self.center[1], self.radius_x, self.radius_y
        );
        fs::write(path, data)
    }

    fn load_from(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let line = content.lines().next().ok_or("Empty file")?;
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 4 {
            let cx = parts[0].parse().map_err(|_| "Invalid center x")?;
            let cy = parts[1].parse().map_err(|_| "Invalid center y")?;
            let rx = parts[2].parse().map_err(|_| "Invalid radius x")?;
            let ry = parts[3].parse().map_err(|_| "Invalid radius y")?;
            Ok(Self {
                center: [cx, cy],
                radius_x: rx,
                radius_y: ry,
                dragging: false,
            })
        } else {
            Err("Implicit curve file format invalid (expected cx,cy,rx,ry)".to_string())
        }
    }

    fn default_shape() -> Self {
        Self {
            center: [0.0, 0.0],
            radius_x: 2.5,
            radius_y: 1.5,
            dragging: false,
        }
    }
}
