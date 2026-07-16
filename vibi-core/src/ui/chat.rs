use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, Label, Orientation, Align, Separator, Stack};
use crate::chat_store::ChatStore;
use crate::ui::ai_notebook::AiNotebook;
use std::rc::Rc;
use std::cell::RefCell;
use crate::api::api_chat::ApiChat;

#[derive(Clone)]
pub struct ChatClearHandle {
    clear_fn: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl ChatClearHandle {
    pub fn new() -> Self {
        ChatClearHandle { clear_fn: Rc::new(RefCell::new(None)) }
    }
    pub fn set(&self, f: Box<dyn Fn()>) {
        *self.clear_fn.borrow_mut() = Some(f);
    }
    pub fn clear(&self) {
        if let Some(ref f) = *self.clear_fn.borrow() {
            f();
        }
    }
}

pub fn build_chat_view(
    chat_store: Rc<RefCell<ChatStore>>,
    logger: Rc<RefCell<crate::logger::Logger>>,
    ai_notebook: Rc<RefCell<AiNotebook>>,
    on_chat_update: Box<dyn Fn()>,
    on_title_change: Box<dyn Fn(String)>,
    on_reset_title: Box<dyn Fn()>,
) -> (GtkBox, GtkBox, ChatClearHandle) {
    // Check if API mode is enabled
    let storage = crate::storage::AppStorage::load();
    let api_mode = storage.api_mode;
    let openai_key = storage.openai_api_key.clone();
    let gemini_key = storage.gemini_api_key.clone();

    let api_chat = if api_mode {
        Some(Rc::new(crate::api_chat::ApiChat::new(openai_key, gemini_key)))
    } else {
        None
    };

    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);

    let topbar = GtkBox::new(Orientation::Horizontal, 0);
    topbar.style_context().add_class("topbar");
    topbar.set_margin_start(24);
    topbar.set_margin_end(24);
    topbar.set_margin_top(14);
    topbar.set_margin_bottom(14);

    let chat_title = Label::new(Some("VibiClaw"));
    chat_title.style_context().add_class("topbar-title");
    topbar.pack_start(&chat_title, false, false, 0);

    let divider = Separator::new(Orientation::Horizontal);
    divider.style_context().add_class("topbar-divider");

    root.pack_start(&topbar, false, false, 0);
    root.pack_start(&divider, false, false, 0);

    // Stack: dashboard or webview
    let view_stack = Stack::new();
    view_stack.set_hexpand(true);
    view_stack.set_vexpand(true);

    // === DASHBOARD PAGE ===
    let dashboard = build_dashboard();
    view_stack.add_titled(&dashboard, "dashboard", "Dashboard");

    // === WEBVIEW PAGE (placeholder, filled when AI selected) ===
    let webview_page = GtkBox::new(Orientation::Vertical, 0);
    view_stack.add_titled(&webview_page, "webview", "WebView");

    // API Chat page (native bubbles)
    let api_chat_page = build_api_chat_page(api_chat.clone(), logger.clone(), chat_store.clone());
    view_stack.add_titled(&api_chat_page, "api_chat", "API Chat");

    root.pack_start(&view_stack, true, true, 0);

    // AI Bridge (webview lives here)
    let ai_bridge = crate::ai_bridge::AiBridge::new();
    let ai_bridge_rc = Rc::new(ai_bridge);
    ai_bridge_rc.webview.set_hexpand(true);
    ai_bridge_rc.webview.set_vexpand(true);

    let bridge_send = ai_bridge_rc.clone();
    let bridge_switch = ai_bridge_rc.clone();

    let view_stack_dash = view_stack.clone();
    let chat_title_dash = chat_title.clone();

    // Build dashboard with clickable AI cards
    let ais = vec![
        ("chatgpt", "🟢", "ChatGPT", "GPT-5.5"),
        ("gemini", "🔵", "Gemini", "Gemini 2.5"),
        ("claude", "🟠", "Claude", "Claude Opus"),
        ("deepseek", "🐋", "DeepSeek", "DeepSeek-V3"),
        ("grok", "⚡", "Grok", "Grok-3"),
        ("qwen", "🟣", "Qwen", "Qwen 3.7"),
        ("kimi", "🌙", "Kimi", "Kimi 2.6"),
    ];

