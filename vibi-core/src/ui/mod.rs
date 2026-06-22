pub mod sidebar;
pub mod chat;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Orientation, ScrolledWindow, PolicyType, Label, Button, Separator};
use std::rc::Rc;
use std::cell::RefCell;

pub fn build_window(app: &Application) {
    let root = GtkBox::new(Orientation::Horizontal, 0);

    let sb = sidebar::build_sidebar();
    root.append(&sb);

    let main_area = GtkBox::new(Orientation::Vertical, 0);
    main_area.set_hexpand(true);
    main_area.set_vexpand(true);
    main_area.add_css_class("main-area");

    let (chat_view, preview_panel) = chat::build_chat_view();
    main_area.append(&chat_view);

    root.append(&main_area);
    root.append(&preview_panel);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Vibi AI")
        .default_width(1280)
        .default_height(800)
        .child(&root)
        .build();

    window.add_css_class("light");

    window.present();
}