use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, Orientation, Align, ScrolledWindow, PolicyType, FlowBox, Entry, Button, FileChooserDialog, Stack, Separator};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use gdk_pixbuf::Pixbuf;
use serde::{Deserialize, Serialize};

thread_local! {
    static EDITOR_REBUILD: RefCell<Option<Box<dyn Fn()>>> = RefCell::new(None);
}

use std::sync::LazyLock;
static FOLDER_STATE: LazyLock<std::sync::Mutex<HashMap<String, bool>>> = LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

fn is_folder_collapsed(path: &str) -> bool {
    let state = FOLDER_STATE.lock().unwrap();
    // Default: collapsed on first load
    *state.get(path).unwrap_or(&true)
}

fn toggle_folder_state(path: &str) -> bool {
    let mut state = FOLDER_STATE.lock().unwrap();
    let current = *state.get(path).unwrap_or(&true);
    let new = !current;
    state.insert(path.to_string(), new);
    new
}

pub fn trigger_editor_rebuild() {

    EDITOR_REBUILD.with(|r| {
        if let Some(ref rebuild) = *r.borrow() {
            rebuild();
        }
    });
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub uuid: String,
    pub name: String,
    pub category: String,
    pub directory: String,
    pub files: HashMap<String, String>,
    pub linked_chat: Option<String>,
    pub github_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftProject {
    pub name: String,
    pub directory: String,
    pub github_link: Option<String>,
    pub github_user: Option<String>,
    pub is_private: bool,
    pub has_token: bool,
    pub token: Option<String>,
}

pub fn build_agentic_view(projects: Rc<RefCell<Vec<Project>>>) -> GtkBox {
    let drafts: Rc<RefCell<Vec<DraftProject>>> = Rc::new(RefCell::new(Vec::new()));
    let selected_dir: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);
    root.style_context().add_class("agentic-root");

    let stack = Stack::new();
    stack.set_hexpand(true);
    stack.set_vexpand(true);

    let categories_page = build_categories_page(projects.clone(), drafts.clone(), selected_dir.clone(), stack.clone());
    stack.add_titled(&categories_page, "categories", "Categories");


    let create_page = build_create_project_page(projects.clone(), drafts.clone(), stack.clone());
    stack.add_titled(&create_page, "create", "Create Project");

    let editor_page = build_code_editor_page(stack.clone(), selected_dir.clone());
    stack.add_titled(&editor_page, "editor", "Editor");

    root.pack_start(&stack, true, true, 0);
    root
}

