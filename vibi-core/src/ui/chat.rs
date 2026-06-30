use gtk::prelude::*;

use gtk::{Box as GtkBox, Button, Entry, Label, Orientation, Align, ScrolledWindow, PolicyType, Separator};
use gtk::glib;
use gtk::FileChooserDialog;

use crate::user_commands::{CommandRegistry, CommandSuggestionPopover};
use crate::chat_store::ChatStore;
use std::rc::Rc;
use std::cell::RefCell;
use rand::Rng;

const WELCOME_QUOTES: &[&str] = &[
    "The stage is yours!",
    "What's the agenda today?",
    "You're about to build something amazing today!",
    "Something great is about to be created!",
    "Ready to ship greatness?",
    "Let's make some magic happen.",
    "Your code awaits, legend.",
    "Time to turn ideas into reality.",
    "Another day, another masterpiece.",
    "Vibe check: passed. Let's code.",
    "The terminal is your canvas.",
    "Dream big, code bigger.",
    "Innovation starts with a single prompt.",
    "Build fast, stay creative.",
    "Your future self is already proud.",
    "Lock in. It's go time.",
    "Great things never come from comfort zones.",
    "You're one prompt away from genius.",
    "Make it work, make it right, make it fast.",
    "The best code is the code you ship.",
    "Create something worth remembering.",
    "Your potential is unlimited.",
    "Time to outdo your past self.",
    "Focus mode: activated.",
    "Let your creativity run wild.",
    "Code is poetry in motion.",
    "Every expert was once a beginner.",
    "Ship it like you mean it.",
    "The world needs what you're building.",
    "Start small, dream big, ship fast.",
    "Your keyboard is a magic wand.",
    "Today's code is tomorrow's legacy.",
    "No limits, just possibilities.",
    "Build the change you want to see.",
    "The prompt is loaded. Execute.",
    "Channel your inner 10x engineer.",
    "Simplicity is the ultimate sophistication.",
    "First, solve the problem. Then, write the code.",
    "Make it so good they can't ignore you.",
    "The quieter you become, the more you can code.",
    "Talk is cheap. Show me the code.",
    "It's not a bug — it's an undocumented feature.",
    "Move fast and build things.",
    "Clean code always looks like it was written by someone who cares.",
    "The only way to do great work is to love what you do.",
    "Stay hungry, stay foolish, stay coding.",
    "Your only limit is your mind.",
    "Debug the world, one line at a time.",
    "Don't wait for opportunity. Create it.",
    "The best time to start was yesterday. The next best is now.",
    "Push the boundaries of what's possible.",
    "Craftsmanship over convenience.",
    "Build bridges, not walls.",
    "Think twice, code once.",
    "Your imagination is the only limit.",
    "Dare to be different. Code boldly.",
    "Small steps lead to giant leaps.",
    "The compiler is your friend... mostly.",
    "Creativity is intelligence having fun.",
    "Design is not just what it looks like. Design is how it works.",
    "In code we trust.",
    "Elegance is not optional.",
    "The best error message is no error message.",
    "Write code that tells a story.",
    "First do it, then do it right, then do it better.",
    "Software is a great combination of artistry and engineering.",
    "The function of good software is to make the complex appear simple.",
    "Perfection is achieved when there is nothing left to take away.",
    "It works on my machine.",
    "I'm not a great programmer; I'm just a good programmer with great habits.",
    "Programming is the art of telling another human what you want the computer to do.",
    "A language that doesn't affect how you think is not worth knowing.",
    "The art of programming is the skill of controlling complexity.",
    "Good code is its own best documentation.",
    "The best programs are written so that computers can execute them quickly and humans can understand them clearly.",
    "Programming is the closest thing we have to a superpower.",
    "Code is like humor. When you have to explain it, it's bad.",
    "Fix the cause, not the symptom.",
    "Before software can be reusable, it first has to be usable.",
    "The best thing about a boolean is even if you are wrong, you are only off by a bit.",
    "Better to have 100 functions operate on one data structure than 10 functions on 10 data structures.",
    "The cheapest, fastest, and most reliable components are those that aren't there.",
    "Deleted code is debugged code.",
    "A system that is simple in its core is easier to maintain and extend.",
    "The purpose of software engineering is to control complexity, not to create it.",
    "Good design adds value faster than it adds cost.",
    "The best developers are lazy and dumb — they automate everything and keep things simple.",
    "Fall seven times, stand up eight. Debug, fix, repeat.",
    "Every line of code you don't write is a line of code you don't have to debug.",
    "The joy of coding is the joy of creating something from nothing.",
    "Code is not just for computers. It's for the people who will read it later.",
    "The sign of a great programmer is knowing what to build and what to leave out.",
];

