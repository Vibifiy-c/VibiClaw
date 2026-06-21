use eframe::egui;
use egui::*;
use super::{VibiApp, theme::Theme};

pub fn draw(ui: &mut egui::Ui, app: &mut VibiApp, t: &Theme) {
    let available_height = ui.available_height();
    let input_area_height = 120.0;

    ScrollArea::vertical()
        .max_height((available_height - input_area_height).max(100.0))
        .show(ui, |ui| {
            if app.messages.is_empty() {
                draw_welcome(ui, t);
            } else {
                ui.add_space(28.0);
                let msgs = app.messages.clone();
                for (is_user, text) in msgs {
                    draw_message_row(ui, t, is_user, &text);
                }
                ui.add_space(28.0);
            }
        });

    ui.add_space(16.0);
    draw_input_area(ui, app, t);
}

fn draw_welcome(ui: &mut egui::Ui, t: &Theme) {
    ui.vertical_centered(|ui| {
        ui.add_space((ui.available_height() / 2.0 - 100.0).max(20.0));

        let (rect, _) = ui.allocate_exact_size(vec2(56.0, 56.0), Sense::hover());
        ui.painter().rect_filled(rect, Rounding::same(14.0), t.accent());

        ui.add_space(20.0);
        ui.label(RichText::new("Welcome to Vibi AI").size(22.0).color(t.text()));
        ui.add_space(8.0);
        ui.label(
            RichText::new("Start a conversation or explore the available features.")
                .size(14.0)
                .color(t.text_secondary()),
        );
    });
}

fn draw_message_row(ui: &mut egui::Ui, t: &Theme, is_user: bool, text: &str) {
    ui.horizontal(|ui| {
        ui.add_space(24.0);
        if is_user {
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                avatar(ui, t, true);
                ui.add_space(8.0);
                bubble(ui, t, true, text);
            });
        } else {
            avatar(ui, t, false);
            ui.add_space(8.0);
            bubble(ui, t, false, text);
        }
    });
    ui.add_space(6.0);
}

fn avatar(ui: &mut egui::Ui, t: &Theme, is_user: bool) {
    let (rect, _) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::hover());
    ui.painter().rect_filled(rect, Rounding::same(7.0), t.surface2());
    ui.painter().rect_stroke(rect, Rounding::same(7.0), Stroke::new(1.0, t.border()));
    let label = if is_user { "U" } else { "V" };
    let color = if is_user { t.text_secondary() } else { t.accent() };
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, label, FontId::proportional(11.0), color);
}

fn bubble(ui: &mut egui::Ui, t: &Theme, is_user: bool, text: &str) {
    let max_width = ui.available_width() * 0.68;
    let bg = if is_user { t.accent() } else { Color32::TRANSPARENT };
    let text_color = if is_user { Color32::WHITE } else { t.text() };

    Frame::none()
        .fill(bg)
        .rounding(12.0)
        .inner_margin(Margin::symmetric(14.0, 10.0))
        .show(ui, |ui| {
            ui.set_max_width(max_width);
            ui.label(RichText::new(text).size(14.0).color(text_color));
        });
}

fn draw_input_area(ui: &mut egui::Ui, app: &mut VibiApp, t: &Theme) {
    ui.vertical_centered(|ui| {
        let box_width = ui.available_width().min(700.0);

        Frame::none()
            .fill(t.surface())
            .stroke(Stroke::new(1.0, t.border()))
            .rounding(24.0)
            .inner_margin(Margin::symmetric(16.0, 10.0))
            .show(ui, |ui| {
                ui.set_width(box_width);
                ui.horizontal(|ui| {
                    let attach = Button::new(RichText::new("+").size(18.0).color(t.text_muted()))
                        .frame(false)
                        .min_size(vec2(34.0, 34.0));
                    ui.add(attach);

                    let text_edit = TextEdit::multiline(&mut app.input_text)
                        .desired_rows(1)
                        .frame(false)
                        .hint_text("Type your message...")
                        .text_color(t.text());
                    ui.add_sized(vec2(box_width - 100.0, 34.0), text_edit);

                    let send = Button::new(RichText::new("➤").size(15.0).color(Color32::WHITE))
                        .fill(t.accent())
                        .rounding(18.0)
                        .min_size(vec2(36.0, 36.0));
                    if ui.add(send).clicked() && !app.input_text.trim().is_empty() {
                        let msg = app.input_text.trim().to_string();
                        app.messages.push((true, msg));
                        app.input_text.clear();
                    }
                });
            });

        ui.add_space(10.0);
        ui.label(RichText::new("Press Shift+Enter for new line").size(11.0).color(t.text_muted()));
        ui.add_space(20.0);
    });
}