fn build_categories_page(projects: Rc<RefCell<Vec<Project>>>, drafts: Rc<RefCell<Vec<DraftProject>>>, selected_dir: Rc<RefCell<Option<String>>>, stack: Stack) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);

    let topbar = GtkBox::new(Orientation::Horizontal, 0);
    topbar.style_context().add_class("topbar");
    topbar.set_margin_start(24);
    topbar.set_margin_end(24);
    topbar.set_margin_top(14);
    topbar.set_margin_bottom(14);

    let title = Label::new(Some("Code Editor"));
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

    let heading = Label::new(Some("My Projects"));
    heading.style_context().add_class("agentic-heading");
    heading.set_halign(Align::Start);
    content.pack_start(&heading, false, false, 0);

    let projects_grid = gtk::FlowBox::new();
    projects_grid.set_selection_mode(gtk::SelectionMode::None);
    projects_grid.set_min_children_per_line(2);
    projects_grid.set_max_children_per_line(6);
    projects_grid.set_homogeneous(false);
    projects_grid.set_row_spacing(16);
    projects_grid.set_column_spacing(16);
    projects_grid.set_halign(Align::Start);
    projects_grid.set_valign(Align::Start);

    let grid_ref = Rc::new(RefCell::new(projects_grid.clone()));

    // Load projects from sandbox disk
    {
        let sandbox = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("vibi-ai")
            .join("sandbox");
        if sandbox.exists() {
            if let Ok(entries) = std::fs::read_dir(&sandbox) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let proj_file = path.join(".vibecode");
                        if proj_file.exists() {
                            if let Ok(data) = std::fs::read_to_string(&proj_file) {
                                let name = data.lines()
                                    .find(|l| l.starts_with("name="))
                                    .map(|l| l[5..].to_string())
                                    .unwrap_or_else(|| path.file_name().unwrap().to_string_lossy().to_string());
                                let uuid = data.lines()
                                    .find(|l| l.starts_with("uuid="))
                                    .map(|l| l[5..].to_string())
                                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                                let id = data.lines()
                                    .find(|l| l.starts_with("id="))
                                    .map(|l| l[3..].to_string())
                                    .unwrap_or_else(|| uuid.clone());
                                let mut proj_list = projects.borrow_mut();
                                let dir = path.file_name().unwrap().to_string_lossy().to_string();
                                if !proj_list.iter().any(|p| p.uuid == uuid) {
                                    proj_list.push(Project {
                                        id,
                                        uuid,
                                        name,
                                        category: "custom".to_string(),
                                        directory: dir,
                                        files: HashMap::new(),
                                        linked_chat: None,
                                        github_link: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Function to refresh the grid
    let refresh_grid = {
        let p = projects.clone();
        let d = drafts.clone();
        let g = grid_ref.clone();
        let s = stack.clone();
        let sel_dir = selected_dir.clone();
        move || {
            let grid = g.borrow();
            let children = grid.children();
            for child in &children {
                grid.remove(child);
            }
            // Add project cards
            for proj in p.borrow().iter() {
                let card = build_project_card(&proj.name, &proj.id, &proj.directory, s.clone(), sel_dir.clone(), grid_ref.clone());
                grid.insert(&card, -1);
            }
            // Add draft cards
            for (i, draft) in d.borrow().iter().enumerate() {
                let card = build_draft_card(&draft.name, i, d.clone());
                grid.insert(&card, -1);
            }
            // Add + card
            let add = build_add_project_card(p.clone(), grid.clone(), s.clone());
            grid.insert(&add, -1);
            grid.show_all();
        }
    };

    // Initial population
    refresh_grid();

    content.pack_start(&projects_grid, false, false, 0);

    // Store refresh function on the root widget so it can be called
    let refresh = Rc::new(RefCell::new(Some(refresh_grid)));
    root.connect_map(move |w| {
        // Only refresh if this page is actually visible
        if w.is_visible() {
            println!("[CodeEditor] Categories page mapped, refreshing grid");
            if let Some(ref f) = *refresh.borrow() {
                f();
            }
        }
    });

    scroll.add(&content);
    root.pack_start(&scroll, true, true, 0);
    root
}

fn build_project_card(name: &str, id: &str, directory: &str, stack: Stack, sel_dir: Rc<RefCell<Option<String>>>, grid_ref: Rc<RefCell<gtk::FlowBox>>) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.set_size_request(180, 150);

    // Main card button
    let card = Button::new();
    card.style_context().add_class("project-card");
    card.set_relief(gtk::ReliefStyle::None);
    card.set_hexpand(true);
    card.set_vexpand(true);

    let content = GtkBox::new(Orientation::Vertical, 12);
    content.set_halign(Align::Center);
    content.set_valign(Align::Center);

    let icon = Label::new(Some("📁"));
    icon.style_context().add_class("project-card-icon");
    icon.set_halign(Align::Center);
    content.pack_start(&icon, false, false, 0);

    let name_label = Label::new(Some(name));
    name_label.style_context().add_class("project-card-name");
    name_label.set_halign(Align::Center);
    name_label.set_ellipsize(pango::EllipsizeMode::End);
    name_label.set_max_width_chars(14);
    content.pack_start(&name_label, false, false, 0);

    card.add(&content);

    let dir_clone = directory.to_string();
    let s = stack.clone();
    let sd = sel_dir.clone();
    card.connect_clicked(move |_| {
        *sd.borrow_mut() = Some(dir_clone.clone());
        s.set_visible_child_name("editor");
        crate::ui::code_editor::trigger_editor_rebuild();
    });

    // Action buttons row
    let actions = GtkBox::new(Orientation::Horizontal, 4);
    actions.set_halign(Align::Center);
    actions.set_margin_top(2);
    actions.set_margin_bottom(6);

    let edit_btn = Button::with_label("✏️");
    edit_btn.style_context().add_class("project-action-btn");
    edit_btn.set_size_request(28, 28);

    let delete_btn = Button::with_label("🗑");
    delete_btn.style_context().add_class("project-action-btn");
    delete_btn.style_context().add_class("project-delete-btn");
    delete_btn.set_size_request(28, 28);

    actions.pack_start(&edit_btn, false, false, 0);
    actions.pack_start(&delete_btn, false, false, 0);

    // Main layout
    let outer = GtkBox::new(Orientation::Vertical, 0);
    outer.style_context().add_class("project-card-outer");
    outer.pack_start(&card, true, true, 0);
    outer.pack_start(&actions, false, false, 0);
    container.pack_start(&outer, true, true, 0);

    // Edit handler
    let dir_edit = directory.to_string();
    let name_edit = name.to_string();
    let id_edit = id.to_string();
    let card_edit = card.clone();
    edit_btn.connect_clicked(move |_| {
        show_rename_project_dialog(&name_edit, &dir_edit, &id_edit, &card_edit);
    });

    // Delete handler
    let dir_del = directory.to_string();
    let id_del = id.to_string();
    let grid_del = grid_ref.clone();
    let container_del = container.clone();
    delete_btn.connect_clicked(move |_| {
        show_delete_project_dialog(&dir_del, &container_del, &grid_del);
    });

    container
}

fn show_rename_project_dialog(name: &str, directory: &str, id: &str, card: &Button) {
    let dialog = gtk::Dialog::new();
    dialog.set_modal(true);
    dialog.set_default_size(350, 150);
    dialog.set_title("Rename Project");

    let content = dialog.content_area();
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_spacing(8);

    let label = Label::new(Some("New project name:"));
    label.set_halign(Align::Start);
    content.pack_start(&label, false, false, 0);

    let entry = Entry::new();
    entry.set_text(name);
    content.pack_start(&entry, false, false, 0);

    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Rename", gtk::ResponseType::Ok);
    dialog.show_all();

    let response = dialog.run();
    if response == gtk::ResponseType::Ok {
        let new_name = entry.text().to_string();
        if !new_name.trim().is_empty() && new_name != name {
            let sandbox = dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("vibi-ai")
                .join("sandbox");
            let old_path = sandbox.join(directory);
            let new_path = sandbox.join(&new_name);
            
            // Rename directory on disk
            if old_path.exists() {
                std::fs::rename(&old_path, &new_path).ok();
            }
            
            // Update .vibecode file
            let proj_file = new_path.join(".vibecode");
            if proj_file.exists() {
                if let Ok(data) = std::fs::read_to_string(&proj_file) {
                    let updated = data.replace(&format!("name={}", name), &format!("name={}", new_name));
                    std::fs::write(&proj_file, updated).ok();
                }
            }
            
            // Update card label
            if let Some(content) = card.child().and_then(|c| c.downcast::<GtkBox>().ok()) {
                for child in content.children() {
                    if let Some(lbl) = child.downcast_ref::<Label>() {
                        if lbl.style_context().has_class("project-card-name") {
                            lbl.set_text(&new_name);
                        }
                    }
                }
            }
        }
    }
    dialog.close();
}

fn show_delete_project_dialog(directory: &str, container: &GtkBox, grid: &Rc<RefCell<gtk::FlowBox>>) {
    let dialog = gtk::Dialog::new();
    dialog.set_modal(true);
    dialog.set_default_size(350, 150);
    dialog.set_title("Delete Project");

    let content = dialog.content_area();
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_margin_top(12);
    content.set_margin_bottom(12);

    let label = Label::new(Some(&format!("Delete project \"{}\"?\nThis will remove the project folder from the sandbox.", directory)));
    label.set_wrap(true);
    label.set_halign(Align::Start);
    content.pack_start(&label, false, false, 0);

    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    let delete_btn = dialog.add_button("Delete", gtk::ResponseType::Ok);
    delete_btn.style_context().add_class("dialog-btn-danger");
    dialog.show_all();

    let response = dialog.run();
    if response == gtk::ResponseType::Ok {
        // Delete from disk
        let sandbox = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("vibi-ai")
            .join("sandbox")
            .join(directory);
        if sandbox.exists() {
            std::fs::remove_dir_all(&sandbox).ok();
        }
        // Remove from UI grid
        let grid_ref = grid.borrow();
        grid_ref.remove(container);
    }
    dialog.close();
}

fn build_add_project_card(_projects: Rc<RefCell<Vec<Project>>>, _grid: gtk::FlowBox, stack: Stack) -> GtkBox {
    let card = GtkBox::new(Orientation::Vertical, 12);
    card.style_context().add_class("project-card-add");
    card.set_size_request(180, 150);

    let plus = Label::new(Some("+"));
    plus.style_context().add_class("project-card-plus");
    plus.set_halign(Align::Center);
    plus.set_valign(Align::Center);
    card.pack_start(&plus, true, true, 0);

    let event_box = gtk::EventBox::new();
    event_box.add(&card);

    // Hover effect for the plus
    let plus_clone = plus.clone();
    event_box.connect_enter_notify_event(move |_, _| {
        plus_clone.style_context().add_class("project-card-plus-hover");
        false.into()
    });

    let plus_clone2 = plus.clone();
    event_box.connect_leave_notify_event(move |_, _| {
        plus_clone2.style_context().remove_class("project-card-plus-hover");
        false.into()
    });

    let s = stack.clone();
    event_box.connect_button_press_event(move |_, _| {
        s.set_visible_child_name("create");
        false.into()
    });

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.pack_start(&event_box, true, true, 0);
    container
}

fn build_create_project_page(projects: Rc<RefCell<Vec<Project>>>, drafts: Rc<RefCell<Vec<DraftProject>>>, stack: Stack) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);

    // Topbar
    let topbar = GtkBox::new(Orientation::Horizontal, 0);
    topbar.style_context().add_class("topbar");
    topbar.set_margin_start(24);
    topbar.set_margin_end(24);
    topbar.set_margin_top(14);
    topbar.set_margin_bottom(14);

    let back_btn = Button::with_label("← Back");
    back_btn.style_context().add_class("agentic-back-btn");
    let s = stack.clone();
    back_btn.connect_clicked(move |_| s.set_visible_child_name("categories"));
    topbar.pack_start(&back_btn, false, false, 0);

    let title = Label::new(Some("Create New Project"));
    title.style_context().add_class("topbar-title");
    title.set_margin_start(12);
    topbar.pack_start(&title, false, false, 0);

    let divider = Separator::new(Orientation::Horizontal);
    divider.style_context().add_class("topbar-divider");

    root.pack_start(&topbar, false, false, 0);
    root.pack_start(&divider, false, false, 0);

    let scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroll.set_vexpand(true);
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);

    let form = GtkBox::new(Orientation::Vertical, 16);
    form.set_margin_start(60);
    form.set_margin_end(60);
    form.set_margin_top(24);
    form.set_margin_bottom(40);

    // Project Name
    let name_label = Label::new(Some("Project Name"));
    name_label.style_context().add_class("create-label");
    name_label.set_halign(Align::Start);
    form.pack_start(&name_label, false, false, 0);

    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("My Awesome Project"));
    name_entry.style_context().add_class("create-input");
    form.pack_start(&name_entry, false, false, 0);

    // Project Directory
    let dir_label = Label::new(Some("Project Directory"));
    dir_label.style_context().add_class("create-label");
    dir_label.set_halign(Align::Start);
    form.pack_start(&dir_label, false, false, 0);

    let dir_box = GtkBox::new(Orientation::Horizontal, 8);
    let dir_entry = Entry::new();
    dir_entry.set_placeholder_text(Some("./my_project"));
    dir_entry.style_context().add_class("create-input");
    dir_entry.set_hexpand(true);
    dir_box.pack_start(&dir_entry, true, true, 0);

    let browse_btn = Button::with_label("Browse");
    browse_btn.style_context().add_class("create-browse-btn");
    dir_box.pack_start(&browse_btn, false, false, 0);
    form.pack_start(&dir_box, false, false, 0);

    // GitHub Section
    let gh_section = Label::new(Some("GitHub Repository (Optional)"));
    gh_section.style_context().add_class("create-section-title");
    gh_section.set_halign(Align::Start);
    gh_section.set_margin_top(12);
    form.pack_start(&gh_section, false, false, 0);

    let gh_label = Label::new(Some("Repository Link"));
    gh_label.style_context().add_class("create-label");
    gh_label.set_halign(Align::Start);
    form.pack_start(&gh_label, false, false, 0);

    let gh_entry = Entry::new();
    gh_entry.set_placeholder_text(Some("https://github.com/username/repo"));
    gh_entry.style_context().add_class("create-input");
    form.pack_start(&gh_entry, false, false, 0);

    let gh_user_label = Label::new(Some("GitHub Username"));
    gh_user_label.style_context().add_class("create-label");
    gh_user_label.set_halign(Align::Start);
    form.pack_start(&gh_user_label, false, false, 0);

    let gh_user_entry = Entry::new();
    gh_user_entry.set_placeholder_text(Some("your-username"));
    gh_user_entry.style_context().add_class("create-input");
    form.pack_start(&gh_user_entry, false, false, 0);

    // Public/Private toggle
    let visibility_label = Label::new(Some("Is this repository public or private?"));
    visibility_label.style_context().add_class("create-label");
    visibility_label.set_halign(Align::Start);
    form.pack_start(&visibility_label, false, false, 0);

    let vis_box = GtkBox::new(Orientation::Horizontal, 8);
    let public_btn = Button::with_label("Public");
    public_btn.style_context().add_class("create-toggle-btn");
    let private_btn = Button::with_label("Private");
    private_btn.style_context().add_class("create-toggle-btn");

    let is_private = Rc::new(RefCell::new(false));
    let ip = is_private.clone();
    let priv_btn_clone = private_btn.clone();
    public_btn.connect_clicked(move |btn| {
        *ip.borrow_mut() = false;
        btn.style_context().add_class("create-toggle-active");
        priv_btn_clone.style_context().remove_class("create-toggle-active");
    });
    let ip2 = is_private.clone();
    let pub_btn_clone = public_btn.clone();
    private_btn.connect_clicked(move |btn| {
        *ip2.borrow_mut() = true;
        btn.style_context().add_class("create-toggle-active");
        pub_btn_clone.style_context().remove_class("create-toggle-active");
    });
    public_btn.style_context().add_class("create-toggle-active");

    vis_box.pack_start(&public_btn, false, false, 0);
    vis_box.pack_start(&private_btn, false, false, 0);
    form.pack_start(&vis_box, false, false, 0);

    // Token question
    let token_q = Label::new(Some("Do you have a GitHub token saved in your environment?"));
    token_q.style_context().add_class("create-label");
    token_q.set_halign(Align::Start);
    form.pack_start(&token_q, false, false, 0);

    let token_toggle = GtkBox::new(Orientation::Horizontal, 8);
    let token_yes = Button::with_label("Yes");
    token_yes.style_context().add_class("create-toggle-btn");
    let token_no = Button::with_label("No");
    token_no.style_context().add_class("create-toggle-btn");

    let has_token = Rc::new(RefCell::new(false));
    let token_entry = Entry::new();
    token_entry.set_placeholder_text(Some("ghp_xxxxxxxxxxxxxxxxxxxx"));
    token_entry.style_context().add_class("create-input");
    token_entry.set_visible(false);

    let ht = has_token.clone();
    let te = token_entry.clone();
    let token_no_clone = token_no.clone();
    token_yes.connect_clicked(move |btn| {
        *ht.borrow_mut() = true;
        btn.style_context().add_class("create-toggle-active");
        token_no_clone.style_context().remove_class("create-toggle-active");
        te.set_visible(false);
    });
    let ht2 = has_token.clone();
    let te2 = token_entry.clone();
    let token_yes_clone = token_yes.clone();
    token_no.connect_clicked(move |btn| {
        *ht2.borrow_mut() = false;
        btn.style_context().add_class("create-toggle-active");
        token_yes_clone.style_context().remove_class("create-toggle-active");
        te2.set_visible(true);
    });
    token_no.style_context().add_class("create-toggle-active");

    token_toggle.pack_start(&token_yes, false, false, 0);
    token_toggle.pack_start(&token_no, false, false, 0);
    form.pack_start(&token_toggle, false, false, 0);
    form.pack_start(&token_entry, false, false, 0);

    // Action buttons
    let actions = GtkBox::new(Orientation::Horizontal, 12);
    actions.set_halign(Align::End);
    actions.set_margin_top(20);

    let cancel_btn = Button::with_label("Cancel");
    cancel_btn.style_context().add_class("create-cancel-btn");
    let draft_btn = Button::with_label("Save as Draft");
    draft_btn.style_context().add_class("create-draft-btn");
    let create_btn = Button::with_label("Create Project");
    create_btn.style_context().add_class("create-submit-btn");

    actions.pack_start(&cancel_btn, false, false, 0);
    actions.pack_start(&draft_btn, false, false, 0);
    actions.pack_start(&create_btn, false, false, 0);
    form.pack_start(&actions, false, false, 0);

    scroll.add(&form);
    root.pack_start(&scroll, true, true, 0);

    // Wire browse button
    let dir_clone = dir_entry.clone();
    browse_btn.connect_clicked(move |_| {
        let fc = FileChooserDialog::new::<gtk::Window>(
            Some("Choose Project Directory"),
            None,
            gtk::FileChooserAction::SelectFolder,
        );
        fc.add_button("Select", gtk::ResponseType::Ok);
        fc.add_button("Cancel", gtk::ResponseType::Cancel);
        let de = dir_clone.clone();
        fc.connect_response(move |d, resp| {
            if resp == gtk::ResponseType::Ok {
                if let Some(path) = d.file() {
                    de.set_text(&path.path().unwrap().to_string_lossy());
                }
            }
            d.close();
        });
        fc.show_all();
    });

    // Wire cancel — clear and go back
    let s_cancel = stack.clone();
    let ne = name_entry.clone();
    let de = dir_entry.clone();
    let ge = gh_entry.clone();
    let gue = gh_user_entry.clone();
    let te = token_entry.clone();
    cancel_btn.connect_clicked(move |_| {
        ne.set_text("");
        de.set_text("");
        ge.set_text("");
        gue.set_text("");
        te.set_text("");
        s_cancel.set_visible_child_name("categories");
    });

    // Wire create
    let proj_create = projects.clone();
    let s_create = stack.clone();
    let ne2 = name_entry.clone();
    let de2 = dir_entry.clone();
    let ge2 = gh_entry.clone();
    let gue2 = gh_user_entry.clone();
    let ip3 = is_private.clone();
    let ht3 = has_token.clone();
    let te3 = token_entry.clone();
    create_btn.connect_clicked(move |_| {
        let name = ne2.text().to_string();
        let dir = de2.text().to_string();
        if name.trim().is_empty() || dir.trim().is_empty() {
            return;
        }
        let gh = ge2.text().to_string();
        let project_uuid = uuid::Uuid::new_v4().to_string();
        let project_name = name.trim().to_string();
        
        // Create project directory in sandbox using project name
        let sandbox_base = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("vibi-ai")
            .join("sandbox")
            .join(&project_name);
        std::fs::create_dir_all(&sandbox_base).ok();
        
        // Copy files from selected directory into sandbox/project_name
        let source_dir = std::path::PathBuf::from(dir.trim());
        if source_dir.exists() && source_dir.is_dir() {
            fn copy_dir(src: &std::path::Path, dst: &std::path::Path) {
                if let Ok(entries) = std::fs::read_dir(src) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let dest = dst.join(entry.file_name());
                        if path.is_dir() {
                            std::fs::create_dir_all(&dest).ok();
                            copy_dir(&path, &dest);
                        } else {
                            std::fs::copy(&path, &dest).ok();
                        }
                    }
                }
            }
            copy_dir(&source_dir, &sandbox_base);
        }
        
        // Write .vibecode file
        let proj_file = sandbox_base.join(".vibecode");
        let proj_data = format!("uuid={}\nid={}\nname={}\ncreated={}\n",
            project_uuid,
            project_uuid,
            &project_name,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        );
        std::fs::write(&proj_file, proj_data).ok();
        
        let project = Project {
            id: project_uuid.clone(),
            uuid: project_uuid,
            directory: project_name.clone(),
            name: project_name,
            category: "custom".to_string(),
            files: HashMap::new(),
            linked_chat: None,
            github_link: if gh.is_empty() { None } else { Some(gh) },
        };
        proj_create.borrow_mut().push(project);
        // Clear fields
        ne2.set_text("");
        de2.set_text("");
        ge2.set_text("");
        gue2.set_text("");
        te3.set_text("");
        s_create.set_visible_child_name("categories");
    });

    // Wire draft
    let d = drafts.clone();
    let ne_d = name_entry.clone();
    let de_d = dir_entry.clone();
    let ge_d = gh_entry.clone();
    let gue_d = gh_user_entry.clone();
    let ip_d = is_private.clone();
    let ht_d = has_token.clone();
    let te_d = token_entry.clone();
    let s_draft = stack.clone();
    draft_btn.connect_clicked(move |_| {
        let draft = DraftProject {
            name: ne_d.text().to_string(),
            directory: de_d.text().to_string(),
            github_link: if ge_d.text().is_empty() { None } else { Some(ge_d.text().to_string()) },
            github_user: if gue_d.text().is_empty() { None } else { Some(gue_d.text().to_string()) },
            is_private: *ip_d.borrow(),
            has_token: *ht_d.borrow(),
            token: if te_d.text().is_empty() { None } else { Some(te_d.text().to_string()) },
        };
        d.borrow_mut().push(draft);
        s_draft.set_visible_child_name("categories");
    });

    root
}

