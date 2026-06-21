use eframe::egui;
use egui::*;
use super::{VibiApp, Tab, theme::Theme};

pub fn draw(ui: &mut egui::Ui, app: &mut VibiApp, t: &Theme) {
    // ---- header ----
    Frame::none()
        .inner_margin(Margin::symmetric(16.0, 16.0))
        .show(ui, |ui| {
            ui.set_min_height(38.0);
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::hover());
                ui.painter().rect_filled(rect, Rounding::same(8.0), t.accent());

                if !app.sidebar_collapsed {
                    ui.add_space(10.0);
                    ui.label(RichText::new("Vibi ").color(t.text()).size(16.0));
                    ui.label(RichText::new("AI").color(t.accent()).size(16.0));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let toggle = ui.add(
                            Button::new(RichText::new("☰").size(16.0).color(t.text_secondary()))
                                .frame(false),
                        );
                        if toggle.clicked() {
                            app.sidebar_collapsed = !app.sidebar_collapsed;
                        }
                    });
                }
            });
        });
    ui.separator();

    if app.sidebar_collapsed {
        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            let toggle = ui.add(Button::new(RichText::new("☰").color(t.text_secondary())).frame(false));
            if toggle.clicked() {
                app.sidebar_collapsed = false;
            }
        });
        return;
    }

    // ---- new chat button ----
    Frame::none()
        .inner_margin(Margin::symmetric(8.0, 12.0))
        .show(ui, |ui| {
            let btn = Button::new(RichText::new("+  New chat").color(Color32::WHITE).size(13.0))
                .fill(t.accent())
                .min_size(vec2(ui.available_width(), 34.0))
                .rounding(8.0);
            if ui.add(btn).clicked() {
                // TODO: new chat
            }
        });

    // ---- nav menu ----
    Frame::none()
        .inner_margin(Margin::symmetric(8.0, 12.0))
        .show(ui, |ui| {
            nav_item(ui, app, t, Tab::Chat, "Chat");
            nav_item(ui, app, t, Tab::Agentic, "Agentic Tool");
            nav_item_badge(ui, app, t, Tab::Notebook, "AI Notebook", "WEB");
        });
    ui.separator();

    // ---- recent chats ----
    Frame::none()
        .inner_margin(Margin::symmetric(8.0, 12.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new("RECENT CHATS")
                    .color(t.text_muted())
                    .size(10.0),
            );
            ui.add_space(6.0);
            ScrollArea::vertical().show(ui, |_ui| {
                // chat list — empty for now
            });
        });

    let remaining = ui.available_height();
    if remaining > 96.0 {
        ui.add_space(remaining - 96.0);
    }

    // ---- footer ----
    ui.separator();
    Frame::none()
        .inner_margin(Margin::symmetric(8.0, 12.0))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                let label = if app.dark_mode { "☀  Toggle theme" } else { "🌙  Toggle theme" };
                let btn = Button::new(RichText::new(label).color(t.text_secondary()).size(13.0))
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::new(1.0, t.border()))
                    .min_size(vec2(ui.available_width(), 32.0))
                    .rounding(7.0);
                if ui.add(btn).clicked() {
                    app.dark_mode = !app.dark_mode;
                }

                ui.add_space(6.0);

                let btn2 = Button::new(RichText::new("⚙  Settings").color(t.text_secondary()).size(12.0))
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::new(1.0, t.border()))
                    .min_size(vec2(ui.available_width(), 30.0))
                    .rounding(7.0);
                if ui.add(btn2).clicked() {
                    // TODO: settings
                }
            });
        });
}

fn nav_item(ui: &mut egui::Ui, app: &mut VibiApp, t: &Theme, tab: Tab, label: &str) {
    nav_item_inner(ui, app, t, tab, label, None);
}
fn nav_item_badge(ui: &mut egui::Ui, app: &mut VibiApp, t: &Theme, tab: Tab, label: &str, badge: &str) {
    nav_item_inner(ui, app, t, tab, label, Some(badge));
}

fn nav_item_inner(ui: &mut egui::Ui, app: &mut VibiApp, t: &Theme, tab: Tab, label: &str, badge: Option<&str>) {
    let active = app.active_tab == tab;
    let bg = if active { t.accent_light() } else { Color32::TRANSPARENT };
    let text_color = if active { t.accent() } else { t.text_secondary() };

    let resp = Frame::none()
        .fill(bg)
        .rounding(7.0)
        .inner_margin(Margin::symmetric(12.0, 10.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).color(text_color).size(13.0));
                if let Some(b) = badge {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        Frame::none()
                            .fill(t.accent())
                            .rounding(10.0)
                            .inner_margin(Margin::symmetric(6.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(b).color(Color32::WHITE).size(9.0));
                            });
                    });
                }
            });
        });

    let id = Id::new(("nav", label));
    if ui.interact(resp.response.rect, id, Sense::click()).clicked() {
        app.active_tab = tab;
    }
    ui.add_space(2.0);
}