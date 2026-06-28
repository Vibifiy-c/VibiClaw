use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, Orientation, Align, ScrolledWindow, PolicyType, FlowBox, Entry, Button, FileChooserDialog, Stack, Separator};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub category: String,
    pub files: HashMap<String, String>,
    pub linked_chat: Option<String>,
    pub github_link: Option<String>,
}

pub fn build_agentic_view(projects: Rc<RefCell<Vec<Project>>>) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);
    root.style_context().add_class("agentic-root");

    let stack = Stack::new();
    stack.set_hexpand(true);
    stack.set_vexpand(true);

    let categories_page = build_categories_page(projects.clone(), stack.clone());
    stack.add_titled(&categories_page, "categories", "Categories");

    let dialog_page = build_dialog_page(projects, stack.clone());
    stack.add_titled(&dialog_page, "dialog", "Dialog");

    root.pack_start(&stack, true, true, 0);
    root
}

fn build_categories_page(projects: Rc<RefCell<Vec<Project>>>, stack: Stack) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);

    let topbar = GtkBox::new(Orientation::Horizontal, 0);
    topbar.style_context().add_class("topbar");
    topbar.set_margin_start(24);
    topbar.set_margin_end(24);
    topbar.set_margin_top(14);
    topbar.set_margin_bottom(14);

    let title = Label::new(Some("Agentic Tool"));
    title.style_context().add_class("topbar-title");
    topbar.pack_start(&title, false, false, 0);

    let divider = Separator::new(Orientation::Horizontal);
    divider.style_context().add_class("topbar-divider");

    root.pack_start(&topbar, false, false, 0);
    root.pack_start(&divider, false, false, 0);

    let scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroll.set_vexpand(true);
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);

    let content = GtkBox::new(Orientation::Vertical, 16);
    content.set_margin_start(24);
    content.set_margin_end(24);
    content.set_margin_top(20);
    content.set_margin_bottom(20);

    let heading = Label::new(Some("Choose a Category"));
    heading.style_context().add_class("agentic-heading");
    heading.set_halign(Align::Start);
    content.pack_start(&heading, false, false, 0);

    let proj_list = projects.borrow().clone();
    if !proj_list.is_empty() {
        let proj_section = build_projects_section(&proj_list);
        content.pack_start(&proj_section, false, false, 0);
    }

    let grid = GtkBox::new(Orientation::Vertical, 12);
    grid.style_context().add_class("agentic-grid");

    let stack_clone = stack.clone();
    grid.pack_start(&category_section("Coding", &[("💻", "Coding", "coding")], stack_clone.clone()), false, false, 0);

    grid.pack_start(&category_section("Design", &[("🎨", "Figma", "figma"), ("🖌", "Canva", "canva")], stack_clone.clone()), false, false, 0);

    grid.pack_start(&category_section("Microsoft Office", &[
        ("📊", "Excel", "excel"), ("📝", "Word", "word"), ("📽", "PowerPoint", "powerpoint"),
        ("📓", "OneNote", "onenote"), ("📐", "Visio", "visio"), ("🗄", "Access", "access"),
        ("📰", "Publisher", "publisher"), ("📱", "Sway", "sway"), ("📋", "Forms", "forms"),
        ("🎬", "Clipchamp", "clipchamp"), ("🌐", "SharePoint", "sharepoint"), ("☁", "OneDrive", "onedrive"),
    ], stack_clone), false, false, 0);

    content.pack_start(&grid, false, false, 0);
    scroll.add(&content);
    root.pack_start(&scroll, true, true, 0);
    root
}

fn build_projects_section(projects: &[Project]) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 8);
    let label = Label::new(Some("Your Projects"));
    label.style_context().add_class("agentic-section-title");
    label.set_halign(Align::Start);
    section.pack_start(&label, false, false, 0);

    for p in projects {
        let row = GtkBox::new(Orientation::Horizontal, 10);
        row.style_context().add_class("agentic-project-row");
        row.set_margin_top(4);
        row.set_margin_bottom(4);

        let icon = Label::new(Some("📁"));
        icon.style_context().add_class("agentic-project-icon");
        row.pack_start(&icon, false, false, 0);

        let info = GtkBox::new(Orientation::Vertical, 2);
        let name = Label::new(Some(&p.name));
        name.style_context().add_class("agentic-project-name");
        info.pack_start(&name, false, false, 0);

        let meta = Label::new(Some(&format!("{} files · {}", p.files.len(), p.category)));
        meta.style_context().add_class("agentic-project-meta");
        info.pack_start(&meta, false, false, 0);

        row.pack_start(&info, true, true, 0);
        section.pack_start(&row, false, false, 0);
    }
    section
}

fn category_section(title: &str, items: &[(&str, &str, &str)], stack: Stack) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 8);

    let label = Label::new(Some(title));
    label.style_context().add_class("agentic-section-title");
    label.set_halign(Align::Start);
    section.pack_start(&label, false, false, 0);

    let flow = FlowBox::new();
    flow.set_selection_mode(gtk::SelectionMode::None);
    flow.set_min_children_per_line(2);
    flow.set_max_children_per_line(10);
    flow.set_homogeneous(true);
    flow.set_row_spacing(12);
    flow.set_column_spacing(12);

    for (icon, name, _cat_key) in items {
        let card = category_card(icon, name);
        let s = stack.clone();
        card.connect_button_press_event(move |_, _| {
            s.set_visible_child_name("dialog");
            false.into()
        });
        flow.insert(&card, -1);
    }

    section.pack_start(&flow, false, false, 0);
    section
}