fn build_draft_card(name: &str, index: usize, drafts: Rc<RefCell<Vec<DraftProject>>>) -> GtkBox {
    let card = GtkBox::new(Orientation::Vertical, 12);
    card.style_context().add_class("project-card");
    card.set_size_request(180, 150);

    let icon = Label::new(Some("📝"));
    icon.style_context().add_class("project-card-icon");
    icon.set_halign(Align::Center);
    card.pack_start(&icon, false, false, 0);

    let name_label = Label::new(Some(name));
    name_label.style_context().add_class("project-card-name");
    name_label.set_halign(Align::Center);
    name_label.set_ellipsize(pango::EllipsizeMode::End);
    name_label.set_max_width_chars(14);
    card.pack_start(&name_label, false, false, 0);

    // Draft badge
    let badge = Label::new(Some("DRAFT"));
    badge.style_context().add_class("project-draft-badge");
    badge.set_halign(Align::Center);
    card.pack_start(&badge, false, false, 0);

    let event_box = gtk::EventBox::new();
    event_box.add(&card);

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.pack_start(&event_box, true, true, 0);
    container
}

fn get_tree_scroll_value(tree_holder: &GtkBox) -> f64 {
    if let Some(tree) = tree_holder.children().get(0) {
        if let Some(inner) = tree.clone().downcast::<GtkBox>().ok() {
            if let Some(scroll_widget) = inner.children().get(2) {
                if let Ok(sw) = scroll_widget.clone().downcast::<ScrolledWindow>() {
                    return sw.vadjustment().value();
                }
            }
        }
    }
    0.0
}