    let mut y = 0;
    for (id, emoji, name, desc) in &ais {
        let card = build_ai_card(emoji, name, desc);
        let id = id.to_string();
        let name = name.to_string();
        let bridge = bridge_switch.clone();
        let stack = view_stack_dash.clone();
        let title_label = chat_title_dash.clone();

        let api = api_chat.clone();
        card.connect_clicked(move |_| {
            if let Some(ref api_chat) = api {
                // API mode: start native chat
                api_chat.set_model(&id);
                api_chat.clear_history();
                title_label.set_text(&name);
                stack.set_visible_child_name("api_chat");
            } else {
                // Web mode: load webview
                bridge.load_model(&id);
                title_label.set_text(&name);
                stack.set_visible_child_name("webview");
            }
        });

        dashboard.attach(&card, y % 4, y / 4, 1, 1);
        y += 1;
    }

    // Back button (floating, only visible in webview)
    let back_btn = Button::with_label("← Back");
    back_btn.style_context().add_class("floating-back-btn");
    back_btn.set_halign(Align::Start);
    back_btn.set_valign(Align::Start);
    back_btn.set_margin_start(16);
    back_btn.set_margin_top(8);

    let stack_back = view_stack.clone();
    let title_back = chat_title.clone();
    back_btn.connect_clicked(move |_| {
        stack_back.set_visible_child_name("dashboard");
        title_back.set_text("VibiClaw");
    });

    webview_page.pack_start(&back_btn, false, false, 0);
    webview_page.pack_start(&ai_bridge_rc.webview, true, true, 0);

    view_stack.set_visible_child_name("dashboard");

    let clear_handle = ChatClearHandle::new();
    let store_for_clear = chat_store.clone();
    let chat_title_for_clear = chat_title.clone();
    clear_handle.set(Box::new(move || {
        if let Some(active) = store_for_clear.borrow().get_active() {
            chat_title_for_clear.set_text(&active.title);
        } else {
            chat_title_for_clear.set_text("VibiClaw");
        }
    }));

    let preview_panel = GtkBox::new(Orientation::Vertical, 0);
    preview_panel.set_width_request(0);
    preview_panel.set_visible(false);

    (root, preview_panel, clear_handle)
}

fn build_dashboard() -> gtk::Grid {
    let grid = gtk::Grid::new();
    grid.set_row_spacing(16);
    grid.set_column_spacing(16);
    grid.set_halign(Align::Center);
    grid.set_valign(Align::Center);
    grid.set_margin_top(40);
    grid.set_margin_bottom(40);
    grid.set_margin_start(40);
    grid.set_margin_end(40);
    grid
}

