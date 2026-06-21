mod types;
mod analyzer;
mod sandbox;
mod executor;
mod commands;
mod ui;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Vibi AI")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Vibi AI",
        options,
        Box::new(|_cc| Box::new(ui::VibiApp::new())),
    )
}