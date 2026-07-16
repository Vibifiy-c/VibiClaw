use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppStorage {
    pub theme: String,
    pub sidebar_collapsed: bool,
    pub projects: Vec<StoredProject>,
    pub api_mode: bool,
    pub openai_api_key: Option<String>,
    pub gemini_api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoredProject {
    pub id: String,
    pub name: String,
    pub category: String,
    pub file_count: usize,
    pub github_link: Option<String>,
    pub files_path: String,
}

impl AppStorage {
    pub fn load() -> Self {
        let path = storage_path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(storage) = serde_json::from_str(&data) {
                    return storage;
                }
            }
        }
        AppStorage {
            theme: "light".to_string(),
            sidebar_collapsed: false,
            projects: Vec::new(),
            api_mode: false,
            openai_api_key: None,
            gemini_api_key: None,
        }
    }

    pub fn save(&self) {
        let path = storage_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            fs::write(&path, json).ok();
        }
    }

    pub fn set_theme(&mut self, theme: &str) {
        self.theme = theme.to_string();
        self.save();
    }

    pub fn set_sidebar_collapsed(&mut self, collapsed: bool) {
        self.sidebar_collapsed = collapsed;
        self.save();
    }

    pub fn add_project(&mut self, project: StoredProject) {
        self.projects.push(project);
        self.save();
    }

    pub fn remove_project(&mut self, id: &str) {
        self.projects.retain(|p| p.id != id);
        self.save();
    }
}

fn storage_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("vibi-ai");
    path.push("storage.json");
    path
}

pub fn projects_dir() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("vibi-ai");
    path.push("projects");
    path
}

pub fn save_project_files(id: &str, files: &std::collections::HashMap<String, String>) {
    let dir = projects_dir().join(id);
    fs::create_dir_all(&dir).ok();
    for (name, content) in files {
        let file_path = dir.join(name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&file_path, content).ok();
    }
}

pub fn load_project_files(id: &str) -> Option<std::collections::HashMap<String, String>> {
    let dir = projects_dir().join(id);
    if !dir.exists() {
        return None;
    }
    let mut files = std::collections::HashMap::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        files.insert(name.to_string(), content);
                    }
                }
            }
        }
    }
    Some(files)
}