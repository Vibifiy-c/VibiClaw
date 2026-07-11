use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, Label, Orientation, Align, ScrolledWindow, PolicyType, Window, WindowType, Separator};
use webkit2gtk::{WebView, WebViewExt};
use gio;
use std::rc::Rc;
use std::cell::RefCell;

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
        ("claude", "🟠", "Claude", "claude.ai", "https://claude.ai"),
        ("gemini", "🔵", "Gemini", "gemini.google.com", "https://accounts.google.com/ServiceLogin?service=gemini"),
        ("deepseek", "🐋", "DeepSeek", "chat.deepseek.com", "https://chat.deepseek.com/sign_in"),
        ("grok", "⚡", "Grok", "grok.com", "https://grok.com/restore"),
        ("qwen", "🟣", "Qwen", "tongyi.aliyun.com", "https://tongyi.aliyun.com/qianwen/"),
        ("kimi", "🌙", "Kimi", "kimi.com", "https://www.kimi.com"),
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
    card.set_size_request(240, 220);
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
    
    let btn_row = GtkBox::new(Orientation::Horizontal, 8);
    btn_row.set_halign(Align::Center);
    
    let login_btn = Button::with_label("Login");
    login_btn.style_context().add_class("login-ai-btn");
    btn_row.pack_start(&login_btn, false, false, 0);
    
    let logout_btn = Button::with_label("Logout");
    logout_btn.style_context().add_class("login-ai-logout-btn");
    logout_btn.set_visible(false);
    btn_row.pack_start(&logout_btn, false, false, 0);
    
    card.pack_start(&btn_row, false, false, 0);
    
    let url_owned = url.to_string();
    let status_clone = status.clone();
    let login_clone = login_btn.clone();
    let logout_clone = logout_btn.clone();
    
    login_btn.connect_clicked(move |_| {
        open_login_popup(&url_owned, status_clone.clone(), login_clone.clone(), logout_clone.clone());
    });
    
    let url_logout = url.to_string();
    let status_logout = status.clone();
    let login_logout = login_btn.clone();
    let logout_logout = logout_btn.clone();
    logout_btn.connect_clicked(move |_| {
        clear_cookies_and_reset(&url_logout, status_logout.clone(), login_logout.clone(), logout_logout.clone());
    });
    
    card
}

fn open_login_popup(url: &str, status_label: Label, login_btn: Button, logout_btn: Button) {
    let popup = Window::new(WindowType::Toplevel);
    popup.set_title("Login");
    popup.set_default_size(500, 650);
    popup.set_modal(true);
    
    let container = GtkBox::new(Orientation::Vertical, 0);
    
    let toolbar = GtkBox::new(Orientation::Horizontal, 8);
    toolbar.set_margin_start(8);
    toolbar.set_margin_end(8);
    toolbar.set_margin_top(4);
    toolbar.set_margin_bottom(4);
    
    let done_btn = Button::with_label("✓ Done");
    done_btn.style_context().add_class("login-done-btn");
    let s = status_label.clone();
    let lb = login_btn.clone();
    let lo = logout_btn.clone();
    let pc = popup.clone();
    done_btn.connect_clicked(move |_| {
        s.set_text("🟢 Logged in");
        lb.set_label("Re-login");
        lo.set_visible(true);
        pc.close();
    });
    toolbar.pack_end(&done_btn, false, false, 0);
    
    container.pack_start(&toolbar, false, false, 0);
    
    let webview = WebView::new();
    webview.set_hexpand(true);
    webview.set_vexpand(true);
    webview.load_uri(url);
    
    let wv = webview.clone();
    let logged_in = Rc::new(RefCell::new(false));
    let li = logged_in.clone();
    let s2 = status_label.clone();
    let lb2 = login_btn.clone();
    let lo2 = logout_btn.clone();
    let popup2 = popup.clone();
    
    webview.connect_load_changed(move |_, event| {
        if event == webkit2gtk::LoadEvent::Finished && !*li.borrow() {
            let s3 = s2.clone();
            let lb3 = lb2.clone();
            let lo3 = lo2.clone();
            let popup3 = popup2.clone();
            let li2 = li.clone();
            let wv2 = wv.clone();
            
            let js = "document.querySelector('.avatar') || document.querySelector('[data-testid=\"profile-button\"]') || document.querySelector('.gb_Ja') || document.querySelector('[aria-label*=\"Account\"]') || document.querySelector('[class*=\"profile\"]') || document.querySelector('[class*=\"user\"]') ? 'logged-in' : 'not-logged'";
            
            wv2.run_javascript(js, None::<&gio::Cancellable>, move |result| {
                if let Ok(res) = result {
                    let raw = format!("{:?}", res);
                    if raw.contains("logged-in") {
                        *li2.borrow_mut() = true;
                        s3.set_text("🟢 Logged in");
                        lb3.set_label("Re-login");
                        lo3.set_visible(true);
                        gtk::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                            popup3.close();
                            gtk::glib::ControlFlow::Break
                        });
                    }
                }
            });
        }
    });
    
    container.pack_start(&webview, true, true, 0);
    popup.add(&container);
    popup.show_all();
}

fn clear_cookies_and_reset(url: &str, status_label: Label, login_btn: Button, logout_btn: Button) {
    let webview = WebView::new();
    webview.load_uri(url);
    let js = "document.cookie.split(';').forEach(function(c) { document.cookie = c.replace(/^ +/, '').replace(/=.*/, '=;expires=' + new Date().toUTCString() + ';path=/'); });";
    let status = status_label.clone();
    let lb = login_btn.clone();
    let lo = logout_btn.clone();
    webview.connect_load_changed(move |wv, event| {
        if event == webkit2gtk::LoadEvent::Finished {
            let s = status.clone();
            let l = lb.clone();
            let o = lo.clone();
            wv.run_javascript(js, None::<&gio::Cancellable>, move |_| {
                s.set_text("🔴 Not logged in");
                l.set_label("Login");
                o.set_visible(false);
            });
        }
    });
}