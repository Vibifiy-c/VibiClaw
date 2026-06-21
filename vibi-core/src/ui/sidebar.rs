use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, Separator, Align};
use std::rc::Rc;
use std::cell::Cell;

pub fn build_sidebar() -> GtkBox {
    let sidebar = GtkBox::new(Orientation::Vertical, 0);
sidebar.add_css_class("sidebar");
sidebar.set_width_request(260);
sidebar.set_hexpand(false);


    let header = GtkBox::new(Orientation::Horizontal, 8);
    header.set_margin_top(16);
    header.set_margin_bottom(16);
    header.set_margin_start(16);
    header.set_margin_end(16);

    let logo_icon = GtkBox::new(Orientation::Horizontal, 0);
    logo_icon.add_css_class("logo-icon");
    logo_icon.set_size_request(32, 32);
    header.append(&logo_icon);

    let logo_vibi = Label::new(Some("Vibi "));
    logo_vibi.add_css_class("logo-text-vibi");
    header.append(&logo_vibi);

    let logo_ai = Label::new(Some("AI"));
    logo_ai.add_css_class("logo-text-ai");
    header.append(&logo_ai);

    let header_spacer = GtkBox::new(Orientation::Horizontal, 0);
    header_spacer.set_hexpand(true);
    header.append(&header_spacer);

    let toggle_btn = Button::with_label("☰");
    toggle_btn.add_css_class("footer-btn");
    header.append(&toggle_btn);

    sidebar.append(&header);
    sidebar.append(&Separator::new(Orientation::Horizontal));

    let new_chat_btn = Button::with_label("+  New chat");
    new_chat_btn.add_css_class("new-chat-btn");
    new_chat_btn.set_margin_top(12);
    new_chat_btn.set_margin_bottom(12);
    new_chat_btn.set_margin_start(8);
    new_chat_btn.set_margin_end(8);
    sidebar.append(&new_chat_btn);

    let nav_box = GtkBox::new(Orientation::Vertical, 2);
    nav_box.set_margin_start(8);
    nav_box.set_margin_end(8);

    let chat_btn = nav_item_button("Chat", None);
    chat_btn.add_css_class("active");
    nav_box.append(&chat_btn);

    let agentic_btn = nav_item_button("Agentic Tool", None);
    nav_box.append(&agentic_btn);

    let notebook_btn = nav_item_button("AI Notebook", Some("WEB"));
    nav_box.append(&notebook_btn);

    sidebar.append(&nav_box);
    sidebar.append(&Separator::new(Orientation::Horizontal));

    let recent_label = Label::new(Some("RECENT CHATS"));
    recent_label.add_css_class("sidebar-label");
    recent_label.set_halign(Align::Start);
    recent_label.set_margin_start(16);
    recent_label.set_margin_top(12);
    sidebar.append(&recent_label);

    let spacer = GtkBox::new(Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    sidebar.append(&spacer);

    sidebar.append(&Separator::new(Orientation::Horizontal));

    let footer = GtkBox::new(Orientation::Vertical, 6);
    footer.set_margin_top(12);
    footer.set_margin_bottom(12);
    footer.set_margin_start(8);
    footer.set_margin_end(8);

    let theme_btn = Button::with_label("☀  Toggle theme");
    theme_btn.add_css_class("footer-btn");
    footer.append(&theme_btn);

    let settings_btn = Button::with_label("⚙  Settings");
    settings_btn.add_css_class("footer-btn");
    footer.append(&settings_btn);

    sidebar.append(&footer);

    let collapsed = Rc::new(Cell::new(false));

    let sidebar_clone = sidebar.clone();
    let header_clone = header.clone();
    let logo_icon_clone = logo_icon.clone();
    let logo_vibi_clone = logo_vibi.clone();
    let logo_ai_clone = logo_ai.clone();
    let new_chat_btn_clone = new_chat_btn.clone();
    let nav_box_clone = nav_box.clone();
    let recent_label_clone = recent_label.clone();
    let footer_clone = footer.clone();
    let header_spacer_clone = header_spacer.clone();

    toggle_btn.connect_clicked(move |_| {
        let new_state = !collapsed.get();
        collapsed.set(new_state);

        logo_icon_clone.set_visible(!new_state);
        logo_vibi_clone.set_visible(!new_state);
        logo_ai_clone.set_visible(!new_state);
        new_chat_btn_clone.set_visible(!new_state);
        nav_box_clone.set_visible(!new_state);
        recent_label_clone.set_visible(!new_state);
        footer_clone.set_visible(!new_state);
        header_spacer_clone.set_visible(!new_state);

        if new_state {
            sidebar_clone.set_width_request(52);
            header_clone.set_margin_start(10);
            header_clone.set_margin_end(10);
        } else {
            sidebar_clone.set_width_request(260);
            header_clone.set_margin_start(16);
            header_clone.set_margin_end(16);
        }
    });

    sidebar
}

fn nav_item_button(label_text: &str, badge: Option<&str>) -> Button {
    let btn = Button::new();
    btn.add_css_class("nav-item");

    let content = GtkBox::new(Orientation::Horizontal, 0);
    let label = Label::new(Some(label_text));
    label.set_halign(Align::Start);
    label.set_hexpand(true);
    content.append(&label);

    if let Some(b) = badge {
        let badge_label = Label::new(Some(b));
        badge_label.add_css_class("badge-web");
        content.append(&badge_label);
    }

    btn.set_child(Some(&content));
    btn
}