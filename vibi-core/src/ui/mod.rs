pub mod theme;
pub mod sidebar;
pub mod chat;
pub mod agentic;
pub mod dialogs;
pub mod settings;

use eframe::egui;
use egui::*;
use theme::Theme;

#[derive(PartialEq, Clone, Copy)]
pub enum Tab {
    Chat,
    Agentic,
    Notebook,
}

pub struct VibiApp {
    pub theme: Theme,
    pub sidebar_collapsed: bool,
    pub active_tab: Tab,
    pub input_text: String,
    pub dark_mode: bool,
    pub messages: Vec<(bool, String)>,
}

impl VibiApp {
    pub fn new() -> Self {
        let system_dark = matches!(dark_light::detect(), dark_light::Mode::Dark);
        Self {
            theme: Theme::new(system_dark),
            sidebar_collapsed: false,
            active_tab: Tab::Chat,
            input_text: String::new(),
            dark_mode: system_dark,
            messages: Vec::new(),
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let t = &self.theme;
        let mut visuals = if self.dark_mode { egui::Visuals::dark() } else { egui::Visuals::light() };
        visuals.window_fill = t.bg();
        visuals.panel_fill = t.bg();
        visuals.override_text_color = Some(t.text());
        ctx.set_visuals(visuals);
    }
}

impl eframe::App for VibiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme = Theme::new(self.dark_mode);
        self.apply_theme(ctx);
        let t = self.theme.clone();

        let sidebar_width = if self.sidebar_collapsed { 70.0 } else { 260.0 };

        egui::SidePanel::left("sidebar")
            .exact_width(sidebar_width)
            .resizable(false)
            .frame(egui::Frame::none().fill(t.sidebar_bg()))
            .show(ctx, |ui| {
                sidebar::draw(ui, self, &t);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(t.bg()))
            .show(ctx, |ui| {
                match self.active_tab {
                    Tab::Chat => chat::draw(ui, self, &t),
                    Tab::Agentic => agentic::draw(ui, self, &t),
                    Tab::Notebook => { ui.label("notebook tab — todo"); },
                };
            });

        if let Some(pos) = ctx.pointer_latest_pos() {
            ctx.set_cursor_icon(egui::CursorIcon::None);
            let painter = ctx.layer_painter(LayerId::new(
                Order::Tooltip,
                Id::new("cursor_layer"),
            ));
            theme::draw_cursor(&painter, pos, t.accent());
        }
    }
}