mod types;
mod analyzer;
mod sandbox;
mod executor;
mod commands;
mod ui;
mod user_commands;
mod storage;
mod chat_store;
mod crypto;
mod logger;

use gtk::prelude::*;
use gtk::{Application, CssProvider};

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
    provider.load_from_data(include_str!("ui/style.css").as_bytes()).ok();
    
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::default().expect("Could not connect to display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
