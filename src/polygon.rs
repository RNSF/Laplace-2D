use crate::shape::Shape;
use eframe::egui;
use egui_plot::{Line, PlotPoints, Points};
use nalgebra::Point2;
use rand::Rng;
use rand::RngExt;
use std::fs;

pub struct PolygonShape {
    pub polygon: Vec<[f64; 2]>,
    pub dragging_vertex: Option<usize>,
}

pub fn default_polygon() -> Vec<[f64; 2]> {
    vec![
        [0.0, 3.0],
        [2.5, 0.5],
        [1.5, -2.5],
        [-1.5, -2.5],
        [-2.5, 0.5],
    ]
}

fn point_to_segment_distance(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let ap = [p[0] - a[0], p[1] - a[1]];
    let ab_len_sq = ab[0] * ab[0] + ab[1] * ab[1];

    if ab_len_sq == 0.0 {
        let dx = p[0] - a[0];
        let dy = p[1] - a[1];
        return (dx * dx + dy * dy).sqrt();
    }

    let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / ab_len_sq).clamp(0.0, 1.0);
    let closest = [a[0] + t * ab[0], a[1] + t * ab[1]];

    let dx = p[0] - closest[0];
    let dy = p[1] - closest[1];
    (dx * dx + dy * dy).sqrt()
}

impl Shape for PolygonShape {
    fn distance_to(&self, p: Point2<f64>) -> f64 {
        let p_arr = [p.x, p.y];
        let mut min_dist = f64::MAX;
        let n = self.polygon.len();
        for i in 0..n {
            let a = self.polygon[i];
            let b = self.polygon[(i + 1) % n];
            let dist = point_to_segment_distance(p_arr, a, b);
            if dist < min_dist {
                min_dist = dist;
            }
        }
        min_dist
    }

