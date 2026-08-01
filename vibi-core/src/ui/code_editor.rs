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
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);
    root.style_context().add_class("agentic-root");

    let stack = Stack::new();
    stack.set_hexpand(true);
    stack.set_vexpand(true);

    let categories_page = build_categories_page(projects.clone(), drafts.clone(), stack.clone());
    stack.add_titled(&categories_page, "categories", "Categories");



    let create_page = build_create_project_page(projects.clone(), drafts.clone(), stack.clone());
    stack.add_titled(&create_page, "create", "Create Project");

    let editor_page = build_code_editor_page(stack.clone());
    stack.add_titled(&editor_page, "editor", "Editor");

    root.pack_start(&stack, true, true, 0);
    root
}

fn build_categories_page(projects: Rc<RefCell<Vec<Project>>>, drafts: Rc<RefCell<Vec<DraftProject>>>, stack: Stack) -> GtkBox {
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
    projects_grid.set_homogeneous(true);
    projects_grid.set_row_spacing(16);
    projects_grid.set_column_spacing(16);
    projects_grid.set_halign(Align::Start);

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
                        let proj_file = path.join(".vibi_project");
                        if proj_file.exists() {
                            if let Ok(data) = std::fs::read_to_string(&proj_file) {
                                let name = data.lines()
                                    .find(|l| l.starts_with("name="))
                                    .map(|l| l[5..].to_string())
                                    .unwrap_or_else(|| path.file_name().unwrap().to_string_lossy().to_string());
                                let id = data.lines()
                                    .find(|l| l.starts_with("id="))
                                    .map(|l| l[3..].to_string())
                                    .unwrap_or_else(|| format!("proj_{}", chrono::Utc::now().timestamp()));
                                // Only add if not already in the list
                                let mut proj_list = projects.borrow_mut();
                                if !proj_list.iter().any(|p| p.id == id) {
                                    proj_list.push(Project {
                                        id,
                                        name,
                                        category: "custom".to_string(),
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
        move || {
            let grid = g.borrow();
            let children = grid.children();
            for child in &children {
                grid.remove(child);
            }
            // Add project cards
            for proj in p.borrow().iter() {
                let card = build_project_card(&proj.name, &proj.id, s.clone());
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
    root.connect_map(move |_| {
        if let Some(ref f) = *refresh.borrow() {
            f();
        }
    });

    scroll.add(&content);
    root.pack_start(&scroll, true, true, 0);
    root
}

fn build_project_card(name: &str, id: &str, stack: Stack) -> GtkBox {
    let card = GtkBox::new(Orientation::Vertical, 12);
    card.style_context().add_class("project-card");
    card.set_size_request(180, 150);

    let icon = Label::new(Some("📁"));
    icon.style_context().add_class("project-card-icon");
    icon.set_halign(Align::Center);
    card.pack_start(&icon, false, false, 0);

    let name_label = Label::new(Some(name));
    name_label.style_context().add_class("project-card-name");
    name_label.set_halign(Align::Center);
    name_label.set_ellipsize(pango::EllipsizeMode::End);
    name_label.set_max_width_chars(16);
    card.pack_start(&name_label, false, false, 0);

    let id_clone = id.to_string();
    let s = stack.clone();
    let event_box = gtk::EventBox::new();
    event_box.add(&card);
    event_box.connect_button_press_event(move |_, _| {
        s.set_visible_child_name("editor");
        false.into()
    });

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.pack_start(&event_box, true, true, 0);
    container
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
        let project_id = format!("proj_{}", chrono::Utc::now().timestamp());
        
        // Create project directory in sandbox
        let sandbox = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("vibi-ai")
            .join("sandbox")
            .join(dir.trim());
        std::fs::create_dir_all(&sandbox).ok();
        
        // Write a .vibi_project file
        let proj_file = sandbox.join(".vibi_project");
        let proj_data = format!("id={}\nname={}\ncreated={}\n", 
            project_id, 
            name.trim(),
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        );
        std::fs::write(&proj_file, proj_data).ok();
        
        let project = Project {
            id: project_id,
            name: name.trim().to_string(),
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

fn build_code_editor_page(stack: Stack) -> GtkBox {
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

    // Scrollable wrapper
    let editor_scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    editor_scroll.set_hexpand(true);
    editor_scroll.set_vexpand(true);
    editor_scroll.set_policy(PolicyType::Automatic, PolicyType::Automatic);

    // Main editor area: file tree + editor
    let editor_box = GtkBox::new(Orientation::Horizontal, 0);
    editor_box.set_hexpand(true);
    editor_box.set_vexpand(true);

    // File tree sidebar
    let file_tree = build_file_tree();
    editor_box.pack_start(&file_tree, false, false, 0);

    // Separator between tree and editor
    editor_box.pack_start(&Separator::new(Orientation::Vertical), false, false, 0);

    // Code editing area
    let editor_area = build_editor_area();
    editor_box.pack_start(&editor_area, true, true, 0);

    editor_scroll.add(&editor_box);
    root.pack_start(&editor_scroll, true, true, 0);
    root
}

fn build_file_tree() -> GtkBox {
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

    // Read actual sandbox directory
    let sandbox = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("vibi-ai")
        .join("sandbox");
    
    fn add_directory_entries(parent: &GtkBox, path: &std::path::Path, depth: i32) {
        if depth > 4 { return; }
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
                let indent = "  ".repeat(depth as usize);
                let icon = if is_dir { "📁" } else { "📄" };
                
                let row = GtkBox::new(Orientation::Horizontal, 4);
                row.set_margin_start(12 + depth * 12);
                row.set_margin_top(1);
                row.set_margin_bottom(1);
                let label = Label::new(Some(&format!("{} {}{}", icon, indent, name)));
                label.style_context().add_class("file-tree-item");
                label.set_halign(Align::Start);
                row.pack_start(&label, false, false, 0);
                parent.pack_start(&row, false, false, 0);
                
                if is_dir {
                    add_directory_entries(parent, &entry.path(), depth + 1);
                }
            }
        }
    }
    
    add_directory_entries(&tree_list, &sandbox, 0);

    scroll.add(&tree_list);
    container.pack_start(&scroll, true, true, 0);
    container
}
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.set_size_request(220, -1);
    container.style_context().add_class("file-tree");

    let header = Label::new(Some("📁 Files"));
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

    // Demo files
    let demo_files = vec![
        ("📄", "main.rs", "src/"),
        ("📄", "style.css", "src/ui/"),
        ("📄", "dashboard.rs", "src/ui/"),
        ("📄", "chatgpt.js", "src/agentic_detection/"),
        ("📁", "vibi_lang", ""),
        ("📄", "  lexer.rs", "src/vibi_lang/"),
        ("📄", "  parser.rs", "src/vibi_lang/"),
        ("📄", "  compiler.rs", "src/vibi_lang/"),
    ];

    for (icon, name, _path) in &demo_files {
        let row = GtkBox::new(Orientation::Horizontal, 4);
        row.set_margin_start(12);
        row.set_margin_top(2);
        row.set_margin_bottom(2);
        let label = Label::new(Some(&format!("{} {}", icon, name)));
        label.style_context().add_class("file-tree-item");
        label.set_halign(Align::Start);
        row.pack_start(&label, false, false, 0);
        tree_list.pack_start(&row, false, false, 0);
    }

    scroll.add(&tree_list);
    container.pack_start(&scroll, true, true, 0);
    container
}

fn build_editor_area() -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.set_hexpand(true);
    container.set_vexpand(true);

    // Tab bar
    let tab_bar = GtkBox::new(Orientation::Horizontal, 0);
    tab_bar.style_context().add_class("editor-tab-bar");
    let tab = Label::new(Some("  main.rs  ✕  "));
    tab.style_context().add_class("editor-tab");
    tab_bar.pack_start(&tab, false, false, 0);
    container.pack_start(&tab_bar, false, false, 0);
    container.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);

    // Line numbers + text view
    let editor_body = GtkBox::new(Orientation::Horizontal, 0);
    editor_body.set_hexpand(true);
    editor_body.set_vexpand(true);

    // Line numbers
    let line_numbers = GtkBox::new(Orientation::Vertical, 0);
    line_numbers.set_size_request(48, -1);
    line_numbers.style_context().add_class("editor-line-numbers");
    let line_scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    line_scroll.set_policy(PolicyType::Never, PolicyType::Never);
    let line_labels = GtkBox::new(Orientation::Vertical, 0);
    for i in 1..=50 {
        let num = Label::new(Some(&format!("{:>3}", i)));
        num.style_context().add_class("editor-line-num");
        num.set_halign(Align::End);
        num.set_margin_end(8);
        line_labels.pack_start(&num, false, false, 0);
    }
    line_scroll.add(&line_labels);
    line_numbers.pack_start(&line_scroll, true, true, 0);
    editor_body.pack_start(&line_numbers, false, false, 0);
    editor_body.pack_start(&Separator::new(Orientation::Vertical), false, false, 0);

    // Code text view
    let code_view = gtk::TextView::new();
    code_view.set_vexpand(true);
    code_view.set_hexpand(true);
    code_view.set_wrap_mode(gtk::WrapMode::None);
    code_view.set_monospace(true);
    code_view.set_left_margin(12);
    code_view.set_top_margin(8);

    if let Some(buffer) = code_view.buffer() {
        buffer.set_text(r#"// main.rs - VibiClaw Entry Point

mod types;
mod sandbox;
mod executor;
mod ui;
mod ai_bridge;
mod vibi_lang;
mod storage;
mod chat_store;
mod crypto;
mod logger;
mod hardware_usage;

use gtk::prelude::*;
use gtk::Application;

fn main() {
    let app = Application::builder()
        .application_id("com.vibi.claw")
        .build();

    app.connect_startup(|_| {
        load_css();
        hardware_usage::start_hardware_server();
    });
    
    app.connect_activate(ui::build_window);
    app.run();
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        include_str!("ui/style.css").as_bytes()
    ).ok();
    
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::default()
            .expect("Could not connect to display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}"#);
    }

    let code_scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    code_scroll.set_policy(PolicyType::Automatic, PolicyType::Automatic);
    code_scroll.add(&code_view);
    editor_body.pack_start(&code_scroll, true, true, 0);

    container.pack_start(&editor_body, true, true, 0);
    container
}