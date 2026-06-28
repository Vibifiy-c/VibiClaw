use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, Orientation, Button, Entry, Separator};
use webkit2gtk::{WebView, WebViewExt};

pub fn build_browser_view() -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);

    let toolbar = GtkBox::new(Orientation::Horizontal, 8);
    toolbar.style_context().add_class("browser-toolbar");
    toolbar.set_margin_start(8);
    toolbar.set_margin_end(8);
    toolbar.set_margin_top(8);
    toolbar.set_margin_bottom(4);

    let back_btn = Button::with_label("←");
    back_btn.style_context().add_class("browser-nav-btn");
    toolbar.pack_start(&back_btn, false, false, 0);

    let forward_btn = Button::with_label("→");
    forward_btn.style_context().add_class("browser-nav-btn");
    toolbar.pack_start(&forward_btn, false, false, 0);

    let refresh_btn = Button::with_label("↻");
    refresh_btn.style_context().add_class("browser-nav-btn");
    toolbar.pack_start(&refresh_btn, false, false, 0);

    let home_btn = Button::with_label("🏠");
    home_btn.style_context().add_class("browser-nav-btn");
    toolbar.pack_start(&home_btn, false, false, 0);

    let url_entry = Entry::new();
    url_entry.set_placeholder_text(Some("Enter URL or search..."));
    url_entry.style_context().add_class("browser-url-bar");
    url_entry.set_hexpand(true);
    toolbar.pack_start(&url_entry, true, true, 0);

    let go_btn = Button::with_label("Go");
    go_btn.style_context().add_class("browser-go-btn");
    toolbar.pack_start(&go_btn, false, false, 0);

    root.pack_start(&toolbar, false, false, 0);

    let webview = WebView::new();
    webview.set_hexpand(true);
    webview.set_vexpand(true);
    webview.load_uri("https://www.google.com");

    let wv = webview.clone();
    let url_clone = url_entry.clone();
    go_btn.connect_clicked(move |_| {
        let mut url = url_clone.text().to_string();
        if !url.contains("://") { url = format!("https://{}", url); }
        wv.load_uri(&url);
    });

    let wv2 = webview.clone();
    url_entry.connect_activate(move |e| {
        let mut url = e.text().to_string();
        if !url.contains("://") { url = format!("https://{}", url); }
        wv2.load_uri(&url);
    });

    let wv_back = webview.clone();
    back_btn.connect_clicked(move |_| { wv_back.go_back(); });

    let wv_fwd = webview.clone();
    forward_btn.connect_clicked(move |_| { wv_fwd.go_forward(); });

    let wv_refresh = webview.clone();
    refresh_btn.connect_clicked(move |_| { wv_refresh.reload(); });

    let wv_home = webview.clone();
    home_btn.connect_clicked(move |_| { wv_home.load_uri("https://www.google.com"); });

    let url_bar = url_entry.clone();
    webview.connect_uri_notify(move |wv| {
        if let Some(uri) = wv.uri() {
            url_bar.set_text(&uri.to_string());
        }
    });

    root.pack_start(&webview, true, true, 0);
    root
}