mod types;
mod analyzer;
mod sandbox;
mod executor;
mod commands;
mod ui;

use gtk4::prelude::*;
use gtk4::{Application, CssProvider};
use gtk4::gdk::Display;

fn main() {
    let app = Application::builder()
        .application_id("com.vibi.ai")
        .build();

    app.connect_startup(|_| load_css());
    app.connect_activate(ui::build_window);

    app.run();
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("ui/style.css"));

    gtk4::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}