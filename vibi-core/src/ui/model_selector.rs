use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, Label, Orientation, Align, Revealer, ScrolledWindow, PolicyType};
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub struct ModelSelector {
    pub container: GtkBox,
    pub revealer: Revealer,
    pub selected_model: Rc<RefCell<String>>,
    pub selected_label: Rc<RefCell<Label>>,
    pub selected_icon: Rc<RefCell<Label>>,
}

pub fn build_model_selector() -> ModelSelector {
    let selected_model = Rc::new(RefCell::new("chatgpt".to_string()));
    
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.set_margin_bottom(8);
    
    let trigger = Button::new();
    trigger.style_context().add_class("model-trigger");
    trigger.set_halign(Align::Start);
    
    let trigger_content = GtkBox::new(Orientation::Horizontal, 8);
    let icon = Label::new(Some("🟢"));
    icon.style_context().add_class("model-trigger-icon");
    
    let selected_label = Rc::new(RefCell::new(Label::new(Some("ChatGPT"))));
    let selected_icon = Rc::new(RefCell::new(icon.clone()));
    trigger_content.pack_start(&icon, false, false, 0);
    trigger_content.pack_start(&selected_label.borrow().clone(), false, false, 0);
    let arrow = Label::new(Some("▾"));
    arrow.style_context().add_class("model-trigger-arrow");
    trigger_content.pack_start(&arrow, false, false, 0);
    trigger.add(&trigger_content);
    container.pack_start(&trigger, false, false, 0);
    
    let revealer = Revealer::new();
    revealer.set_transition_duration(200);
    revealer.set_transition_type(gtk::RevealerTransitionType::SlideDown);
    revealer.set_reveal_child(false);
    
    let panel = GtkBox::new(Orientation::Vertical, 0);
    panel.style_context().add_class("model-panel");
    panel.set_margin_top(4);
    
    let scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
    scroll.set_max_content_height(400);
    
    let list = GtkBox::new(Orientation::Vertical, 0);
    
    let models = vec![
        ("chatgpt", "🟢", "ChatGPT", "GPT-4o, GPT-4, GPT-3.5"),
        ("claude", "🟠", "Claude", "Claude 3.5 Sonnet, Opus"),
        ("gemini", "🔵", "Gemini", "Gemini 1.5 Pro, Flash"),
        ("deepseek", "🐋", "DeepSeek", "DeepSeek-V3, R1"),
        ("grok", "⚡", "Grok", "Grok-2"),
        ("qwen", "🟣", "Qwen", "Qwen 2.5"),
        ("kimi", "🌙", "Kimi", "Kimi K2"),
    ];
    
    for (id, emoji, name, desc) in &models {
        let row = build_model_row(id, emoji, name, desc);
        let model_id = id.to_string();
        let model_name = name.to_string();
        let model_emoji = emoji.to_string();
        let sel = selected_model.clone();
        let lbl = selected_label.clone();
        let sicon = selected_icon.clone();
        let rev = revealer.clone();
        row.connect_button_press_event(move |_, _| {
            *sel.borrow_mut() = model_id.clone();
            lbl.borrow().set_text(&model_name);
            sicon.borrow().set_text(&model_emoji);
            rev.set_reveal_child(false);
            false.into()
        });
        list.pack_start(&row, false, false, 0);
    }
    
    scroll.add(&list);
    panel.pack_start(&scroll, false, false, 0);
    revealer.add(&panel);
    container.pack_start(&revealer, false, false, 0);
    
    let rev = revealer.clone();
    trigger.connect_clicked(move |_| {
        let current = rev.reveals_child();
        rev.set_reveal_child(!current);
    });
    
    ModelSelector {
        container,
        revealer,
        selected_model,
        selected_label,
        selected_icon,
    }
}

fn build_model_row(_id: &str, emoji: &str, name: &str, desc: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 10);
    row.style_context().add_class("model-row");
    row.set_margin_start(8);
    row.set_margin_end(8);
    row.set_margin_top(4);
    row.set_margin_bottom(4);
    
    let icon = Label::new(Some(emoji));
    icon.style_context().add_class("model-icon");
    row.pack_start(&icon, false, false, 0);
    
    let info = GtkBox::new(Orientation::Vertical, 2);
    let name_label = Label::new(Some(name));
    name_label.style_context().add_class("model-name");
    name_label.set_halign(Align::Start);
    info.pack_start(&name_label, false, false, 0);
    
    let desc_label = Label::new(Some(desc));
    desc_label.style_context().add_class("model-desc");
    desc_label.set_halign(Align::Start);
    info.pack_start(&desc_label, false, false, 0);
    
    row.pack_start(&info, true, true, 0);
    row
}