fn restore_tree_scroll_value(tree_holder: &Rc<RefCell<GtkBox>>, value: f64) {
    let th = tree_holder.clone();
    gtk::glib::idle_add_local_once(move || {
        let holder = th.borrow();
        if let Some(tree) = holder.children().get(0) {
            if let Some(inner) = tree.clone().downcast::<GtkBox>().ok() {
                if let Some(scroll_widget) = inner.children().get(2) {
                    if let Ok(sw) = scroll_widget.clone().downcast::<ScrolledWindow>() {
                        sw.vadjustment().set_value(value);
                    }
                }
            }
        }
    });
}

fn rebuild_tab_bar(
    tab_bar: &Rc<RefCell<GtkBox>>,
    open_tabs: &Rc<RefCell<Vec<(String, String)>>>,
    code_view: &gtk::TextView,
    active_index: usize,
    tab_scroll: &gtk::ScrolledWindow,
) {
    // Collect all data first — no GTK mutations during borrows
    let parent = tab_bar.borrow().parent();
    let tabs_snapshot: Vec<(String, String)> = open_tabs.borrow().clone();
    
    let new_tb = GtkBox::new(Orientation::Horizontal, 0);
    new_tb.style_context().add_class("editor-tab-bar");
    
    let buffer = code_view.buffer();
    
    for (i, (path, name)) in tabs_snapshot.iter().enumerate() {
        let tab_btn = Button::new();
        tab_btn.set_relief(gtk::ReliefStyle::None);
        tab_btn.style_context().add_class("editor-tab");
        if i == active_index {
            tab_btn.style_context().add_class("editor-tab-active");
        }
        
        let tab_content = GtkBox::new(Orientation::Horizontal, 4);
        let tab_label = Label::new(Some(name));
        tab_label.set_halign(Align::Start);
        tab_content.pack_start(&tab_label, false, false, 0);
        
        let close_btn = Button::with_label("×");
        close_btn.set_relief(gtk::ReliefStyle::None);
        close_btn.style_context().add_class("editor-tab-close");
        tab_content.pack_start(&close_btn, false, false, 0);
        
        tab_btn.add(&tab_content);
        new_tb.pack_start(&tab_btn, false, false, 0);
        
        // Wire close button
        let path_close = path.clone();
        let tabs_for_close = open_tabs.clone();
        let tb_for_close = tab_bar.clone();
        let cv_for_close = code_view.clone();
        if let Some(ref buf) = buffer {
            let buf_close = buf.clone();
            close_btn.connect_clicked(move |_| {
                let now_empty = {
                    let mut tabs = tabs_for_close.borrow_mut();
                    tabs.retain(|(p, _)| p != &path_close);
                    tabs.is_empty()
                };
                if now_empty {
                    buf_close.set_text("");
                    crate::ui::code_editor::trigger_editor_rebuild();
                }
                rebuild_tab_bar(&tb_for_close, &tabs_for_close, &cv_for_close, 0);
            });
        }
        
        // Wire tab click
        let path_tab = path.clone();
        let path_close = path.clone();
        let tabs_for_click = open_tabs.clone();
        let tb_for_click = tab_bar.clone();
        let cv_for_click = code_view.clone();
        if let Some(ref buf) = buffer {
            let buf_tab = buf.clone();
            tab_btn.connect_clicked(move |_| {
                if let Ok(content) = std::fs::read_to_string(&path_tab) {
                    buf_tab.set_text(&content);
                }
                let new_active = tabs_for_click.borrow().iter().position(|(p, _)| p == &path_tab).unwrap_or(0);
                rebuild_tab_bar(&tb_for_click, &tabs_for_click, &cv_for_click, new_active);
            });
        }
        

    }
    
        let plus_btn = Button::with_label("+");
    plus_btn.set_relief(gtk::ReliefStyle::None);
    plus_btn.style_context().add_class("editor-tab-new");
    let ot_plus = open_tabs.clone();
    let tb_plus = tab_bar.clone();
    let cv_plus = code_view.clone();
    plus_btn.connect_clicked(move |_| {
        let mut tabs = ot_plus.borrow_mut();
        let untitled = format!("Untitled-{}", tabs.len() + 1);
        tabs.push(("".to_string(), untitled.clone()));
        let new_idx = tabs.len() - 1;
        drop(tabs);
        if let Some(buf) = cv_plus.buffer() {
            buf.set_text("");
        }
        rebuild_tab_bar(&tb_plus, &ot_plus, &cv_plus, new_idx);
    });
    new_tb.pack_start(&plus_btn, false, false, 0);


    // Now mutate GTK — all borrows above are dropped
    if let Some(child) = tab_scroll.child() {
        tab_scroll.remove(&child);
    }
    tab_scroll.add(&new_tb);
    
    *tab_bar.borrow_mut() = new_tb;
    tab_bar.borrow().show_all();
}