fn build_ai_card(emoji: &str, name: &str, desc: &str) -> Button {
    let card = Button::new();
    card.style_context().add_class("ai-dashboard-card");
    card.set_size_request(180, 140);

    let content = GtkBox::new(Orientation::Vertical, 10);
    content.set_margin_top(16);
    content.set_margin_bottom(16);

    let icon = Label::new(Some(emoji));
    icon.style_context().add_class("ai-dashboard-icon");
    icon.set_halign(Align::Center);
    content.pack_start(&icon, false, false, 0);

    let name_label = Label::new(Some(name));
    name_label.style_context().add_class("ai-dashboard-name");
    name_label.set_halign(Align::Center);
    content.pack_start(&name_label, false, false, 0);

    let desc_label = Label::new(Some(desc));
    desc_label.style_context().add_class("ai-dashboard-desc");
    desc_label.set_halign(Align::Center);
    content.pack_start(&desc_label, false, false, 0);

    card.add(&content);
    card

    fn build_api_chat_page(
    api_chat: Option<Rc<ApiChat>>,
    logger: Rc<RefCell<crate::logger::Logger>>,
    chat_store: Rc<RefCell<ChatStore>>,
) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.set_hexpand(true);
    container.set_vexpand(true);

    let scroll = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroll.set_vexpand(true);
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

    let message_area = GtkBox::new(Orientation::Vertical, 8);
    message_area.set_vexpand(true);
    message_area.set_valign(Align::Start);
    message_area.set_margin_start(24);
    message_area.set_margin_end(24);
    message_area.set_margin_top(16);
    message_area.set_margin_bottom(16);
    scroll.add(&message_area);
    container.pack_start(&scroll, true, true, 0);

    // Input bar
    let input_area = GtkBox::new(Orientation::Horizontal, 8);
    input_area.set_margin_start(24);
    input_area.set_margin_end(24);
    input_area.set_margin_bottom(16);
    input_area.style_context().add_class("input-box");

    let entry = gtk::Entry::new();
    entry.set_placeholder_text(Some("Type your message..."));
    entry.style_context().add_class("chat-entry");
    entry.set_hexpand(true);
    input_area.pack_start(&entry, true, true, 0);

    let send_btn = Button::with_label("➤");
    send_btn.style_context().add_class("send-btn");
    input_area.pack_start(&send_btn, false, false, 0);

    container.pack_start(&input_area, false, false, 0);

    // Send handler
    if let Some(ref api) = api_chat {
        let api = api.clone();
        let msg_area = message_area.clone();
        let entry_clone = entry.clone();
        let logger = logger.clone();
        let scroll_clone = scroll.clone();

        send_btn.connect_clicked(move |_| {
            let text = entry_clone.text().to_string();
            if text.trim().is_empty() { return; }

            // User bubble
            let user_row = GtkBox::new(Orientation::Horizontal, 0);
            user_row.set_halign(Align::End);
            user_row.set_margin_bottom(4);
            let user_bubble = Label::new(Some(&text));
            user_bubble.set_wrap(true);
            user_bubble.set_max_width_chars(50);
            user_bubble.style_context().add_class("user-bubble");
            user_row.pack_start(&user_bubble, false, false, 0);
            msg_area.pack_start(&user_row, false, false, 0);
            user_row.show_all();
            entry_clone.set_text("");

            // Send to API
            let api = api.clone();
            let msg_area = msg_area.clone();
            let logger = logger.clone();
            let scroll = scroll_clone.clone();
            std::thread::spawn(move || {
                match api.send_message(&text) {
                    Ok(response) => {
                        logger.borrow_mut().log(
                            crate::logger::LogLevel::Info,
                            "API",
                            &format!("Response: {} tokens", response.tokens_used),
                        );

                        // Check for VibiClaw blocks
                        let vibi_results = ApiChat::execute_vibi_blocks(&response.content);

                        gtk::glib::idle_add_local(move || {
                            let ai_row = GtkBox::new(Orientation::Horizontal, 0);
                            ai_row.set_halign(Align::Start);
                            ai_row.set_margin_bottom(4);
                            let ai_bubble = Label::new(Some(&response.content));
                            ai_bubble.set_wrap(true);
                            ai_bubble.set_max_width_chars(50);
                            ai_bubble.style_context().add_class("ai-bubble");
                            ai_row.pack_start(&ai_bubble, false, false, 0);
                            msg_area.pack_start(&ai_row, false, false, 0);
                            ai_row.show_all();

                            // Show VibiClaw results
                            for result in &vibi_results {
                                let result_row = GtkBox::new(Orientation::Horizontal, 0);
                                result_row.set_halign(Align::Start);
                                let result_label = Label::new(Some(result));
                                result_label.style_context().add_class("vibi-result");
                                result_row.pack_start(&result_label, false, false, 0);
                                msg_area.pack_start(&result_row, false, false, 0);
                                result_row.show_all();
                            }

                            // Scroll to bottom
                            let adj = scroll.vadjustment();
                            adj.set_value(adj.upper());
                            gtk::glib::ControlFlow::Break
                        });
                    }
                    Err(e) => {
                        gtk::glib::idle_add_local(move || {
                            let error_label = Label::new(Some(&format!("Error: {}", e)));
                            error_label.style_context().add_class("vibi-error");
                            msg_area.pack_start(&error_label, false, false, 0);
                            error_label.show_all();
                            gtk::glib::ControlFlow::Break
                        });
                    }
                }
            });
        });

        entry.connect_activate({
            let btn = send_btn.clone();
            move |_| btn.emit_clicked()
        });
    }

    container
}
}