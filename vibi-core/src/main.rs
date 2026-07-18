mod types;
mod sandbox;
mod executor;
mod ui;
mod ai_bridge;
mod vibi_lang;
mod storage;
mod chat_store;
mod crypto;
mod logger;
mod debug;
mod hardware_usage;
pub mod api;

use gtk::prelude::*;
use gtk::{Application, CssProvider};
use glib::LogLevel;

fn main() {
    // Temp: test VibiClaw compiler
    let test_path = std::path::Path::new("test.vl");
    if test_path.exists() {
        let source = std::fs::read_to_string(test_path).expect("Failed to read test.v");
        vibi_lang::cli::test_compile(&source);
    } else {
        println!("No test.v found, create one to test the compiler");
    }
    // Global panic hook - logs crashes with location
    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        let location = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
        eprintln!("[main.rs] [Error] in {} : {}", location, msg);
    }));
    
    // GTK/GLib warning and critical logs
    glib::log_set_default_handler(|domain, level, msg| {
        let domain_str = domain.unwrap_or_default();
        let level_str = match level {
            LogLevel::Error | LogLevel::Critical => "[Error]",
            LogLevel::Warning => "[Warning]",
            _ => "[Debug]",
        };
        eprintln!("[main.rs] {} in {} : {}", level_str, domain_str, msg);
    });
    
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    std::env::set_var("GTK_CSD", "0");
    
    let webkit_data = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("vibi-ai")
        .join("webkit");
    std::fs::create_dir_all(&webkit_data).ok();
    std::env::set_var("WEBKIT_LOCAL_STORAGE_DIRECTORY", webkit_data.to_str().unwrap());
    
    let app = Application::builder()
        .application_id("com.vibi.ai")
        .build();

    app.connect_startup(|_| {
        load_css();
        hardware_usage::start_hardware_server();
    });
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
