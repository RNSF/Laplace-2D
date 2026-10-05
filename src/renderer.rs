use crate::shape::{BoundaryType, Shape};
use eframe::egui::{Color32, ColorImage};
use nalgebra::{Point2, Rotation2, Vector2};
use rand::RngExt;

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

                let dist = shape.distance_to(Point2::new(x, y));
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

pub struct WalkOnSphereRenderer {
    pub max_expected_dist: f64,
}

impl WalkOnSphereRenderer {
    pub fn new(max_expected_dist: f64) -> Self {
        Self { max_expected_dist }
    }
}

impl ShapeRenderer for WalkOnSphereRenderer {
    fn render_image(
        &self,
        shape: &dyn Shape,
        width: usize,
        height: usize,
        min_val: f64,
        max_val: f64,
    ) -> ColorImage {
        let mut pixels = vec![Color32::RED; width * height];
        let mut rng = rand::rng();
        let epsilon = 0.1;

        for y_idx in 0..height {
            let v = y_idx as f64 / (height as f64 - 1.0);
            let y = max_val - v * (max_val - min_val);

            for x_idx in 0..width {
                let u = x_idx as f64 / (width as f64 - 1.0);
                let x = min_val + u * (max_val - min_val);

                let point = Point2::new(x, y);

                if (!shape.is_point_inside(point)) {
                    continue;
                }

                let mut value = 0.0;

                let iter_count = 1000;

                for i in 0..iter_count {
                    let mut bouncing_point = point;

                    loop {
                        let radius = shape.distance_to(bouncing_point);
                        if (radius < epsilon) {
                            break;
                        }
                        let theta = rng.random_range(0.0..std::f64::consts::TAU);
                        bouncing_point += Rotation2::new(theta) * Vector2::new(radius, 0.0);
                    }

                    let boundary_condition = shape.boundary_condition_at(bouncing_point);

                    match boundary_condition.bc_type {
                        BoundaryType::First => {
                            value += boundary_condition.value;
                        }
                        BoundaryType::Second => {}
                    }
                }

                let avg = value / iter_count as f64;

                let t = (avg + 10.0) / 20.0;

                let c = (t * 255.0) as u8;

                pixels[y_idx * width + x_idx] = Color32::from_rgb(c, c, c);
            }
        }
        ColorImage {
            size: [width, height],
            pixels,
        }
    }
}
