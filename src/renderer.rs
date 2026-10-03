use crate::shape::Shape;
use eframe::egui::{Color32, ColorImage};

pub trait ShapeRenderer: Send + Sync {
    fn render_image(
        &self,
        shape: &dyn Shape,
        width: usize,
        height: usize,
        min_val: f64,
        max_val: f64,
    ) -> ColorImage;
}

pub struct DistanceFieldRenderer {
    pub max_expected_dist: f64,
}

impl DistanceFieldRenderer {
    pub fn new(max_expected_dist: f64) -> Self {
        Self { max_expected_dist }
    }
}

impl ShapeRenderer for DistanceFieldRenderer {
    fn render_image(
        &self,
        shape: &dyn Shape,
        width: usize,
        height: usize,
        min_val: f64,
        max_val: f64,
    ) -> ColorImage {
        let mut pixels = vec![Color32::BLACK; width * height];
        for y_idx in 0..height {
            let v = y_idx as f64 / (height as f64 - 1.0);
            let y = max_val - v * (max_val - min_val);

            for x_idx in 0..width {
                let u = x_idx as f64 / (width as f64 - 1.0);
                let x = min_val + u * (max_val - min_val);

                let dist = shape.distance_to([x, y]);
                let t = (dist / self.max_expected_dist).clamp(0.0, 1.0);

                let r = ((1.0 - t).powi(2) * 255.0) as u8;
                let g = ((1.0 - t) * 200.0) as u8;
                let b = (255.0 * (0.3 + 0.7 * t)) as u8;

                pixels[y_idx * width + x_idx] = Color32::from_rgb(r, g, b);
            }
        }
        ColorImage {
            size: [width, height],
            pixels,
        }
    }
}

pub struct SolidBackgroundRenderer {
    pub color: Color32,
}

impl SolidBackgroundRenderer {
    pub fn new(color: Color32) -> Self {
        Self { color }
    }
}

impl ShapeRenderer for SolidBackgroundRenderer {
    fn render_image(
        &self,
        _shape: &dyn Shape,
        width: usize,
        height: usize,
        _min_val: f64,
        _max_val: f64,
    ) -> ColorImage {
        let pixels = vec![self.color; width * height];
        ColorImage {
            size: [width, height],
            pixels,
        }
    }
}
