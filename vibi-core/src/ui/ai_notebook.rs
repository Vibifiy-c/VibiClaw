use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, Orientation, Notebook, ScrolledWindow, PolicyType};
use webkit2gtk::{WebView, WebViewExt};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub struct AiNotebook {
    pub container: GtkBox,
    pub notebook: Notebook,
    pub webviews: Rc<RefCell<HashMap<String, WebView>>>,
}

pub fn build_ai_notebook() -> AiNotebook {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.set_hexpand(true);
    container.set_vexpand(true);
    
    let topbar = GtkBox::new(Orientation::Horizontal, 0);
    topbar.style_context().add_class("topbar");
    topbar.set_margin_start(24);
    topbar.set_margin_end(24);
    topbar.set_margin_top(14);
    topbar.set_margin_bottom(14);
    
    let title = Label::new(Some("AI Notebook"));
    title.style_context().add_class("topbar-title");
    topbar.pack_start(&title, false, false, 0);
    
    let divider = gtk::Separator::new(Orientation::Horizontal);
    divider.style_context().add_class("topbar-divider");
    
    container.pack_start(&topbar, false, false, 0);
    container.pack_start(&divider, false, false, 0);
    
    let notebook = Notebook::new();
    notebook.set_hexpand(true);
    notebook.set_vexpand(true);
    notebook.set_scrollable(true);
    notebook.set_tab_pos(gtk::PositionType::Top);
    
    let webviews: Rc<RefCell<HashMap<String, WebView>>> = Rc::new(RefCell::new(HashMap::new()));
    
    let models = vec![
        ("chatgpt", "🟢 ChatGPT", "https://chat.openai.com"),
        ("claude", "🟠 Claude", "https://claude.ai"),
        ("gemini", "🔵 Gemini", "https://gemini.google.com"),
        ("deepseek", "🐋 DeepSeek", "https://chat.deepseek.com"),
        ("grok", "⚡ Grok", "https://grok.com"),
        ("qwen", "🟣 Qwen", "https://tongyi.aliyun.com/qianwen"),
        ("kimi", "🌙 Kimi", "https://kimi.moonshot.cn"),
    ];
    
    for (id, name, url) in &models {
        let scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        scroll.set_policy(PolicyType::Automatic, PolicyType::Automatic);
        
        let webview = WebView::new();
        webview.set_hexpand(true);
        webview.set_vexpand(true);
        webview.load_uri(url);
        
        scroll.add(&webview);
        
        let tab_label = GtkBox::new(Orientation::Horizontal, 4);
        let parts: Vec<&str> = name.splitn(2, ' ').collect();
        let icon_text = parts.get(0).unwrap_or(&"");
        let name_text = parts.get(1).unwrap_or(&"");
        let icon = Label::new(Some(icon_text));
        let text = Label::new(Some(name_text));
        tab_label.pack_start(&icon, false, false, 0);
        tab_label.pack_start(&text, false, false, 0);
        tab_label.show_all();
        
        notebook.append_page(&scroll, Some(&tab_label));
        webviews.borrow_mut().insert(id.to_string(), webview.clone());
    }
    
    container.pack_start(&notebook, true, true, 0);
    
    AiNotebook {
        container,
        notebook,
        webviews,
    }
}