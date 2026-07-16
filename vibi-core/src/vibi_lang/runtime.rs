use crate::executor::Executor;
use crate::types::{Command, CommandStatus};

pub fn execute(mut commands: Vec<Command>, executor: &Executor) -> Vec<String> {
    let mut results = Vec::new();
    
    for cmd in &mut commands {
        executor.run(cmd);
        match &cmd.status {
            CommandStatus::Done => {
                let output = cmd.output.as_deref().unwrap_or("Done");
                results.push(format!("✅ {}", output));
            }
            CommandStatus::Failed(e) => {
                results.push(format!("❌ {}", e));
            }
            _ => {
                results.push(format!("⚠️ {:?}", cmd.status));
            }
        }
    }
    
    results
}