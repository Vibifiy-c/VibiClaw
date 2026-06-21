use eframe::egui;
use super::{VibiApp, theme::Theme};

pub fn draw(ui: &mut egui::Ui, app: &mut VibiApp, t: &Theme) {
    ui.label("agentic");
}