    fn closest_point(&self, p: Point2<f64>) -> Point2<f64> {
        let p_arr = [p.x, p.y];
        if self.is_point_inside(p) {
            return p;
        }
        let mut min_dist = f64::MAX;
        let mut closest = p_arr;
        let n = self.polygon.len();
        for i in 0..n {
            let a = self.polygon[i];
            let b = self.polygon[(i + 1) % n];
            let ab = [b[0] - a[0], b[1] - a[1]];
            let ap = [p_arr[0] - a[0], p_arr[1] - a[1]];
            let ab_len_sq = ab[0] * ab[0] + ab[1] * ab[1];
            let c = if ab_len_sq == 0.0 {
                a
            } else {
                let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / ab_len_sq).clamp(0.0, 1.0);
                [a[0] + t * ab[0], a[1] + t * ab[1]]
            };
            let dx = p_arr[0] - c[0];
            let dy = p_arr[1] - c[1];
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < min_dist {
                min_dist = dist;
                closest = c;
            }
        }
        Point2::new(closest[0], closest[1])
    }

    fn is_point_inside(&self, p: Point2<f64>) -> bool {
        let mut inside = false;
        let n = self.polygon.len();
        if n < 3 {
            return false;
        }
        let mut j = n - 1;
        for i in 0..n {
            let xi = self.polygon[i][0];
            let yi = self.polygon[i][1];
            let xj = self.polygon[j][0];
            let yj = self.polygon[j][1];

            let intersect = ((yi > p.y) != (yj > p.y))
                && (p.x < (xj - xi) * (p.y - yi) / (yj - yi + f64::EPSILON) + xi);
            if intersect {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    fn random_point_inside<R: Rng + ?Sized>(&self, rng: &mut R) -> Point2<f64>
    where
        Self: Sized,
    {
        if self.polygon.is_empty() {
            return Point2::origin();
        }

        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for v in &self.polygon {
            min_x = min_x.min(v[0]);
            max_x = max_x.max(v[0]);
            min_y = min_y.min(v[1]);
            max_y = max_y.max(v[1]);
        }

        loop {
            let x = rng.random_range(min_x..=max_x);
            let y = rng.random_range(min_y..=max_y);
            let p = Point2::new(x, y);
            if self.is_point_inside(p) {
                return p;
            }
        }
    }

    fn render_plot(&mut self, plot_ui: &mut egui_plot::PlotUi, ctx: &egui::Context) {
        let mut outline_points = self.polygon.clone();
        if let Some(&first) = self.polygon.first() {
            outline_points.push(first);
        }

        plot_ui.line(
            Line::new(PlotPoints::from(outline_points))
                .color(egui::Color32::WHITE)
                .width(2.5),
        );

        plot_ui.points(
            Points::new(PlotPoints::from(self.polygon.clone()))
                .radius(6.0)
                .color(egui::Color32::YELLOW),
        );

        let primary_down = ctx.input(|i| i.pointer.primary_down());
        let primary_pressed = ctx.input(|i| i.pointer.primary_pressed());
        let secondary_pressed = ctx.input(|i| i.pointer.secondary_pressed());

        if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
            let p = [pointer_pos.x, pointer_pos.y];

            if secondary_pressed {
                let mut delete_idx = None;
                let mut min_d = 0.5;
                for (i, v) in self.polygon.iter().enumerate() {
                    let dx = v[0] - p[0];
                    let dy = v[1] - p[1];
                    let d = (dx * dx + dy * dy).sqrt();
                    if d < min_d {
                        min_d = d;
                        delete_idx = Some(i);
                    }
                }
                if let Some(idx) = delete_idx {
                    if self.polygon.len() > 3 {
                        self.polygon.remove(idx);
                        self.dragging_vertex = None;
                    }
                }
            }

            if primary_pressed {
                let mut clicked_vertex = None;
                let mut min_d = 0.5;
                for (i, v) in self.polygon.iter().enumerate() {
                    let dx = v[0] - p[0];
                    let dy = v[1] - p[1];
                    let d = (dx * dx + dy * dy).sqrt();
                    if d < min_d {
                        min_d = d;
                        clicked_vertex = Some(i);
                    }
                }

                if let Some(idx) = clicked_vertex {
                    self.dragging_vertex = Some(idx);
                } else {
                    let mut clicked_segment = None;
                    let mut min_seg_d = 0.35;
                    let mut insert_pos = p;
                    let n = self.polygon.len();
                    for i in 0..n {
                        let a = self.polygon[i];
                        let b = self.polygon[(i + 1) % n];
                        let ab = [b[0] - a[0], b[1] - a[1]];
                        let ap = [p[0] - a[0], p[1] - a[1]];
                        let ab_len_sq = ab[0] * ab[0] + ab[1] * ab[1];
                        let closest = if ab_len_sq == 0.0 {
                            a
                        } else {
                            let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / ab_len_sq).clamp(0.0, 1.0);
                            [a[0] + t * ab[0], a[1] + t * ab[1]]
                        };
                        let dx = p[0] - closest[0];
                        let dy = p[1] - closest[1];
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < min_seg_d {
                            min_seg_d = dist;
                            clicked_segment = Some(i);
                            insert_pos = closest;
                        }
                    }
                    if let Some(seg_idx) = clicked_segment {
                        self.polygon.insert(seg_idx + 1, insert_pos);
                        self.dragging_vertex = Some(seg_idx + 1);
                    }
                }
            }

            if primary_down {
                if let Some(idx) = self.dragging_vertex {
                    if idx < self.polygon.len() {
                        self.polygon[idx] = p;
                    }
                }
            } else {
                self.dragging_vertex = None;
            }
        } else if !primary_down {
            self.dragging_vertex = None;
        }
    }

    fn render_sidebar_controls(&mut self, ui: &mut egui::Ui) {
        ui.label("🖱️ **Polygon Interactions:**");
        ui.label("• Left-click & drag vertex: Move point");
        ui.label("• Left-click line: Add & drag new point");
        ui.label("• Right-click vertex: Delete point");
    }

    fn save_to(&self, path: &str) -> std::io::Result<()> {
        let mut data = String::new();
        for v in &self.polygon {
            data.push_str(&format!("{},{}\n", v[0], v[1]));
        }
        fs::write(path, data)
    }

    fn load_from(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut new_poly = Vec::new();
        for line in content.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(x), Ok(y)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                    new_poly.push([x, y]);
                }
            }
        }
        if new_poly.len() >= 3 {
            Ok(Self {
                polygon: new_poly,
                dragging_vertex: None,
            })
        } else {
            Err("Polygon file must contain at least 3 points".to_string())
        }
    }

    fn default_shape() -> Self {
        Self {
            polygon: default_polygon(),
            dragging_vertex: None,
        }
    }
}