mod app;
mod implicit;
mod polygon;
mod renderer;
mod shape;

use app::ShapeApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([950.0, 750.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Modular Shape Distance Field App",
        options,
        Box::new(|_cc| Ok(Box::new(ShapeApp::default()))),
    )
}
