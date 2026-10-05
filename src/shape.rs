use nalgebra::Point2;
use rand::rngs::ThreadRng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryType {
    First,  // Dirichlet (e.g., fixed value)
    Second, // Second type / Neumann (e.g., derivative/flux)
}

#[derive(Debug, Clone, Copy)]
pub struct BoundaryCondition {
    pub bc_type: BoundaryType,
    pub value: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct BoundaryConditionResult {
    pub bc_type: BoundaryType,
    pub value: f64,
    pub closest_point: Point2<f64>,
}

pub trait Shape: Send + Sync {
    /// Calculates the minimum distance from the point to the shape.
    fn distance_to(&self, p: Point2<f64>) -> f64;

    /// Returns the actual closest point on or inside the shape to the given point.
    fn closest_point(&self, p: Point2<f64>) -> Point2<f64>;

    /// Checks whether the given point is inside the shape.
    fn is_point_inside(&self, p: Point2<f64>) -> bool;

    /// Generates a random point inside the shape using the provided RNG.
    fn random_point_inside(&self, rng: &mut ThreadRng) -> Point2<f64>;

    /// Renders the shape onto the egui plot UI.
    fn render_plot(&mut self, plot_ui: &mut egui_plot::PlotUi, ctx: &eframe::egui::Context);

    /// Renders sidebar controls for modifying the shape parameters.
    fn render_sidebar_controls(&mut self, ui: &mut eframe::egui::Ui);

    /// Serializes and saves the shape to a file.
    fn save_to(&self, path: &str) -> std::io::Result<()>;

    /// Loads a shape from a file.
    fn load_from(path: &str) -> Result<Self, String>
    where
        Self: Sized;

    /// Returns a default instance of the shape.
    fn default_shape() -> Self
    where
        Self: Sized;

    /// Returns the boundary condition and closest point for an arbitrary query point.
    fn boundary_condition_at(&self, p: Point2<f64>) -> BoundaryConditionResult;
}