fn build_code_editor_page(stack: Stack, project_directory: Rc<RefCell<Option<String>>>) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);

    let topbar = GtkBox::new(Orientation::Horizontal, 0);
    topbar.style_context().add_class("topbar");
    topbar.set_margin_start(24);
    topbar.set_margin_end(24);
    topbar.set_margin_top(14);
    topbar.set_margin_bottom(14);

    let back_btn = Button::with_label("← Projects");
    back_btn.style_context().add_class("agentic-back-btn");
    let s = stack.clone();
    back_btn.connect_clicked(move |_| s.set_visible_child_name("categories"));
    topbar.pack_start(&back_btn, false, false, 0);

    let title = Label::new(Some("Code Editor"));
    title.style_context().add_class("topbar-title");
    title.set_margin_start(12);
    topbar.pack_start(&title, false, false, 0);

    let divider = Separator::new(Orientation::Horizontal);
    divider.style_context().add_class("topbar-divider");

    root.pack_start(&topbar, false, false, 0);
    root.pack_start(&divider, false, false, 0);

    let editor_scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    editor_scroll.set_hexpand(true);
    editor_scroll.set_vexpand(true);
    editor_scroll.set_policy(PolicyType::Automatic, PolicyType::Automatic);

    let editor_box = GtkBox::new(Orientation::Horizontal, 0);
    editor_box.set_hexpand(true);
    editor_box.set_vexpand(true);

    // Shared state for tabs and editor
    let tab_bar = Rc::new(RefCell::new(GtkBox::new(Orientation::Horizontal, 0)));
    tab_bar.borrow().style_context().add_class("editor-tab-bar");
    
    // Track open tabs: (file_path, file_name)
    let open_tabs: Rc<RefCell<Vec<(String, String)>>> = Rc::new(RefCell::new(Vec::new()));
    
    let code_view = gtk::TextView::new();
    code_view.set_vexpand(true);
    code_view.set_hexpand(true);
    code_view.set_wrap_mode(gtk::WrapMode::None);
    code_view.set_monospace(true);
    code_view.set_left_margin(12);
    code_view.set_top_margin(8);
    code_view.set_editable(true);
    code_view.set_cursor_visible(true);
    
    let code_scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    code_scroll.set_policy(PolicyType::Automatic, PolicyType::Automatic);
    code_scroll.add(&code_view);

    // Welcome overlay
    let welcome_overlay = gtk::Overlay::new();
    welcome_overlay.add(&code_scroll);
    
    let welcome_box = GtkBox::new(Orientation::Vertical, 16);
    welcome_box.set_halign(Align::Center);
    welcome_box.set_valign(Align::Center);
    welcome_box.set_hexpand(true);
    welcome_box.set_vexpand(true);
    
    let claw_icon = Label::new(Some("V"));
    claw_icon.style_context().add_class("welcome-claw-icon");
    claw_icon.set_halign(Align::Center);
    welcome_box.pack_start(&claw_icon, false, false, 0);
    
    let welcome_title = Label::new(Some("VibiClaw Code Editor"));
    welcome_title.style_context().add_class("welcome-title-text");
    welcome_title.set_halign(Align::Center);
    welcome_box.pack_start(&welcome_title, false, false, 0);
    
    let welcome_sub = Label::new(Some("Open a file from the tree to start editing"));
    welcome_sub.style_context().add_class("welcome-sub-text");
    welcome_sub.set_halign(Align::Center);
    welcome_box.pack_start(&welcome_sub, false, false, 0);
    
    welcome_overlay.add_overlay(&welcome_box);
    
    let welcome_ref = Rc::new(RefCell::new(welcome_box));

    // File tree holder — stable reference for rebuilds
    let tree_holder = Rc::new(RefCell::new(GtkBox::new(Orientation::Vertical, 0)));
    let file_tree = build_file_tree(code_view.clone(), tab_bar.clone(), project_directory.clone(), welcome_ref.clone(), open_tabs.clone());
    tree_holder.borrow().pack_start(&file_tree, true, true, 0);

    editor_box.pack_start(&*tree_holder.borrow(), false, false, 0);
    editor_box.pack_start(&Separator::new(Orientation::Vertical), false, false, 0);

    // Editor area with tabs + code
    let editor_area = GtkBox::new(Orientation::Vertical, 0);
    editor_area.set_hexpand(true);
    editor_area.set_vexpand(true);
    
    let dir_label = Label::new(Some("📂 Loading..."));
    dir_label.style_context().add_class("editor-dir-label");
    dir_label.set_halign(Align::Start);
    dir_label.set_margin_start(12);
    dir_label.set_margin_top(4);
    let dir_label_clone = dir_label.clone();
    editor_area.pack_start(&dir_label, false, false, 0);
    let tab_scroll = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    tab_scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);
    tab_scroll.set_min_content_height(35);
    tab_scroll.set_hexpand(true);
    let tab_scroll_rc = Rc::new(tab_scroll);
    tab_scroll_rc.add(&*tab_bar.borrow());
    editor_area.pack_start(&*tab_scroll_rc, false, false, 0);
    editor_area.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);
    editor_area.pack_start(&welcome_overlay, true, true, 0);

    editor_box.pack_start(&editor_area, true, true, 0);
    editor_scroll.add(&editor_box);
    root.pack_start(&editor_scroll, true, true, 0);

    // Register rebuild function
    let pd_rebuild = project_directory.clone();
    let cv_rebuild = code_view.clone();
    let tb_rebuild = tab_bar.clone();
    let th_rebuild = tree_holder.clone();
    let dl_rebuild = dir_label_clone.clone();
    let wr_rebuild = welcome_ref.clone();
    let ot_rebuild = open_tabs.clone();
    EDITOR_REBUILD.with(|r| {
        *r.borrow_mut() = Some(Box::new(move || {
            let dir_text = match pd_rebuild.borrow().as_ref() {
                Some(d) => format!("Project: {}", d),
                None => "Sandbox (root)".to_string(),
            };
            dl_rebuild.set_text(&dir_text);
            let scroll_pos = get_tree_scroll_value(&th_rebuild.borrow());
            let holder = th_rebuild.borrow();
            for child in holder.children() {
                holder.remove(&child);
            }
            let new_tree = build_file_tree(cv_rebuild.clone(), tb_rebuild.clone(), pd_rebuild.clone(), wr_rebuild.clone(), ot_rebuild.clone());
            holder.pack_start(&new_tree, true, true, 0);
            holder.show_all();
            drop(holder);
            restore_tree_scroll_value(&th_rebuild, scroll_pos);
        }));
    });
    
    // Rebuild file tree every time editor becomes visible
    let pd_map = project_directory.clone();
    let cv_map = code_view.clone();
    let tb_map = tab_bar.clone();
    let th_map = tree_holder.clone();
    let dl_map = dir_label_clone.clone();
    let wr_map = welcome_ref.clone();
    let ot_map = open_tabs.clone();
    root.connect_map(move |_| {
        let dir_text = match pd_map.borrow().as_ref() {
            Some(d) => format!("Project: {}", d),
            None => "Sandbox (root)".to_string(),
        };
        dl_map.set_text(&dir_text);

        let scroll_pos = get_tree_scroll_value(&th_map.borrow());
        let holder = th_map.borrow();
        for child in holder.children() {
            holder.remove(&child);
        }
        let new_tree = build_file_tree(cv_map.clone(), tb_map.clone(), pd_map.clone(), wr_map.clone(), ot_map.clone());
        holder.pack_start(&new_tree, true, true, 0);
        holder.show_all();
        drop(holder);
        restore_tree_scroll_value(&th_map, scroll_pos);
    });
    
    // File watcher — rebuild only on actual changes (debounced)
    let pd_watch = project_directory.clone();
    let cv_watch = code_view.clone();
    let tb_watch = tab_bar.clone();
    let th_watch = tree_holder.clone();
    let wr_watch = welcome_ref.clone();
    let ot_watch = open_tabs.clone();
    
    let (tx, rx) = async_channel::unbounded::<()>();
    let tx_clone = tx.clone();
    
    // Watch sandbox directory for changes
    let sandbox_watch = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("vibi-ai")
        .join("sandbox");
    
    std::thread::spawn(move || {
        use inotify::{Inotify, WatchMask};
        if let Ok(mut watcher) = Inotify::init() {
            // Helper to recursively add watches
            fn add_watch_recursive(watcher: &mut Inotify, path: &std::path::Path) {
                let _ = watcher.watches().add(path, WatchMask::CREATE | WatchMask::DELETE | WatchMask::MODIFY | WatchMask::MOVED_TO | WatchMask::MOVED_FROM);
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            add_watch_recursive(watcher, &entry.path());
                        }
                    }
                }
            }
            add_watch_recursive(&mut watcher, &sandbox_watch);
            
            let mut buffer = [0u8; 4096];
            loop {
                if watcher.read_events(&mut buffer).is_ok() {
                    // Check for new directories and watch them too
                    add_watch_recursive(&mut watcher, &sandbox_watch);
                    // Debounce: wait 300ms for burst of events to settle
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    let _ = tx_clone.try_send(());
                }
            }
        }
    });
    
    // Receive change events on main thread and rebuild
    gtk::glib::spawn_future_local(async move {
        while let Ok(()) = rx.recv().await {
            let scroll_pos = get_tree_scroll_value(&th_watch.borrow());
            let holder = th_watch.borrow();
            for child in holder.children() {
                holder.remove(&child);
            }
            let wr_watch = welcome_ref.clone();
            let new_tree = build_file_tree(cv_watch.clone(), tb_watch.clone(), pd_watch.clone(), wr_watch.clone(), ot_watch.clone());
            holder.pack_start(&new_tree, true, true, 0);
            holder.show_all();
            drop(holder);
            restore_tree_scroll_value(&th_watch, scroll_pos);
        }
    });
    
    root
}

