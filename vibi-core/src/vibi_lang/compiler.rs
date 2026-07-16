use super::parser::{VibiProgram, Tool};
use crate::types::{Command, CommandKind};

pub fn compile(program: VibiProgram) -> Result<Vec<Command>, Vec<String>> {
    let mut commands = Vec::new();
    
    for job in &program.jobs {
        let cmd = tool_to_command(&job.tool);
        commands.push(cmd);
        
        for cond in &job.conditions {
            for body_job in &cond.body {
                commands.push(tool_to_command(&body_job.tool));
            }
        }
    }
    
    Ok(commands)
}

fn tool_to_command(tool: &Tool) -> Command {
    match tool {
        Tool::CreateFile { name, path } => Command::new(
            CommandKind::CreateFile,
            Some(resolve_path(name, path.as_deref())),
            None,
        ),
        Tool::SaveFile { name, path, content } => Command::new(
            CommandKind::CreateFile,
            Some(resolve_path(name, path.as_deref())),
            content.clone(),
        ),
        Tool::DeleteFile { name, path } => Command::new(
            CommandKind::DeleteFile,
            Some(resolve_path(name, path.as_deref())),
            None,
        ),
        Tool::CreateFolder { name, path } => Command::new(
            CommandKind::CreateFolder,
            Some(resolve_path(name, path.as_deref())),
            None,
        ),
        Tool::SaveFolder { name, path, .. } => Command::new(
            CommandKind::CreateFolder,
            Some(resolve_path(name, path.as_deref())),
            None,
        ),
        Tool::DeleteFolder { name, path } => Command::new(
            CommandKind::DeleteFolder,
            Some(resolve_path(name, path.as_deref())),
            None,
        ),
        Tool::RunCommand { command, dir: _ } => Command::new(
            CommandKind::RunShell,
            None,
            Some(command.clone()),
        ),
        Tool::EditFile { name, path, content, .. } => Command::new(
            CommandKind::EditFile,
            Some(resolve_path(name, path.as_deref())),
            content.clone(),
        ),
        Tool::InstallDep { dependency, .. } => Command::new(
            CommandKind::InstallDep,
            None,
            Some(dependency.clone()),
        ),
        Tool::DeleteDep { dependency, .. } => Command::new(
            CommandKind::RunShell,
            None,
            Some(format!("cargo remove {}", dependency)),
        ),
    }
}

fn resolve_path(name: &str, path: Option<&str>) -> String {
    match path {
        Some(p) if !p.is_empty() => format!("{}/{}", p, name),
        _ => name.to_string(),
    }
}