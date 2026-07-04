use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, Label, Orientation, Align, ScrolledWindow, PolicyType, Window, WindowType, Separator};
use webkit2gtk::{WebView, WebViewExt};
use gio;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone)]
pub struct LoginCenterPage {
    pub container: GtkBox,
}

pub fn build_login_center_page(stack: gtk::Stack) -> LoginCenterPage {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.set_hexpand(true);
    container.set_vexpand(true);
    
    let topbar = GtkBox::new(Orientation::Horizontal, 0);
    topbar.style_context().add_class("topbar");
    topbar.set_margin_start(24);
    topbar.set_margin_end(24);
    topbar.set_margin_top(14);
    topbar.set_margin_bottom(14);
    
    let back_btn = Button::with_label("← Back");
    back_btn.style_context().add_class("login-back-btn");
    let s = stack.clone();
    back_btn.connect_clicked(move |_| s.set_visible_child_name("ai_notebook"));
    topbar.pack_start(&back_btn, false, false, 0);
    
    let title = Label::new(Some("AI Login Center"));
    title.style_context().add_class("topbar-title");
    title.set_margin_start(12);
    topbar.pack_start(&title, false, false, 0);
    
    let divider = Separator::new(Orientation::Horizontal);
    divider.style_context().add_class("topbar-divider");
    
    container.pack_start(&topbar, false, false, 0);
    container.pack_start(&divider, false, false, 0);
    
    let scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroll.set_vexpand(true);
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
    
    let content = GtkBox::new(Orientation::Vertical, 24);
    content.set_margin_start(60);
    content.set_margin_end(60);
    content.set_margin_top(40);
    content.set_margin_bottom(40);
    content.set_halign(Align::Center);
    content.set_valign(Align::Start);
    
    let grid = gtk::FlowBox::new();
    grid.set_selection_mode(gtk::SelectionMode::None);
    grid.set_min_children_per_line(2);
    grid.set_max_children_per_line(4);
    grid.set_homogeneous(true);
    grid.set_row_spacing(16);
    grid.set_column_spacing(16);
    grid.set_hexpand(true);
    
    let ais = vec![
        ("chatgpt", "🟢", "ChatGPT", "chat.openai.com", "https://auth.openai.com/log-in"),
        ("claude", "🟠", "Claude", "claude.ai", "https://claude.ai/login"),
        ("gemini", "🔵", "Gemini", "gemini.google.com", "https://accounts.google.com/ServiceLogin?service=gemini"),
        ("deepseek", "🐋", "DeepSeek", "chat.deepseek.com", "https://chat.deepseek.com/sign_in"),
        ("grok", "⚡", "Grok", "grok.com", "https://grok.com/restore"),
        ("qwen", "🟣", "Qwen", "tongyi.aliyun.com", "https://tongyi.aliyun.com/qianwen/"),
        ("kimi", "🌙", "Kimi", "kimi.moonshot.cn", "https://kimi.moonshot.cn/"),
    ];
    
    for (id, emoji, name, domain, url) in &ais {
        let card = build_ai_login_card(id, emoji, name, domain, url);
        grid.insert(&card, -1);
    }
    
    content.pack_start(&grid, false, false, 0);
    scroll.add(&content);
    container.pack_start(&scroll, true, true, 0);
    
    LoginCenterPage { container }
}

fn build_ai_login_card(id: &str, emoji: &str, name: &str, domain: &str, url: &str) -> GtkBox {
    let card = GtkBox::new(Orientation::Vertical, 16);
    card.style_context().add_class("login-ai-card");
    card.set_size_request(240, 200);
    card.set_margin_start(8);
    card.set_margin_end(8);
    
    let icon = Label::new(Some(emoji));
    icon.style_context().add_class("login-ai-icon");
    icon.set_halign(Align::Center);
    card.pack_start(&icon, false, false, 0);
    
    let name_label = Label::new(Some(name));
    name_label.style_context().add_class("login-ai-name");
    name_label.set_halign(Align::Center);
    card.pack_start(&name_label, false, false, 0);
    
    let domain_label = Label::new(Some(domain));
    domain_label.style_context().add_class("login-ai-domain");
    domain_label.set_halign(Align::Center);
    card.pack_start(&domain_label, false, false, 0);
    
    let status = Label::new(Some("🔴 Not logged in"));
    status.style_context().add_class("login-ai-status");
    status.set_halign(Align::Center);
    card.pack_start(&status, false, false, 0);
    
    let btn = Button::with_label("Login");
    btn.style_context().add_class("login-ai-btn");
    btn.set_halign(Align::Center);
    card.pack_start(&btn, false, false, 0);
    
    let url_owned = url.to_string();
    let status_clone = status.clone();
    let btn_clone = btn.clone();
    
    btn.connect_clicked(move |_| {
        open_login_popup(&url_owned, status_clone.clone(), btn_clone.clone());
    });
    
    card
}

fn open_login_popup(url: &str, status_label: Label, login_btn: Button) {
    let popup = Window::new(WindowType::Toplevel);
    popup.set_title("Login");
    popup.set_default_size(500, 650);
    popup.set_modal(true);
    
    let webview = WebView::new();
    webview.set_hexpand(true);
    webview.set_vexpand(true);
    webview.load_uri(url);
    
    let wv = webview.clone();
    let status = status_label.clone();
    let btn = login_btn.clone();
    let popup_clone = popup.clone();
    
    webview.connect_load_changed(move |_, event| {
        if event == webkit2gtk::LoadEvent::Finished {
            let status = status.clone();
            let btn = btn.clone();
            let popup = popup_clone.clone();
            
            let js = "document.querySelector('.avatar') || document.querySelector('[data-testid=\"profile-button\"]') || document.querySelector('.gb_Ja') || document.querySelector('[aria-label*=\"Account\"]') ? 'logged-in' : 'not-logged'";
            
            wv.run_javascript(js, None::<&gio::Cancellable>, move |result| {
                if let Ok(res) = result {
                    let raw = format!("{:?}", res);
                    if raw.contains("logged-in") {
                        status.set_text("🟢 Logged in");
                        btn.set_label("Re-login");
                        popup.close();
                    }
                }
            });
        }
    });
    
    popup.add(&webview);
    popup.show_all();
}