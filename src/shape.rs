pub trait Shape: Send + Sync {
    fn distance_to(&self, p: [f64; 2]) -> f64;
    fn render_plot(&mut self, plot_ui: &mut egui_plot::PlotUi, ctx: &eframe::egui::Context);
    fn render_sidebar_controls(&mut self, ui: &mut eframe::egui::Ui);
    fn save_to(&self, path: &str) -> std::io::Result<()>;
    fn load_from(path: &str) -> Result<Self, String> where Self: Sized;
    fn default_shape() -> Self where Self: Sized;
}