fn random_quote() -> &'static str {
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..WELCOME_QUOTES.len());
    WELCOME_QUOTES[idx]
}

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

#[derive(Clone)]
struct AttachedFile {
    name: String,
    content: Vec<u8>,
    is_image: bool,
}

pub fn build_chat_view(chat_store: Rc<RefCell<ChatStore>>, logger: Rc<RefCell<crate::logger::Logger>>, on_chat_update: Box<dyn Fn()>, on_title_change: Box<dyn Fn(String)>, on_reset_title: Box<dyn Fn()>) -> (GtkBox, GtkBox, ChatClearHandle) {
    let pending_files: Rc<RefCell<Vec<AttachedFile>>> = Rc::new(RefCell::new(Vec::new()));
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);

    let topbar = GtkBox::new(Orientation::Horizontal, 0);
    topbar.style_context().add_class("topbar");
    topbar.set_margin_start(24);
    topbar.set_margin_end(24);
    topbar.set_margin_top(14);
    topbar.set_margin_bottom(14);

    let chat_title = Label::new(Some("Vibi AI"));
    chat_title.style_context().add_class("topbar-title");
    topbar.pack_start(&chat_title, false, false, 0);

    let divider = Separator::new(Orientation::Horizontal);
    divider.style_context().add_class("topbar-divider");

    root.pack_start(&topbar, false, false, 0);
    root.pack_start(&divider, false, false, 0);

    let scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroll.set_vexpand(true);
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);

    let message_area = GtkBox::new(Orientation::Vertical, 0);
    message_area.set_vexpand(true);
    message_area.set_valign(Align::Fill);

    let welcome = GtkBox::new(Orientation::Vertical, 16);
    welcome.set_valign(Align::Center);
    welcome.set_halign(Align::Center);
    welcome.set_vexpand(true);

    let icon_wrapper = GtkBox::new(Orientation::Horizontal, 0);
    icon_wrapper.set_halign(Align::Center);
    let icon = GtkBox::new(Orientation::Horizontal, 0);
    icon.style_context().add_class("welcome-icon");
    icon.set_size_request(56, 56);
    icon.set_halign(Align::Center);
    icon_wrapper.pack_start(&icon, false, false, 0);
    welcome.pack_start(&icon_wrapper, false, false, 0);

    let title = Label::new(Some("Welcome to Vibi AI"));
    title.style_context().add_class("welcome-title");
    welcome.pack_start(&title, false, false, 0);

    let subtitle = Label::new(Some(random_quote()));
    subtitle.style_context().add_class("welcome-subtitle");
    welcome.pack_start(&subtitle, false, false, 0);

    message_area.pack_start(&welcome, true, true, 0);
    let message_area_weak = message_area.clone();
    scroll.add(&message_area);
    root.pack_start(&scroll, true, true, 0);

    let preview_panel = GtkBox::new(Orientation::Vertical, 0);
    preview_panel.style_context().add_class("preview-panel");
    preview_panel.set_width_request(0);
    preview_panel.set_visible(false);

    let preview_header = GtkBox::new(Orientation::Horizontal, 0);
    preview_header.style_context().add_class("preview-header");
    preview_header.set_margin_start(16);
    preview_header.set_margin_end(8);
    preview_header.set_margin_top(12);
    preview_header.set_margin_bottom(8);

    let preview_title = Label::new(Some("File Preview"));
    preview_title.style_context().add_class("preview-title");
    preview_title.set_hexpand(true);
    preview_title.set_halign(Align::Start);
    preview_header.pack_start(&preview_title, true, true, 0);

    let close_preview = Button::with_label("✕");
    close_preview.style_context().add_class("preview-close");
    preview_header.pack_start(&close_preview, false, false, 0);

    preview_panel.pack_start(&preview_header, false, false, 0);

    let preview_divider = Separator::new(Orientation::Horizontal);
    preview_divider.style_context().add_class("preview-divider");
    preview_panel.pack_start(&preview_divider, false, false, 0);

    let preview_scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    preview_scroll.set_vexpand(true);
    preview_scroll.set_policy(PolicyType::Never, PolicyType::Automatic);

    let preview_content = GtkBox::new(Orientation::Vertical, 8);
    preview_content.set_margin_start(16);
    preview_content.set_margin_end(16);
    preview_content.set_margin_top(12);
    preview_content.set_margin_bottom(12);

    let preview_name = Label::new(None);
    preview_name.style_context().add_class("preview-filename");
    preview_name.set_wrap(true);
    preview_content.pack_start(&preview_name, false, false, 0);

    let preview_text = Label::new(None);
    preview_text.style_context().add_class("preview-text");
    preview_text.set_wrap(true);
    preview_text.set_selectable(true);
    preview_content.pack_start(&preview_text, true, true, 0);

    preview_scroll.add(&preview_content);
    preview_panel.pack_start(&preview_scroll, true, true, 0);

    close_preview.connect_clicked({
        let panel = preview_panel.clone();
        move |_| animate_preview(&panel, false)
    });

    let input_area = GtkBox::new(Orientation::Vertical, 8);
    input_area.set_margin_start(24);
    input_area.set_margin_end(24);
    input_area.set_margin_bottom(24);
    input_area.set_margin_top(16);

    let model_selector = crate::ui::model_selector::build_model_selector();
    input_area.pack_start(&model_selector.container, false, false, 0);

    let pill_container = GtkBox::new(Orientation::Horizontal, 4);
    pill_container.style_context().add_class("pill-container");
    pill_container.set_visible(false);
    pill_container.set_margin_bottom(6);
    input_area.pack_start(&pill_container, false, false, 0);

    let command_registry = Rc::new(CommandRegistry::new());
    let command_popover = Rc::new(CommandSuggestionPopover::new());
    command_popover.container.hide();
    input_area.pack_start(&command_popover.container, false, false, 0);

    let input_box = GtkBox::new(Orientation::Horizontal, 8);
    input_box.style_context().add_class("input-box");

    let attach_btn = Button::with_label("+");
    attach_btn.style_context().add_class("attach-btn");
    input_box.pack_start(&attach_btn, false, false, 0);

    let entry = Entry::new();
    entry.set_placeholder_text(Some("Type your message... (/ for commands)"));
    entry.style_context().add_class("chat-entry");
    entry.set_hexpand(true);
    input_box.pack_start(&entry, true, true, 0);

    let registry_clone = command_registry.clone();
    let popover_clone = command_popover.clone();
    entry.connect_changed(move |e| {
        let text = e.text().to_string();
        popover_clone.show_suggestions(&registry_clone, &text, e);
    });

    let send_btn = Button::with_label("➤");
    send_btn.style_context().add_class("send-btn");
    input_box.pack_start(&send_btn, false, false, 0);

    attach_btn.connect_clicked({
        let pending = pending_files.clone();
        let pill_container = pill_container.clone();
        move |_| {
            let dialog = FileChooserDialog::new::<gtk::Window>(Some("Attach Files"), None, gtk::FileChooserAction::Open);
            dialog.set_select_multiple(true);
            let pending = pending.clone();
            let pill_container = pill_container.clone();
            dialog.connect_response(move |d, response| {
                if response == gtk::ResponseType::Accept {
                    let files = d.files();
                    let mut new_files: Vec<AttachedFile> = Vec::new();
                    for f in &files {
                        let name = f.basename().unwrap().to_string_lossy().to_string();
                        if let Some(path) = f.path() {
                            let is_image = name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg") || name.ends_with(".gif") || name.ends_with(".webp");
                            let content = std::fs::read(&path).unwrap_or_default();
                            new_files.push(AttachedFile { name, content, is_image });
                        }
                    }
                    pending.borrow_mut().append(&mut new_files);
                    refresh_pills(&pill_container, &pending);
                }
                d.close();
            });
            dialog.show_all();
        }
    });

    entry.connect_activate({
        let send_btn = send_btn.clone();
        move |_| { send_btn.emit_clicked(); }
    });

    let entry_clone = entry.clone();
    let pending_send = pending_files.clone();
    let scroll_send = scroll.clone();
    let pill_container_send = pill_container.clone();
    let welcome_widget: Rc<RefCell<Option<GtkBox>>> = Rc::new(RefCell::new(None));
    let welcome_widget_clear = welcome_widget.clone();
    let welcome_for_remove = welcome.clone();
    let chat_title_send = chat_title.clone();
    let chat_store_send = chat_store.clone();
    let ai_bridge: Rc<RefCell<Option<crate::ai_bridge::AiBridge>>> = Rc::new(RefCell::new(None));
    let ai_bridge_send = ai_bridge.clone();
    let model_sel = model_selector.selected_model.clone();
    let root_for_bridge = root.clone();
    let chat_store_for_response = chat_store.clone();
    let message_area_for_response = message_area.clone();

    send_btn.connect_clicked(move |_| {
        let chat_title = chat_title_send.clone();
        let chat_store = chat_store_send.clone();
        let text = entry_clone.text().to_string();
        let files = pending_send.borrow_mut().drain(..).collect::<Vec<_>>();
        if text.trim().is_empty() && files.is_empty() { return; }

        if welcome_widget.borrow().is_none() {
            message_area.remove(&welcome_for_remove);
            *welcome_widget.borrow_mut() = Some(welcome_for_remove.clone());
        }

        let mut store = chat_store.borrow_mut();
        let is_new = store.get_active().is_none() && !text.trim().is_empty();
        if is_new {
            let model = model_sel.borrow().clone();
            store.create_chat(&text, &model);
            store.save();
            
            let bridge = crate::ai_bridge::AiBridge::new(&model);
            let message_area_clone = message_area.clone();
            let chat_store_clone = chat_store_for_response.clone();
            let logger_clone = logger.clone();
            bridge.on_response(move |response: String| {
                let mut store = chat_store_clone.borrow_mut();
                store.add_message_to_active("assistant", &response);
                store.save();
                logger_clone.borrow_mut().log(crate::logger::LogLevel::Info, "AI", &format!("Response: {}", &response[..response.len().min(50)]));
                
                let msg_area = message_area_clone.clone();
                let resp = response.clone();
                gtk::glib::idle_add_local(move || {
                    let msg_row = GtkBox::new(Orientation::Horizontal, 0);
                    msg_row.set_halign(Align::Start);
                    msg_row.set_margin_top(4);
                    msg_row.set_margin_bottom(4);
                    msg_row.set_margin_start(24);
                    
                    let bubble = Label::new(Some(&resp));
                    bubble.set_wrap(true);
                    bubble.set_max_width_chars(60);
                    bubble.style_context().add_class("ai-bubble");
                    msg_row.pack_start(&bubble, true, true, 0);
                    msg_area.pack_start(&msg_row, false, false, 0);
                    msg_row.show_all();
                    gtk::glib::ControlFlow::Break
                });
            });
            root_for_bridge.pack_start(&bridge.webview, false, false, 0);
            bridge.webview.show_all();
            *ai_bridge_send.borrow_mut() = Some(bridge);
            
            logger.borrow_mut().log(crate::logger::LogLevel::Info, "Chat", &format!("New chat created: {}", &text[..text.len().min(30)]));
        }
        if !text.trim().is_empty() {
            store.add_message_to_active("user", &text);
            logger.borrow_mut().log(crate::logger::LogLevel::Info, "Message", &format!("Sent: {}", &text[..text.len().min(50)]));
            let title = store.get_active().map(|c| c.title.clone());
            if let Some(t) = title {
                chat_title.set_text(&t);
                on_title_change(format!("Open: {}", t));
            }
        }
        drop(store);

        if is_new {
            on_chat_update();
        }

        for f in &files {
            logger.borrow_mut().log(crate::logger::LogLevel::Info, "File", &format!("Attached: {}", f.name));
        }

        if !text.trim().is_empty() {
            let msg_row = GtkBox::new(Orientation::Horizontal, 0);
            msg_row.set_halign(Align::End);
            msg_row.set_margin_top(4);
            msg_row.set_margin_bottom(4);
            msg_row.set_margin_end(24);

            let bubble = Label::new(Some(&text));
            bubble.set_wrap(true);
            bubble.set_max_width_chars(60);
            bubble.style_context().add_class("user-bubble");
            msg_row.pack_start(&bubble, true, true, 0);
            message_area.pack_start(&msg_row, false, false, 0);
            msg_row.show_all();
            
            if let Some(ref bridge) = *ai_bridge_send.borrow() {
                bridge.send_message(&text);
            } else {
                println!("[Chat] No AiBridge available, message not sent to AI");
            }
        }
        message_area.show_all();

        entry_clone.set_text("");
        pill_container_send.set_visible(false);
        let children = pill_container_send.children();
        for child in &children {
            pill_container_send.remove(child);
        }

        let scroll_clone = scroll_send.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            let adj = scroll_clone.vadjustment();
            adj.set_value(adj.upper());
            glib::ControlFlow::Break
        });
    });

    input_area.pack_start(&input_box, false, false, 0);

    let hint = Label::new(Some("Press Shift+Enter for new line"));
    hint.style_context().add_class("input-hint");
    hint.set_halign(Align::Center);
    input_area.pack_start(&hint, false, false, 0);

    root.pack_start(&input_area, false, false, 0);

    let clear_handle = ChatClearHandle::new();
    let messages_clear = Rc::new(RefCell::new(message_area_weak.clone()));
    let welcome_clear = welcome.clone();
    let quote_subtitle = subtitle.clone();
    let store_for_clear = chat_store.clone();
    let chat_title_for_clear = chat_title.clone();
    clear_handle.set(Box::new(move || {
        let store = store_for_clear.borrow();
        let mut msgs = messages_clear.borrow_mut();
        let children = msgs.children();
        for child in &children {
            msgs.remove(child);
        }
        if let Some(active) = store.get_active() {
            chat_title_for_clear.set_text(&active.title);
            for msg in &active.messages {
                let msg_row = GtkBox::new(Orientation::Horizontal, 0);
                msg_row.set_halign(if msg.role == "user" { Align::End } else { Align::Start });
                msg_row.set_margin_top(4);
                msg_row.set_margin_bottom(4);
                msg_row.set_margin_end(24);

                let bubble = Label::new(Some(&msg.content));
                bubble.set_wrap(true);
                bubble.set_max_width_chars(60);
                bubble.style_context().add_class(if msg.role == "user" { "user-bubble" } else { "ai-bubble" });
                msg_row.pack_start(&bubble, false, false, 0);
                msgs.pack_start(&msg_row, false, false, 0);
            }
            msgs.show_all();
        } else {
            quote_subtitle.set_text(random_quote());
            msgs.pack_start(&welcome_clear.clone(), true, true, 0);
            chat_title_for_clear.set_text("Vibi AI");
            msgs.show_all();
            *welcome_widget_clear.borrow_mut() = None;
        }
    }));

    (root, preview_panel, clear_handle)
}

