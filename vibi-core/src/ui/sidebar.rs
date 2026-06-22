use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, Separator, Align};
use gtk4::glib;
use std::rc::Rc;
use std::cell::{Cell, RefCell};

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

    let chat_btn = nav_item_button_with_icon("Chat", "💬", None);
    chat_btn.add_css_class("active");
    nav_box.append(&chat_btn);

    let agentic_btn = nav_item_button_with_icon("Agentic Tool", "🕵️", None);
    nav_box.append(&agentic_btn);

    let notebook_btn = nav_item_button_with_icon("AI Notebook", "📓", Some("WEB"));
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

    let theme_btn = footer_button_with_icon("☀", "Toggle theme");
    footer.append(&theme_btn);

    theme_btn.connect_clicked(move |btn| {
        let window = btn.root().and_then(|r| r.downcast::<gtk4::ApplicationWindow>().ok());
        if let Some(win) = window {
            let is_light = !win.css_classes().iter().any(|c| c == "light");
            if is_light {
                win.add_css_class("light");
            } else {
                win.remove_css_class("light");
            }
            if let Some(child) = btn.child() {
                if let Some(content) = child.downcast_ref::<GtkBox>() {
                    if let Some(icon) = content.first_child() {
                        if let Some(label) = icon.downcast_ref::<Label>() {
                            label.set_text(if is_light { "☾" } else { "☀" });
                        }
                    }
                }
            }
        }
    });

    let settings_btn = footer_button_with_icon("⚙", "Settings");
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
    let chat_btn_clone = chat_btn.clone();
    let agentic_btn_clone = agentic_btn.clone();
    let notebook_btn_clone = notebook_btn.clone();
    let theme_btn_clone_anim = theme_btn.clone();
    let settings_btn_clone = settings_btn.clone();

    let counter = Rc::new(RefCell::new(0));

    toggle_btn.connect_clicked(move |_| {
        let new_state = !collapsed.get();
        collapsed.set(new_state);

        let start_width = if new_state { 260 } else { 52 };
        let end_width = if new_state { 52 } else { 260 };
        let start_margin = if new_state { 16 } else { 10 };
        let end_margin = if new_state { 10 } else { 16 };

        logo_icon_clone.set_visible(!new_state);
        logo_vibi_clone.set_visible(!new_state);
        logo_ai_clone.set_visible(!new_state);
        new_chat_btn_clone.set_visible(!new_state);
        nav_box_clone.set_visible(!new_state);
        recent_label_clone.set_visible(!new_state);
        footer_clone.set_visible(!new_state);
        header_spacer_clone.set_visible(!new_state);

        hide_nav_labels(&chat_btn_clone, new_state);
        hide_nav_labels(&agentic_btn_clone, new_state);
        hide_nav_labels(&notebook_btn_clone, new_state);
        hide_footer_labels(&theme_btn_clone_anim, new_state);
        hide_footer_labels(&settings_btn_clone, new_state);

        let sidebar_anim = sidebar_clone.clone();
        let header_anim = header_clone.clone();
        let counter_clone = counter.clone();
        let steps = 20;
        let step_ms = 10;

        *counter_clone.borrow_mut() = 0;

        glib::timeout_add_local(std::time::Duration::from_millis(step_ms), move || {
            let mut count = counter_clone.borrow_mut();
            *count += 1;

            let progress = *count as f64 / steps as f64;
            let current_width = start_width as f64 + (end_width as f64 - start_width as f64) * progress;
            let current_margin = start_margin as f64 + (end_margin as f64 - start_margin as f64) * progress;

            sidebar_anim.set_width_request(current_width as i32);
            header_anim.set_margin_start(current_margin as i32);
            header_anim.set_margin_end(current_margin as i32);

            if *count >= steps {
                sidebar_anim.set_width_request(end_width);
                header_anim.set_margin_start(end_margin);
                header_anim.set_margin_end(end_margin);
                return glib::ControlFlow::Break;
            }
            glib::ControlFlow::Continue
        });
    });

    sidebar
}

fn nav_item_button_with_icon(label_text: &str, icon: &str, badge: Option<&str>) -> Button {
    let btn = Button::new();
    btn.add_css_class("nav-item");

    let content = GtkBox::new(Orientation::Horizontal, 8);
    let icon_label = Label::new(Some(icon));
    icon_label.add_css_class("nav-icon");
    content.append(&icon_label);

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

fn hide_nav_labels(btn: &Button, collapsed: bool) {
    if let Some(child) = btn.child() {
        if let Some(content) = child.downcast_ref::<GtkBox>() {
            let mut current = content.first_child();
            let mut index = 0;
            while let Some(widget) = current {
                if index == 0 {
                    widget.set_visible(true);
                } else {
                    widget.set_visible(!collapsed);
                }
                current = widget.next_sibling();
                index += 1;
            }
        }
    }
    if collapsed {
        btn.set_halign(Align::Center);
        btn.set_width_request(44);
    } else {
        btn.set_halign(Align::Fill);
        btn.set_width_request(-1);
    }
}

fn hide_footer_labels(btn: &Button, collapsed: bool) {
    if let Some(child) = btn.child() {
        if let Some(content) = child.downcast_ref::<GtkBox>() {
            let mut current = content.first_child();
            let mut index = 0;
            while let Some(widget) = current {
                if index == 0 {
                    widget.set_visible(true);
                } else {
                    widget.set_visible(!collapsed);
                }
                current = widget.next_sibling();
                index += 1;
            }
        }
    }
    if collapsed {
        btn.set_halign(Align::Center);
        btn.set_width_request(44);
    } else {
        btn.set_halign(Align::Fill);
        btn.set_width_request(-1);
    }
}

fn footer_button_with_icon(icon: &str, label_text: &str) -> Button {
    let btn = Button::new();
    btn.add_css_class("footer-btn");

    let content = GtkBox::new(Orientation::Horizontal, 8);
    let icon_label = Label::new(Some(icon));
    icon_label.add_css_class("footer-icon");
    content.append(&icon_label);

    let label = Label::new(Some(label_text));
    content.append(&label);

    btn.set_child(Some(&content));
    btn
}