fn get_icon_path(filename: &str) -> Option<String> {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    let icons_dir = std::path::PathBuf::from("src/icons");
    
    let icon_name = match ext.as_str() {
        "rs" => "rust-programming-language-icon.png",
        "py" => "python-programming-language-icon.png",
        "js" => "javascript-programming-language-icon.png",
        "ts" => "typescript-programming-language-icon.png",
        "jsx" | "tsx" => "react-js-icon.png",
        "html" => "html-icon.png",
        "css" => "css-icon.png",
        "scss" | "sass" => "css-icon.png",
        "json" | "xml" | "yaml" | "yml" | "toml" => "yaml-icon.png",
        "md" | "txt" | "log" => "log-file-icon.png",
        "c" | "h" => "c-program-icon.png",
        "cpp" | "cc" | "cxx" | "hpp" => "c-plus-plus-programming-language-icon.png",
        "cs" => "c-sharp-programming-language-icon.png",
        "java" => "java-programming-language-icon.png",
        "kt" | "kts" => "kotlin-programming-language-icon.png",
        "swift" => "swift-programming-language-icon.png",
        "rb" => "ruby-programming-language-icon.png",
        "php" => "php-programming-language-icon.png",
        "r" => "r-programming-language-icon.png",
        "sql" | "sqlite" | "db" => "sqlite-icon.png",
        "sh" | "bash" | "zsh" | "ps1" => "bash-unix-shell-icon.png",
        "dart" => "dart-programming-language-icon.png",
        "zig" => "zig-programming-language-icon.png",
        "pl" => "perl-programming-language-icon.png",
        "node" => "node-js-icon.png",
        "flutter" => "flutter-icon.png",
        _ => return None,
    };
    
    let path = icons_dir.join(icon_name);
    if path.exists() {
        Some(path.to_string_lossy().to_string())
    } else {
        None
    }
}

