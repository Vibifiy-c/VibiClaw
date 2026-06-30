use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub model: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatSession {
    pub fn new(id: String, first_message: &str, model: &str) -> Self {
        let title = generate_title(first_message);
        ChatSession {
            id,
            title,
            model: model.to_string(),
            messages: Vec::new(),
        }
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
        });
    }
}

pub struct ChatStore {
    chats: Vec<ChatSession>,
    active_id: Option<String>,
}

impl ChatStore {
    pub fn load() -> Self {
        let path = chats_path();
        let chats = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|data| serde_json::from_str(&data).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        ChatStore {
            chats,
            active_id: None,
        }
    }

    pub fn save(&self) {
        let path = chats_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.chats) {
            fs::write(&path, json).ok();
        }
    }

    pub fn create_chat(&mut self, first_message: &str, model: &str) -> &ChatSession {
        let id = format!("chat_{}", chrono::Utc::now().timestamp_millis());
        let chat = ChatSession::new(id.clone(), first_message, model);
        self.chats.insert(0, chat);
        self.active_id = Some(id.clone());
        self.save();
        self.chats.first().unwrap()
    }

    pub fn get_active(&self) -> Option<&ChatSession> {
        self.active_id.as_ref().and_then(|id| self.chats.iter().find(|c| &c.id == id))
    }

    pub fn get_active_mut(&mut self) -> Option<&mut ChatSession> {
        self.active_id.as_ref().and_then(|id| self.chats.iter_mut().find(|c| &c.id == id))
    }

    pub fn set_active(&mut self, id: &str) {
        if id.is_empty() {
            self.active_id = None;
        } else {
            self.active_id = Some(id.to_string());
        }
    }

    pub fn all_chats(&self) -> &[ChatSession] {
        &self.chats
    }

    pub fn add_message_to_active(&mut self, role: &str, content: &str) {
        if let Some(chat) = self.get_active_mut() {
            chat.add_message(role, content);
            self.save();
        }
    }

    pub fn delete_chat(&mut self, id: &str) {
        self.chats.retain(|c| c.id != id);
        if self.active_id.as_deref() == Some(id) {
            self.active_id = None;
        }
        self.save();
    }

    pub fn load_chat_messages(&self, id: &str) -> Option<Vec<ChatMessage>> {
        self.chats.iter().find(|c| c.id == id).map(|c| c.messages.clone())
    }
}

fn generate_title(text: &str) -> String {
    let cleaned = text.trim();
    if cleaned.len() <= 40 {
        cleaned.to_string()
    } else {
        let end = cleaned[..40].rfind(' ').unwrap_or(40);
        format!("{}...", &cleaned[..end])
    }
}

fn chats_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("vibi-ai");
    path.push("chats.json");
    path
}