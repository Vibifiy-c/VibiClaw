use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, Label, Orientation, Align, Revealer, Separator, ScrolledWindow, PolicyType};

pub struct ApprovalPanel {
    pub container: GtkBox,
    pub revealer: Revealer,
    pub card_list: GtkBox,
}

impl ApprovalPanel {
    pub fn new() -> Self {
        let revealer = Revealer::new();
        revealer.set_transition_type(gtk::RevealerTransitionType::SlideLeft);
        revealer.set_transition_duration(300);
        revealer.set_reveal_child(false);

        let container = GtkBox::new(Orientation::Vertical, 0);
        container.set_size_request(340, -1);
        container.style_context().add_class("right-panel");
        container.set_vexpand(true);

        // Header
        let header = GtkBox::new(Orientation::Vertical, 0);
        header.style_context().add_class("right-panel-header");
        header.set_margin_start(16);
        header.set_margin_end(16);
        header.set_margin_top(12);
        header.set_margin_bottom(8);

        let title_row = GtkBox::new(Orientation::Horizontal, 8);
        let title = Label::new(Some("🛡️ Approval Queue"));
        title.style_context().add_class("right-panel-title");
        title.set_halign(Align::Start);
        title.set_hexpand(true);
        title_row.pack_start(&title, true, true, 0);

        let close_btn = Button::with_label("✕");
        close_btn.style_context().add_class("right-panel-close-btn");
        title_row.pack_start(&close_btn, false, false, 0);
        header.pack_start(&title_row, false, false, 0);

        // Batch buttons
        let batch_row = GtkBox::new(Orientation::Horizontal, 8);
        batch_row.set_margin_top(8);

        let approve_all = Button::with_label("✅ Approve All");
        approve_all.style_context().add_class("right-panel-approve-all");
        approve_all.set_hexpand(true);
        batch_row.pack_start(&approve_all, true, true, 0);

        let deny_all = Button::with_label("❌ Deny All");
        deny_all.style_context().add_class("right-panel-deny-all");
        deny_all.set_hexpand(true);
        batch_row.pack_start(&deny_all, true, true, 0);

        header.pack_start(&batch_row, false, false, 0);
        container.pack_start(&header, false, false, 0);
        container.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);

        // Scrollable card list
        let scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        scroll.set_vexpand(true);
        scroll.set_policy(PolicyType::Never, PolicyType::Automatic);

        let card_list = GtkBox::new(Orientation::Vertical, 4);
        card_list.set_vexpand(true);
        card_list.set_valign(Align::Start);
        scroll.add(&card_list);
        container.pack_start(&scroll, true, true, 0);

        revealer.add(&container);

        // Close button wiring
        let rev = revealer.clone();
        close_btn.connect_clicked(move |_| {
            rev.set_reveal_child(false);
        });

        RightPanel {
            container,
            revealer,
            card_list,
        }
    }

    pub fn toggle(&self) {
        let revealed = self.revealer.reveals_child();
        self.revealer.set_reveal_child(!revealed);
    }

    pub fn show(&self) {
        self.revealer.set_reveal_child(true);
    }

    pub fn hide(&self) {
        self.revealer.set_reveal_child(false);
    }

    pub fn add_card(&self, tool: &str, path: &str, detail: &str) {
        let card = build_approval_card(tool, path, detail);
        self.card_list.pack_start(&card, false, false, 0);
        card.show_all();
        self.show();
    }

    pub fn clear(&self) {
        let children = self.card_list.children();
        for child in &children {
            self.card_list.remove(child);
        }
    }
}

fn build_approval_card(tool: &str, path: &str, detail: &str) -> GtkBox {
    let card = GtkBox::new(Orientation::Vertical, 6);
    card.style_context().add_class("approval-card");
    card.set_margin_start(12);
    card.set_margin_end(12);
    card.set_margin_top(6);
    card.set_margin_bottom(6);

    let info = GtkBox::new(Orientation::Vertical, 2);

    let tool_label = Label::new(Some(tool));
    tool_label.style_context().add_class("approval-tool");
    tool_label.set_halign(Align::Start);
    info.pack_start(&tool_label, false, false, 0);

    let path_label = Label::new(Some(path));
    path_label.style_context().add_class("approval-path");
    path_label.set_halign(Align::Start);
    path_label.set_ellipsize(pango::EllipsizeMode::Middle);
    info.pack_start(&path_label, false, false, 0);

    if !detail.is_empty() {
        let detail_label = Label::new(Some(detail));
        detail_label.style_context().add_class("approval-detail");
        detail_label.set_halign(Align::Start);
        detail_label.set_wrap(true);
        detail_label.set_max_width_chars(35);
        info.pack_start(&detail_label, false, false, 0);
    }

    card.pack_start(&info, true, true, 0);

    let buttons = GtkBox::new(Orientation::Horizontal, 8);
    buttons.set_margin_top(4);

    let approve = Button::with_label("✅");
    approve.style_context().add_class("approval-approve-btn");
    approve.set_size_request(44, 32);
    buttons.pack_start(&approve, false, false, 0);

    let deny = Button::with_label("❌");
    deny.style_context().add_class("approval-deny-btn");
    deny.set_size_request(44, 32);
    buttons.pack_start(&deny, false, false, 0);

    card.pack_start(&buttons, false, false, 0);

    card
}