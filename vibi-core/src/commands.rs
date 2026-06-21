use std::sync::{Arc, Mutex};
use crate::types::{Command, CommandStatus};
use crate::analyzer::Analyzer;
use crate::executor::Executor;

pub struct AppState {
    pub executor: Executor,
    pub pending_commands: Arc<Mutex<Vec<Command>>>,
    pub auto_execute: Arc<Mutex<bool>>,
    pub sandbox_path: Arc<Mutex<String>>,
}

impl AppState {
    pub fn new(sandbox_path: &str, auto_execute: bool) -> std::io::Result<Self> {
        Ok(Self {
            executor: Executor::new(sandbox_path, auto_execute)?,
            pending_commands: Arc::new(Mutex::new(Vec::new())),
            auto_execute: Arc::new(Mutex::new(auto_execute)),
            sandbox_path: Arc::new(Mutex::new(sandbox_path.to_string())),
        })
    }

    // Called when AI response arrives
    pub fn handle_response(&self, response: &str) -> Vec<Command> {
        let mut commands = Analyzer::analyze(response);
        let auto = *self.auto_execute.lock().unwrap();

        if auto {
            self.executor.process(&mut commands);
        } else {
            // Queue them as Pending for user approval
            let mut pending = self.pending_commands.lock().unwrap();
            pending.extend(commands.clone());
        }

        commands
    }

    // Called when user approves a command by ID
    pub fn approve(&self, id: &str) -> Option<Command> {
        let mut pending = self.pending_commands.lock().unwrap();
        if let Some(pos) = pending.iter().position(|c| c.id == id) {
            let mut cmd = pending.remove(pos);
            drop(pending); // release lock before running
            self.executor.run(&mut cmd);
            return Some(cmd);
        }
        None
    }

    // Called when user rejects a command by ID
    pub fn reject(&self, id: &str) -> bool {
        let mut pending = self.pending_commands.lock().unwrap();
        if let Some(pos) = pending.iter().position(|c| c.id == id) {
            pending[pos].status = CommandStatus::Rejected;
            return true;
        }
        false
    }

    // Toggle auto-execute at runtime
    pub fn set_auto_execute(&self, value: bool) {
        *self.auto_execute.lock().unwrap() = value;
    }

    // Change sandbox path at runtime
    pub fn set_sandbox_path(&self, path: &str) {
        *self.sandbox_path.lock().unwrap() = path.to_string();
    }

    pub fn get_pending(&self) -> Vec<Command> {
        self.pending_commands.lock().unwrap().clone()
    }
}