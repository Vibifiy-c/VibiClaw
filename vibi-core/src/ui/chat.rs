use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Entry, Label, Orientation, Align};

pub fn build_chat_view() -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);

    let message_area = GtkBox::new(Orientation::Vertical, 0);
    message_area.set_vexpand(true);

    let welcome = GtkBox::new(Orientation::Vertical, 16);
    welcome.set_valign(Align::Center);
    welcome.set_halign(Align::Center);

    let icon = GtkBox::new(Orientation::Horizontal, 0);
    icon.add_css_class("welcome-icon");
    icon.set_size_request(56, 56);
    welcome.append(&icon);

    let title = Label::new(Some("Welcome to Vibi AI"));
    title.add_css_class("welcome-title");
    welcome.append(&title);

    let subtitle = Label::new(Some("Start a conversation or explore the available features."));
    subtitle.add_css_class("welcome-subtitle");
    welcome.append(&subtitle);

    message_area.append(&welcome);
    root.append(&message_area);

    let input_area = GtkBox::new(Orientation::Vertical, 8);
    input_area.set_margin_start(24);
    input_area.set_margin_end(24);
    input_area.set_margin_bottom(24);
    input_area.set_margin_top(16);

    let input_box = GtkBox::new(Orientation::Horizontal, 8);
    input_box.add_css_class("input-box");

    let attach_btn = Button::with_label("+");
    attach_btn.add_css_class("attach-btn");
    input_box.append(&attach_btn);

    let entry = Entry::new();
    entry.set_placeholder_text(Some("Type your message..."));
    entry.add_css_class("chat-entry");
    entry.set_hexpand(true);
    input_box.append(&entry);

    let send_btn = Button::with_label("➤");
    send_btn.add_css_class("send-btn");
    input_box.append(&send_btn);

    input_area.append(&input_box);

    let hint = Label::new(Some("Press Shift+Enter for new line"));
    hint.add_css_class("input-hint");
    hint.set_halign(Align::Center);
    input_area.append(&hint);

    root.append(&input_area);

    root
}