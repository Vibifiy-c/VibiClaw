use std::process::Command as ShellCommand;
use crate::types::{Command, CommandKind, CommandStatus};
use crate::sandbox::Sandbox;

pub struct ExecutorConfig {
    pub auto_execute: bool,  // the toggle from settings
}

pub struct Executor {
    pub sandbox: Sandbox,
    pub config: ExecutorConfig,
}

impl Executor {
    pub fn new(sandbox_path: &str, auto_execute: bool) -> std::io::Result<Self> {
        Ok(Self {
            sandbox: Sandbox::new(sandbox_path)?,
            config: ExecutorConfig { auto_execute },
        })
    }

    // Main entry — decide whether to run or queue for approval
    pub fn process(&self, commands: &mut Vec<Command>) {
        for cmd in commands.iter_mut() {
            if self.config.auto_execute {
                self.run(cmd);
            } else {
                // Leave as Pending — UI will approve/reject
                cmd.status = CommandStatus::Pending;
            }
        }
    }

    // Run a single approved command
    pub fn run(&self, cmd: &mut Command) {
        cmd.status = CommandStatus::Running;

        let result = match cmd.kind {
            CommandKind::CreateFile | CommandKind::EditFile => {
                self.run_file_write(cmd)
            }
            CommandKind::DeleteFile => {
                self.run_file_delete(cmd)
            }
            CommandKind::RunShell | CommandKind::InstallDep => {
                self.run_shell(cmd)
            }
        };

        match result {
            Ok(output) => {
                cmd.status = CommandStatus::Done;
                cmd.output = Some(output);
            }
            Err(e) => {
                cmd.status = CommandStatus::Failed(e.clone());
                cmd.output = Some(e);
            }
        }
    }

    fn run_file_write(&self, cmd: &Command) -> Result<String, String> {
        let path = cmd.path.as_deref()
            .ok_or("No path specified")?;
        let content = cmd.content.as_deref()
            .unwrap_or("");

        self.sandbox.write_file(path, content)
            .map(|_| format!("Written: {}", path))
            .map_err(|e| e.to_string())
    }

    fn run_file_delete(&self, cmd: &Command) -> Result<String, String> {
        let path = cmd.path.as_deref()
            .ok_or("No path specified")?;

        self.sandbox.delete_file(path)
            .map(|_| format!("Deleted: {}", path))
            .map_err(|e| e.to_string())
    }

    fn run_shell(&self, cmd: &Command) -> Result<String, String> {
        let command = cmd.content.as_deref()
            .ok_or("No command specified")?;

        let output = ShellCommand::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(self.sandbox.working_dir())
            .output()
            .map_err(|e| e.to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(if stdout.is_empty() { "Done".to_string() } else { stdout })
        } else {
            Err(if stderr.is_empty() { "Command failed".to_string() } else { stderr })
        }
    }
}