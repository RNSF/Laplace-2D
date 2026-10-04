use crate::shape::Shape;
use eframe::egui;
use egui_plot::{Line, PlotPoints, Points};
use nalgebra::Point2;
use rand::{Rng, RngExt};
use std::fs;

pub struct ImplicitCurveShape {
    pub center: Point2<f64>,
    pub radius_x: f64,
    pub radius_y: f64,
    pub dragging: bool,
}

impl Shape for ImplicitCurveShape {
    fn distance_to(&self, p: Point2<f64>) -> f64 {
        // For an ellipse, we can approximate the signed distance or boundary distance.
        // Using the gradient approximation method from your original implementation:
        let d = p - self.center;
        let rx = self.radius_x.max(0.01);
        let ry = self.radius_y.max(0.01);

        let val = (d.x * d.x) / (rx * rx) + (d.y * d.y) / (ry * ry) - 1.0;
        let grad_x = 2.0 * d.x / (rx * rx);
        let grad_y = 2.0 * d.y / (ry * ry);
        let grad_norm = (grad_x * grad_x + grad_y * grad_y).sqrt();

        if grad_norm == 0.0 {
            val.abs()
        } else {
            val.abs() / grad_norm
        }
    }

    fn closest_point(&self, p: Point2<f64>) -> Point2<f64> {
        // Project to normalized space (unit circle), find closest, then map back
        let d = p - self.center;
        let rx = self.radius_x.max(0.01);
        let ry = self.radius_y.max(0.01);

        // If inside or at the center, project out to the boundary or handle gracefully
        if self.is_point_inside(p) {
            // Approximation for inside closest point to boundary, or just use the gradient descent / radial projection
            let norm_dist = (d.x * d.x) / (rx * rx) + (d.y * d.y) / (ry * ry);
            if norm_dist == 0.0 {
                return self.center + nalgebra::Vector2::new(rx, 0.0);
            }
            // Radial projection to boundary
            let scale = 1.0 / norm_dist.sqrt();
            return self.center + nalgebra::Vector2::new(d.x * scale, d.y * scale);
        }

        // For points outside, project onto the ellipse boundary using normalized coordinates
        let nx = d.x / rx;
        let ny = d.y / ry;
        let len = (nx * nx + ny * ny).sqrt();
        if len == 0.0 {
            return self.center + nalgebra::Vector2::new(rx, 0.0);
        }

        let unit_x = nx / len;
        let unit_y = ny / len;

        self.center + nalgebra::Vector2::new(unit_x * rx, unit_y * ry)
    }

    fn is_point_inside(&self, p: Point2<f64>) -> bool {
        let d = p - self.center;
        let rx = self.radius_x.max(0.01);
        let ry = self.radius_y.max(0.01);

        ((d.x * d.x) / (rx * rx)) + ((d.y * d.y) / (ry * ry)) <= 1.0
    }

    fn random_point_inside<R: Rng + ?Sized>(&self, rng: &mut R) -> Point2<f64> {
        // Uniform sampling inside an ellipse using polar coordinates with radius scaling
        let r = rng.random::<f64>().sqrt(); // sqrt ensures uniform area distribution
        let theta = rng.random_range(0.0..std::f64::consts::TAU);

        let x = self.center.x + r * self.radius_x * theta.cos();
        let y = self.center.y + r * self.radius_y * theta.sin();

        Point2::new(x, y)
    }

    fn render_plot(&mut self, plot_ui: &mut egui_plot::PlotUi, ctx: &egui::Context) {
        let n_points = 100;
        let mut points = Vec::with_capacity(n_points + 1);
        for i in 0..=n_points {
            let theta = (i as f64 / n_points as f64) * std::f64::consts::TAU;
            let x = self.center.x + self.radius_x * theta.cos();
            let y = self.center.y + self.radius_y * theta.sin();
            points.push([x, y]);
        }

        plot_ui.line(
            Line::new(PlotPoints::from(points))
                .color(egui::Color32::LIGHT_GREEN)
                .width(2.5),
        );

        plot_ui.points(
            Points::new(PlotPoints::from(vec![[self.center.x, self.center.y]]))
                .radius(6.0)
                .color(egui::Color32::YELLOW),
        );

        let primary_down = ctx.input(|i| i.pointer.primary_down());
        let primary_pressed = ctx.input(|i| i.pointer.primary_pressed());

        if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
            let p = Point2::new(pointer_pos.x, pointer_pos.y);

            if primary_pressed {
                let dist = (p - self.center).norm();
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
            self.center.x, self.center.y, self.radius_x, self.radius_y
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
                center: Point2::new(cx, cy),
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
            center: Point2::origin(),
            radius_x: 2.5,
            radius_y: 1.5,
            dragging: false,
        }
    }
}
