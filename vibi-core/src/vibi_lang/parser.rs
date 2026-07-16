use super::lexer::Token;
use crate::types::{Command, CommandKind};

#[derive(Debug)]
pub struct VibiProgram {
    pub jobs: Vec<Job>,
    pub prints: Vec<String>,
    pub variables: Vec<String>,
}

#[derive(Debug)]
pub struct Job {
    pub tool: Tool,
    pub conditions: Vec<Condition>,
}

#[derive(Debug)]
pub enum Tool {
    CreateFile { name: String, path: Option<String> },
    SaveFile { name: String, path: Option<String>, content: Option<String> },
    DeleteFile { name: String, path: Option<String> },
    CreateFolder { name: String, path: Option<String> },
    SaveFolder { name: String, path: Option<String> },
    DeleteFolder { name: String, path: Option<String> },
    RunCommand { command: String, dir: Option<String> },
    EditFile { name: String, path: Option<String>, content: Option<String>, search: Option<String>, replace: Option<String> },
    InstallDep { dependency: String, path: Option<String> },
    DeleteDep { dependency: String, path: Option<String> },
}

#[derive(Debug)]
pub struct Condition {
    pub left: String,
    pub operator: String,
    pub right: String,
    pub body: Vec<Job>,
    pub else_body: Vec<Job>,
}

pub fn parse(tokens: Vec<Token>) -> Result<VibiProgram, Vec<String>> {
    let mut program = VibiProgram {
        jobs: Vec::new(),
        prints: Vec::new(),
        variables: Vec::new(),
    };
    
    let mut iter = tokens.iter().peekable();
    let mut errors = Vec::new();
    
    // Expect <vibi.claw>
    match iter.next() {
        Some(Token::RootOpen) => {}
        _ => errors.push("Missing <vibi.claw> root tag".to_string()),
    }
    
    while let Some(token) = iter.next() {
        match token {
            Token::JobsOpen => {
                program.jobs = parse_jobs(&mut iter);
            }
            Token::Print(parts) => {
                program.prints.push(parts.join(""));
            }
            Token::VarDecl(names) => {
                program.variables = names.clone();
            }
            Token::If => {
                // Parse condition and add to last job
                if let Some(cond) = parse_condition(&mut iter) {
                    if let Some(last_job) = program.jobs.last_mut() {
                        last_job.conditions.push(cond);
                    }
                }
            }
            Token::Eof => break,
            _ => {}
        }
    }
    
    if errors.is_empty() {
        Ok(program)
    } else {
        Err(errors)
    }
}

fn parse_jobs(iter: &mut std::iter::Peekable<std::slice::Iter<Token>>) -> Vec<Job> {
    let mut jobs = Vec::new();
    
    while let Some(token) = iter.next() {
        match token {
            Token::JobsClose => break,
            Token::ToolFamilyOpen => {
                jobs.extend(parse_tool_family(iter));
            }
            Token::ToolCallOpen { tool, params } => {
                jobs.push(Job {
                    tool: params_to_tool(tool, params),
                    conditions: Vec::new(),
                });
            }
            Token::Eof => break,
            _ => {}
        }
    }
    
    jobs
}

fn parse_tool_family(iter: &mut std::iter::Peekable<std::slice::Iter<Token>>) -> Vec<Job> {
    let mut jobs = Vec::new();
    
    while let Some(token) = iter.next() {
        match token {
            Token::ToolFamilyClose => break,
            Token::ToolCallOpen { tool, params } => {
                jobs.push(Job {
                    tool: params_to_tool(tool, params),
                    conditions: Vec::new(),
                });
            }
            Token::Eof => break,
            _ => {}
        }
    }
    
    jobs
}

fn params_to_tool(tool: &str, params: &[(String, String)]) -> Tool {
    let mut map = std::collections::HashMap::new();
    for (k, v) in params {
        map.insert(k.clone(), v.clone());
    }
    
    // First unnamed param (key="") is the name
    let name = params.first()
        .filter(|(k, _)| k.is_empty())
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    
    let get = |key: &str| map.get(key).cloned();
    
    println!("[Parser] Tool: '{}', name: '{}', params: {:?}", tool, name, params);
    
    match tool {
        "create.file" => Tool::CreateFile {
            name,
            path: get("path"),
        },
        "save.file" => Tool::SaveFile {
            name: name.clone(),
            path: get("path"),
            content: get("content"),
        },
        "delete.file" => Tool::DeleteFile {
            name: name.clone(),
            path: get("path"),
        },
        "create.folder" => Tool::CreateFolder {
            name: name.clone(),
            path: get("path"),
        },
        "save.folder" => Tool::SaveFolder {
            name: name.clone(),
            path: get("path"),
        },
        "delete.folder" => Tool::DeleteFolder {
            name: name.clone(),
            path: get("path"),
        },
        "run.command" => Tool::RunCommand {
            command: name.clone(),
            dir: get("dir"),
        },
        "edit.file" => Tool::EditFile {
            name: name.clone(),
            path: get("path"),
            content: get("content"),
            search: get("search"),
            replace: get("replace"),
        },
        "install.dependencies" => Tool::InstallDep {
            dependency: name.clone(),
            path: get("install?path").or_else(|| get("path")),
        },
        "delete.dependencies" => Tool::DeleteDep {
            dependency: name.clone(),
            path: get("delete?path").or_else(|| get("path")),
        },
        _ => Tool::RunCommand {
            command: format!("unknown tool: {}", tool),
            dir: None,
        },
    }
}

fn parse_condition(iter: &mut std::iter::Peekable<std::slice::Iter<Token>>) -> Option<Condition> {
    let mut left = String::new();
    let mut operator = String::new();
    let mut right = String::new();
    let mut body = Vec::new();
    let mut else_body = Vec::new();
    let mut in_else = false;
    
    // Parse: {condition}={value}
    while let Some(token) = iter.next() {
        match token {
            Token::BlockOpen => {
                // Parse body until } or else
                while let Some(t) = iter.next() {
                    match t {
                        Token::BlockClose => break,
                        Token::Else => { in_else = true; }
                        Token::ElseIf => { in_else = true; }
                        Token::ToolCallOpen { tool, params } => {
                            let job = Job {
                                tool: params_to_tool(tool, params),
                                conditions: Vec::new(),
                            };
                            if in_else { else_body.push(job); } else { body.push(job); }
                        }
                        _ => {}
                    }
                }
                break;
            }
            Token::Semicolon => {
                // End of condition header: if {left}={right};
                break;
            }
            _ => {
                // Collect condition tokens
                left = format!("{}", token_to_string(token));
            }
        }
    }
    
    // Parse body inside { }
    while let Some(token) = iter.next() {
        match token {
            Token::ToolCallOpen { tool, params } => {
                let job = Job {
                    tool: params_to_tool(tool, params),
                    conditions: Vec::new(),
                };
                if in_else { else_body.push(job); } else { body.push(job); }
            }
            Token::Else => in_else = true,
            Token::BlockClose => break,
            Token::Eof => break,
            _ => {}
        }
    }
    
    Some(Condition {
        left,
        operator,
        right,
        body,
        else_body,
    })
}

fn token_to_string(token: &Token) -> String {
    match token {
        Token::Variable(name) => name.clone(),
        _ => format!("{:?}", token),
    }
}