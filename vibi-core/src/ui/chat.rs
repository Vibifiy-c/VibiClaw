use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Entry, Label, Orientation, Align, ScrolledWindow, PolicyType, GestureClick};
use gtk4::glib;
use gtk4::FileChooserDialog;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
struct AttachedFile {
    name: String,
    content: Vec<u8>,
    is_image: bool,
}

pub fn build_chat_view() -> (GtkBox, GtkBox) {
    let pending_files: Rc<RefCell<Vec<AttachedFile>>> = Rc::new(RefCell::new(Vec::new()));
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);

    let topbar = GtkBox::new(Orientation::Horizontal, 0);
    topbar.add_css_class("topbar");
    topbar.set_margin_start(24);
    topbar.set_margin_end(24);
    topbar.set_margin_top(14);
    topbar.set_margin_bottom(14);

    let chat_title = Label::new(Some("Vibi AI"));
    chat_title.add_css_class("topbar-title");
    topbar.append(&chat_title);

    let divider = gtk4::Separator::new(Orientation::Horizontal);
    divider.add_css_class("topbar-divider");

    root.append(&topbar);
    root.append(&divider);

    let scroll = ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);

    let message_area = GtkBox::new(Orientation::Vertical, 0);
    message_area.set_vexpand(true);

    let welcome = GtkBox::new(Orientation::Vertical, 16);
    welcome.set_valign(Align::Center);
    welcome.set_halign(Align::Center);
    welcome.set_vexpand(true);

    let icon_wrapper = GtkBox::new(Orientation::Horizontal, 0);
    icon_wrapper.set_halign(Align::Center);
    let icon = GtkBox::new(Orientation::Horizontal, 0);
    icon.add_css_class("welcome-icon");
    icon.set_size_request(56, 56);
    icon.set_halign(Align::Center);
    icon_wrapper.append(&icon);
    welcome.append(&icon_wrapper);

    let title = Label::new(Some("Welcome to Vibi AI"));
    title.add_css_class("welcome-title");
    welcome.append(&title);

    let subtitle = Label::new(Some("Start a conversation or explore the available features."));
    subtitle.add_css_class("welcome-subtitle");
    welcome.append(&subtitle);

    message_area.append(&welcome);
    scroll.set_child(Some(&message_area));
    root.append(&scroll);

    let preview_panel = GtkBox::new(Orientation::Vertical, 0);
    preview_panel.add_css_class("preview-panel");
    preview_panel.set_width_request(340);
    preview_panel.set_visible(false);

    let preview_header = GtkBox::new(Orientation::Horizontal, 0);
    preview_header.add_css_class("preview-header");
    preview_header.set_margin_start(16);
    preview_header.set_margin_end(8);
    preview_header.set_margin_top(12);
    preview_header.set_margin_bottom(8);

    let preview_title = Label::new(Some("File Preview"));
    preview_title.add_css_class("preview-title");
    preview_title.set_hexpand(true);
    preview_title.set_halign(Align::Start);
    preview_header.append(&preview_title);

    let close_preview = Button::with_label("✕");
    close_preview.add_css_class("preview-close");
    preview_header.append(&close_preview);

    preview_panel.append(&preview_header);

    let preview_divider = gtk4::Separator::new(Orientation::Horizontal);
    preview_divider.add_css_class("preview-divider");
    preview_panel.append(&preview_divider);

    let preview_scroll = ScrolledWindow::new();
    preview_scroll.set_vexpand(true);
    preview_scroll.set_policy(PolicyType::Never, PolicyType::Automatic);

    let preview_content = GtkBox::new(Orientation::Vertical, 8);
    preview_content.set_margin_start(16);
    preview_content.set_margin_end(16);
    preview_content.set_margin_top(12);
    preview_content.set_margin_bottom(12);

    let preview_name = Label::new(None);
    preview_name.add_css_class("preview-filename");
    preview_name.set_wrap(true);
    preview_content.append(&preview_name);

    let preview_text = Label::new(None);
    preview_text.add_css_class("preview-text");
    preview_text.set_wrap(true);
    preview_text.set_selectable(true);
    preview_content.append(&preview_text);

    preview_scroll.set_child(Some(&preview_content));
    preview_panel.append(&preview_scroll);

    root.append(&scroll);

    close_preview.connect_clicked({
        let panel = preview_panel.clone();
        move |_| panel.set_visible(false)
    });

    let input_area = GtkBox::new(Orientation::Vertical, 8);
    input_area.set_margin_start(24);
    input_area.set_margin_end(24);
    input_area.set_margin_bottom(24);
    input_area.set_margin_top(16);

    let pill_container = GtkBox::new(Orientation::Horizontal, 4);
    pill_container.add_css_class("pill-container");
    pill_container.set_visible(false);
    pill_container.set_margin_bottom(6);
    input_area.append(&pill_container);

    let pill_label = Rc::new(RefCell::new(pill_container.clone()));

    let input_box = GtkBox::new(Orientation::Horizontal, 8);
    input_box.add_css_class("input-box");

    let attach_btn = Button::with_label("+");
    attach_btn.add_css_class("attach-btn");
    input_box.append(&attach_btn);

    let entry = Entry::new();
    entry.set_placeholder_text(Some("Type your message..."));
    entry.add_css_class("chat-entry");
    entry.set_hexpand(true);
    input_box.append(&entry);

    let send_btn = Button::with_label("➤");
    send_btn.add_css_class("send-btn");
    input_box.append(&send_btn);
    let messages_clone = Rc::new(RefCell::new(message_area.clone()));
    let welcome_clone = welcome.clone();

    attach_btn.connect_clicked({
        let pending = pending_files.clone();
        let pill_container = pill_container.clone();
        move |_| {
            let dialog = FileChooserDialog::new(
                Some("Attach Files"),
                gtk4::Window::NONE,
                gtk4::FileChooserAction::Open,
                &[("Cancel", gtk4::ResponseType::Cancel), ("Open", gtk4::ResponseType::Accept)],
            );
            dialog.set_select_multiple(true);
            let pending = pending.clone();
            let pill_container = pill_container.clone();
            dialog.connect_response(move |d, response| {
                if response == gtk4::ResponseType::Accept {
                    let list = d.files();
                    let mut new_files: Vec<AttachedFile> = Vec::new();
                    for i in 0..list.n_items() {
                        if let Some(f) = list.item(i).and_then(|obj| obj.downcast::<gtk4::gio::File>().ok()) {
                            let name = f.basename().unwrap_or_default().to_string_lossy().to_string();
                            if let Some(path) = f.path() {
                                let is_image = name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg") || name.ends_with(".gif") || name.ends_with(".webp");
                                let content = std::fs::read(&path).unwrap_or_default();
                                new_files.push(AttachedFile { name, content, is_image });
                            }
                        }
                    }
                    pending.borrow_mut().append(&mut new_files);
                    refresh_pills(&pill_container, &pending);
                }
                d.close();
            });
            dialog.show();
        }
    });

    entry.connect_activate({
        let send_btn = send_btn.clone();
        move |_| { send_btn.emit_clicked(); }
    });

    let entry_clone = entry.clone();
    let pending_send = pending_files.clone();
    let scroll_send = scroll.clone();
    let preview_panel_send = preview_panel.clone();
    let preview_name_send = preview_name.clone();
    let preview_text_send = preview_text.clone();
    let pill_container_send = pill_container.clone();

    send_btn.connect_clicked(move |_| {
        let text = entry_clone.text().to_string();
        let files = pending_send.borrow_mut().drain(..).collect::<Vec<_>>();
        if text.trim().is_empty() && files.is_empty() { return; }

        let mut msgs = messages_clone.borrow_mut();
        let first = msgs.first_child();
        let should_remove = first.as_ref().map(|w| w.clone().downcast::<GtkBox>().ok()) == Some(Some(welcome_clone.clone()));
        if should_remove {
            if let Some(w) = first {
                msgs.remove(&w);
            }
        }

        if text.trim().len() > 0 {
            let title_text = if text.len() > 30 { format!("{}...", &text[..30]) } else { text.clone() };
            chat_title.set_text(&title_text);
        }

        for f in &files {
            let artifact_row = GtkBox::new(Orientation::Horizontal, 0);
            artifact_row.set_halign(Align::End);
            artifact_row.set_margin_top(4);
            artifact_row.set_margin_end(24);

            let card = GtkBox::new(Orientation::Horizontal, 10);
            card.add_css_class("artifact-card");
            card.set_halign(Align::End);

            let icon = if f.is_image { "🖼" } else { "📎" };
            let icon_label = Label::new(Some(icon));
            icon_label.add_css_class("artifact-icon");
            card.append(&icon_label);

            let name_label = Label::new(Some(&f.name));
            name_label.add_css_class("artifact-filename");
            name_label.set_wrap(true);
            name_label.set_max_width_chars(40);
            card.append(&name_label);

            let card_click = card.clone();
            let f_name = f.name.clone();
            let f_content = if f.is_image { "[Image preview not available]".to_string() } else { String::from_utf8_lossy(&f.content).to_string() };
            let panel = preview_panel_send.clone();
            let p_name = preview_name_send.clone();
            let p_text = preview_text_send.clone();
            let gesture = gtk4::GestureClick::new();
            gesture.connect_released(move |_, _, _, _| {
                p_name.set_text(&f_name);
                p_text.set_text(&f_content);
                panel.set_visible(true);
            });
            card.add_controller(gesture);

            artifact_row.append(&card);
            msgs.append(&artifact_row);
        }

        let full_message = if text.is_empty() {
            String::new()
        } else {
            text.clone()
        };

        if !full_message.is_empty() {
            let msg_row = GtkBox::new(Orientation::Horizontal, 0);
            msg_row.set_halign(Align::End);
            msg_row.set_margin_top(4);
            msg_row.set_margin_bottom(4);
            msg_row.set_margin_end(24);

            let bubble = Label::new(Some(&full_message));
            bubble.set_wrap(true);
            bubble.set_max_width_chars(60);
            bubble.add_css_class("user-bubble");
            msg_row.append(&bubble);
            msgs.append(&msg_row);
        }
        drop(msgs);

        entry_clone.set_text("");
        pill_container_send.set_visible(false);
        while let Some(child) = pill_container_send.first_child() {
            pill_container_send.remove(&child);
        }

        let scroll_clone = scroll_send.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            let adj = scroll_clone.vadjustment();
            adj.set_value(adj.upper());
            glib::ControlFlow::Break
        });
    });

    input_area.append(&input_box);

    let hint = Label::new(Some("Press Shift+Enter for new line"));
    hint.add_css_class("input-hint");
    hint.set_halign(Align::Center);
    input_area.append(&hint);

    root.append(&input_area);

    (root, preview_panel)
}

fn refresh_pills(container: &GtkBox, pending: &Rc<RefCell<Vec<AttachedFile>>>) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
    let files = pending.borrow();
    for (i, f) in files.iter().enumerate() {
        let pill = GtkBox::new(Orientation::Horizontal, 6);
        pill.add_css_class("attachment-pill");

        let icon = if f.is_image { "🖼" } else { "📎" };
        let name_label = Label::new(Some(&format!("{} {}", icon, f.name)));
        name_label.add_css_class("pill-label");
        pill.append(&name_label);

        let remove_btn = Button::with_label("✕");
        remove_btn.add_css_class("pill-remove");
        let pending_clone = pending.clone();
        let container_clone = container.clone();
        let idx = i;
        remove_btn.connect_clicked(move |_| {
            let mut files = pending_clone.borrow_mut();
            if idx < files.len() {
                files.remove(idx);
            }
            if files.is_empty() {
                container_clone.set_visible(false);
            }
            refresh_pills(&container_clone, &pending_clone);
        });
        pill.append(&remove_btn);
        container.append(&pill);
    }
    container.set_visible(!files.is_empty());
}