fn build_file_tree(code_view: gtk::TextView, tab_bar: Rc<RefCell<GtkBox>>, project_directory: Rc<RefCell<Option<String>>>, welcome_overlay: Rc<RefCell<GtkBox>>, open_tabs: Rc<RefCell<Vec<(String, String)>>>, tab_scroll: Rc<gtk::ScrolledWindow>) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.set_size_request(220, -1);
    container.style_context().add_class("file-tree");

    let header = Label::new(Some("📁 Sandbox"));
    header.style_context().add_class("file-tree-header");
    header.set_halign(Align::Start);
    header.set_margin_start(12);
    header.set_margin_top(8);
    header.set_margin_bottom(8);
    container.pack_start(&header, false, false, 0);
    container.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);

    let scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroll.set_vexpand(true);
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);

    let tree_list = GtkBox::new(Orientation::Vertical, 0);
    tree_list.set_vexpand(true);

    let stored_dir = project_directory.borrow().clone();
    let sandbox = if let Some(ref dir) = stored_dir {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("vibi-ai")
            .join("sandbox")
            .join(dir)
    } else {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("vibi-ai")
            .join("sandbox")
    };
    
    fn add_directory_entries(
        parent: &GtkBox,
        path: &std::path::Path,
        depth: i32,
        code_view: &gtk::TextView,
        tab_bar: &Rc<RefCell<GtkBox>>,
        welcome_ref: &Rc<RefCell<GtkBox>>,
        open_tabs: &Rc<RefCell<Vec<(String, String)>>>,
        tab_scroll: &Rc<gtk::ScrolledWindow>,
    ) {
        if depth > 6 { return; }
        if let Ok(entries) = std::fs::read_dir(path) {
            let mut items: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            items.sort_by(|a, b| {
                let a_dir = a.path().is_dir();
                let b_dir = b.path().is_dir();
                if a_dir != b_dir { return b_dir.cmp(&a_dir); }
                a.file_name().cmp(&b.file_name())
            });
            for entry in &items {
                let is_dir = entry.path().is_dir();
                let name = entry.file_name().to_string_lossy().to_string();
                let entry_path = entry.path();
                let path_str = entry_path.to_string_lossy().to_string();
                let row = Button::new();
                row.set_relief(gtk::ReliefStyle::None);
                row.style_context().add_class("file-tree-item");
                row.set_size_request(-1, 20);
                
                let row_content = GtkBox::new(Orientation::Horizontal, 4);
                
                if is_dir {
                    let collapsed = is_folder_collapsed(&path_str);
                    let arrow = if collapsed { "▶" } else { "▼" };
                    let icon_label = Label::new(Some("📁"));
                    icon_label.set_size_request(16, 16);
                    row_content.pack_start(&icon_label, false, false, 0);
                    let name_label = Label::new(Some(&format!("{} {}", arrow, name)));
                    row_content.pack_start(&name_label, false, false, 0);
                } else {
                    if let Some(icon_path) = get_icon_path(&name) {
                        if let Ok(pixbuf) = Pixbuf::from_file_at_scale(&icon_path, 14, 14, true) {
                            let icon_image = gtk::Image::from_pixbuf(Some(&pixbuf));
                            row_content.pack_start(&icon_image, false, false, 0);
                        }
                    } else {
                        let fallback = Label::new(Some("📄"));
                        fallback.style_context().add_class("file-tree-item");
                        row_content.pack_start(&fallback, false, false, 0);
                    }
                    let name_label = Label::new(Some(&name));
                    row_content.pack_start(&name_label, false, false, 0);
                }
                
                row.add(&row_content);
                row.set_halign(Align::Start);
                row.set_margin_start(12 + depth * 12);
                row.set_margin_top(1);
                row.set_margin_bottom(1);
                
                if !is_dir {
                    let file_path = entry_path.clone();
                    let cv = code_view.clone();
                    let tb = tab_bar.clone();
                    let file_name = name.clone();
                    let wref = welcome_ref.clone();
                    let tabs = open_tabs.clone();
                    let tb_clone = tab_bar.clone();
                    let cv_clone = code_view.clone();
                    row.connect_clicked(move |_| {
                        wref.borrow().hide();
                            if let Ok(content) = std::fs::read_to_string(&file_path) {
                            if let Some(buffer) = cv_clone.buffer() {
                                buffer.set_text(&content);
                            }
                        }
                        // Add to tabs if not already open
                        let idx = {
                            let mut tabs_mut = tabs.borrow_mut();
                            match tabs_mut.iter().position(|(p, _)| p == &file_path) {
                                Some(idx) => idx,
                                None => {
                                    tabs_mut.push((file_path.to_string_lossy().to_string(), file_name.clone()));
                                    tabs_mut.len() - 1
                                }
                            }
                        };
                        rebuild_tab_bar(&tb_clone, &tabs, &cv_clone, idx, &tab_scroll);
                    });
                    parent.pack_start(&row, false, false, 0);
                } else {
                    // Create a container for this folder's children
                    let children_box = Rc::new(RefCell::new(GtkBox::new(Orientation::Vertical, 0)));
                    
                    let collapsed = is_folder_collapsed(&path_str);
                    if !collapsed {
                        add_directory_entries(&*children_box.borrow(), &entry_path, depth + 1, code_view, tab_bar, welcome_ref, open_tabs);
                        children_box.borrow().show_all();
                    }
                    
                    parent.pack_start(&row, false, false, 0);
                    let child_wrapper = GtkBox::new(Orientation::Vertical, 0);
                    child_wrapper.pack_start(&*children_box.borrow(), false, false, 0);
                    parent.pack_start(&child_wrapper, false, false, 0);
                    
                    if collapsed {
                        child_wrapper.hide();
                    }
                    
                    let path_clone = path_str.clone();
                    let cb = children_box.clone();
                    let cw = child_wrapper.clone();
                    let r = row.clone();
                    let cv = code_view.clone();
                    let tb = tab_bar.clone();
                    let wr = welcome_ref.clone();
                    let ot = open_tabs.clone();
                    row.connect_clicked(move |_| {
                        let new_collapsed = toggle_folder_state(&path_clone);
                        let arrow = if new_collapsed { "▶" } else { "▼" };
                        r.set_label(&format!("{} {}", arrow, name));
                        if new_collapsed {
                            cw.hide();
                        } else {
                            let cb_ref = cb.borrow();
                            let existing = cb_ref.children();
                            for child in &existing {
                                cb_ref.remove(child);
                            }
                            add_directory_entries(&*cb_ref, &entry_path, depth + 1, &cv, &tb, &wr, &ot);
                            cb_ref.show_all();
                            cw.show();
                        }
                    });
                }
            }
        }
    }
    
    add_directory_entries(&tree_list, &sandbox, 0, &code_view, &tab_bar, &welcome_overlay, &open_tabs);

    scroll.add(&tree_list);
    container.pack_start(&scroll, true, true, 0);
    container
}