fn animate_preview(panel: &GtkBox, show: bool) {
    let start_width = if show { 0 } else { 340 };
    let end_width = if show { 340 } else { 0 };
    let panel_clone = panel.clone();
    let duration_ms = 250;
    let steps = 25;
    let step_ms = duration_ms / steps;

    panel_clone.set_visible(true);

    let mut count = 0;
    glib::timeout_add_local(std::time::Duration::from_millis(step_ms as u64), move || {
        count += 1;
        let progress = count as f64 / steps as f64;
        let eased = 1.0 - (1.0 - progress).powi(3);
        let current_width = start_width as f64 + (end_width as f64 - start_width as f64) * eased;
        panel_clone.set_width_request(current_width as i32);
        if count >= steps {
            panel_clone.set_width_request(end_width);
            if !show { panel_clone.set_visible(false); }
            return glib::ControlFlow::Break;
        }
        glib::ControlFlow::Continue
    });
}

fn refresh_pills(container: &GtkBox, pending: &Rc<RefCell<Vec<AttachedFile>>>) {
    let children = container.children();
    for child in &children { container.remove(child); }
    let files = pending.borrow();
    for (i, f) in files.iter().enumerate() {
        let pill = GtkBox::new(Orientation::Horizontal, 6);
        pill.style_context().add_class("attachment-pill");

        let icon = if f.is_image { "🖼" } else { "📎" };
        let name_label = Label::new(Some(&format!("{} {}", icon, f.name)));
        name_label.style_context().add_class("pill-label");
        pill.pack_start(&name_label, false, false, 0);

        let remove_btn = Button::with_label("✕");
        remove_btn.style_context().add_class("pill-remove");
        let pending_clone = pending.clone();
        let container_clone = container.clone();
        let idx = i;
        remove_btn.connect_clicked(move |_| {
            let mut files = pending_clone.borrow_mut();
            if idx < files.len() { files.remove(idx); }
            if files.is_empty() { container_clone.set_visible(false); }
            refresh_pills(&container_clone, &pending_clone);
        });
        pill.pack_start(&remove_btn, false, false, 0);
        container.pack_start(&pill, false, false, 0);
    }
    container.set_visible(!files.is_empty());
}