fn category_card(icon: &str, name: &str) -> GtkBox {
    let card = GtkBox::new(Orientation::Vertical, 12);
    card.style_context().add_class("agentic-card");
    card.set_size_request(180, 150);
    card.set_hexpand(true);

    let icon_box = GtkBox::new(Orientation::Horizontal, 0);
    icon_box.style_context().add_class("agentic-card-icon");
    icon_box.set_size_request(44, 44);
    icon_box.set_halign(Align::Start);

    let icon_label = Label::new(Some(icon));
    icon_label.style_context().add_class("agentic-card-icon-text");
    icon_box.pack_start(&icon_label, false, false, 0);
    card.pack_start(&icon_box, false, false, 0);

    let name_label = Label::new(Some(name));
    name_label.style_context().add_class("agentic-card-title");
    name_label.set_halign(Align::Start);
    card.pack_start(&name_label, false, false, 0);

    card
}

fn build_dialog_page(projects: Rc<RefCell<Vec<Project>>>, stack: Stack) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);
    root.set_margin_start(60);
    root.set_margin_end(60);
    root.set_margin_top(40);
    root.set_margin_bottom(40);

    let back_btn = Button::with_label("← Back");
    back_btn.style_context().add_class("agentic-back-btn");
    back_btn.set_halign(Align::Start);
    let s = stack.clone();
    back_btn.connect_clicked(move |_| s.set_visible_child_name("categories"));
    root.pack_start(&back_btn, false, false, 0);

    let title = Label::new(Some("Add New Project"));
    title.style_context().add_class("agentic-heading");
    title.set_halign(Align::Start);
    title.set_margin_top(16);
    root.pack_start(&title, false, false, 0);

    let form = GtkBox::new(Orientation::Vertical, 16);
    form.set_margin_top(20);

    let github_label = Label::new(Some("GitHub / GitLab Link"));
    github_label.style_context().add_class("agentic-section-title");
    github_label.set_halign(Align::Start);
    form.pack_start(&github_label, false, false, 0);

    let github_input = Entry::new();
    github_input.set_placeholder_text(Some("https://github.com/username/repo"));
    github_input.style_context().add_class("agentic-input");
    form.pack_start(&github_input, false, false, 0);

    let or_label = Label::new(Some("— OR —"));
    or_label.style_context().add_class("agentic-or");
    or_label.set_halign(Align::Center);
    form.pack_start(&or_label, false, false, 0);

    let upload_btn = Button::with_label("📁 Choose Folder from Device");
    upload_btn.style_context().add_class("agentic-upload-btn");
    form.pack_start(&upload_btn, false, false, 0);

    let file_status = Label::new(None);
    file_status.style_context().add_class("agentic-file-status");
    form.pack_start(&file_status, false, false, 0);

    let create_btn = Button::with_label("Create Project");
    create_btn.style_context().add_class("agentic-create-btn");
    create_btn.set_sensitive(false);
    form.pack_start(&create_btn, false, false, 0);

    let selected_files: Rc<RefCell<HashMap<String, String>>> = Rc::new(RefCell::new(HashMap::new()));
    let selected_name: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    let files_clone = selected_files.clone();
    let name_clone = selected_name.clone();
    let status_clone = file_status.clone();
    let create_clone = create_btn.clone();
    upload_btn.connect_clicked(move |_| {
        let dialog = FileChooserDialog::new::<gtk::Window>(Some("Choose Folder"), None, gtk::FileChooserAction::SelectFolder);
        let f = files_clone.clone();
        let n = name_clone.clone();
        let s = status_clone.clone();
        let c = create_clone.clone();
        dialog.connect_response(move |d, resp| {
            if resp == gtk::ResponseType::Accept {
                if let Some(file) = d.file() {
                    let folder_name = file.basename().unwrap().to_string_lossy().to_string();
                    let mut files_map = HashMap::new();
                    if let Ok(entries) = fs::read_dir(file.path().unwrap()) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file() {
                                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                    if let Ok(content) = fs::read_to_string(&path) {
                                        files_map.insert(name.to_string(), content);
                                    }
                                }
                            }
                        }
                    }
                    *f.borrow_mut() = files_map.clone();
                    *n.borrow_mut() = Some(folder_name.clone());
                    s.set_text(&format!("Selected: {} ({} files)", folder_name, files_map.len()));
                    c.set_sensitive(true);
                }
            }
            d.close();
        });
        dialog.show_all();
    });

    let projects_clone = projects.clone();
    let stack_clone = stack.clone();
    let name_clone = selected_name.clone();
    let files_clone = selected_files.clone();
    let github_clone = github_input.clone();
    create_btn.connect_clicked(move |_| {
        let name = name_clone.borrow().clone().unwrap_or_else(|| "Untitled Project".to_string());
        let github = github_clone.text().to_string();
        let github = if github.is_empty() { None } else { Some(github) };
        let files = files_clone.borrow().clone();
        let project = Project {
            id: format!("proj_{}", chrono::Utc::now().timestamp()),
            name,
            category: "coding".to_string(),
            files,
            linked_chat: None,
            github_link: github,
        };
        projects_clone.borrow_mut().push(project);
        stack_clone.set_visible_child_name("categories");
    });

    root.pack_start(&form, false, false, 0);
    root
}