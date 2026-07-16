#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    RootOpen,                          // <vibi.claw>
    JobsOpen,                          // <jobs>
    JobsClose,                         // </jobs>
    ToolFamilyOpen,                    // <vibi.tool>
    ToolFamilyClose,                   // </vibi.tool>
    ToolCallOpen {                     // <vibi.tool?=create.file="hello.rs",?path="src">
        tool: String,
        params: Vec<(String, String)>,
    },
    ToolCallClose(String),             // </vibi.tool?=create.file>
    BlockOpen,                         // {
    BlockClose,                        // }
    If,                                // if
    Else,                              // else
    ElseIf,                            // else if
    Print(Vec<String>),                // #print("msg")+("more")
    VarDecl(Vec<String>),              // vibi.variable, {name1, name2}
    Variable(String),                  // variable.{name}
    Semicolon,                         // ;
    Eof,
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, Vec<String>> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    let mut errors = Vec::new();
    
    while let Some(&ch) = chars.peek() {
        match ch {
            '<' => {
                chars.next();
                let tag = read_until(&mut chars, '>');
                tokens.push(parse_tag(&tag));
            }
            '{' => {
                chars.next();
                tokens.push(Token::BlockOpen);
            }
            '}' => {
                chars.next();
                tokens.push(Token::BlockClose);
            }
            '#' => {
                chars.next();
                if peek_word(&mut chars) == "print" {
                    chars.nth(4); // skip "print"
                    tokens.push(parse_print(&mut chars));
                }
            }
            'v' => {
                if peek_word(&mut chars) == "vibi.variable" {
                    chars.nth(12); // skip "vibi.variable"
                    tokens.push(parse_var_decl(&mut chars));
                } else {
                    chars.next();
                }
            }
            'i' => {
                let word = peek_word(&mut chars);
                if word == "if" {
                    chars.nth(1);
                    tokens.push(Token::If);
                } else {
                    chars.next();
                }
            }
            'e' => {
                let word = peek_word(&mut chars);
                if word == "else" {
                    chars.nth(4);
                    if peek_word(&mut chars).starts_with("if") {
                        chars.nth(2);
                        tokens.push(Token::ElseIf);
                    } else {
                        tokens.push(Token::Else);
                    }
                } else {
                    chars.next();
                }
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }
            ' ' | '\n' | '\r' | '\t' | ',' => {
                chars.next(); // skip whitespace
            }
            _ => {
                // Skip unknown chars for now
                chars.next();
            }
        }
    }
    
    tokens.push(Token::Eof);
    
    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}

fn read_until(chars: &mut std::iter::Peekable<std::str::Chars>, end: char) -> String {
    let mut result = String::new();
    let mut in_quotes = false;
    while let Some(&ch) = chars.peek() {
        if ch == '"' {
            in_quotes = !in_quotes;
        }
        if ch == end && !in_quotes {
            chars.next();
            break;
        }
        result.push(ch);
        chars.next();
    }
    result
}

fn peek_word(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut word = String::new();
    let mut clone = chars.clone();
    while let Some(&ch) = clone.peek() {
        if ch.is_alphanumeric() || ch == '.' || ch == '_' {
            word.push(ch);
            clone.next();
        } else {
            break;
        }
    }
    word
}

fn parse_tag(tag: &str) -> Token {
    let tag = tag.trim();
    
    if tag == "vibi.claw" {
        return Token::RootOpen;
    }
    if tag == "jobs" {
        return Token::JobsOpen;
    }
    if tag == "/jobs" {
        return Token::JobsClose;
    }
    if tag == "vibi.tool" {
        return Token::ToolFamilyOpen;
    }
    if tag == "/vibi.tool" {
        return Token::ToolFamilyClose;
    }
    
    // Tool call: vibi.tool?=create.file="hello.rs",?path="src"
    if tag.starts_with("vibi.tool?=") {
        let rest = &tag["vibi.tool?=".len()..];
        return parse_tool_call(rest, false);
    }
    
    // Tool close: /vibi.tool?=create.file
    if tag.starts_with("/vibi.tool?=") {
        let tool_name = tag["/vibi.tool?=".len()..].trim().to_string();
        return Token::ToolCallClose(tool_name);
    }
    
    Token::Semicolon // fallback
}

fn parse_tool_call(input: &str, _is_close: bool) -> Token {
    let mut params = Vec::new();
    let mut tool_name = String::new();
    
    // Input format: create.file="hello.rs",?path="src"
    // Split by comma, but respect quotes
    let parts = split_params(input);
    
    for (i, part) in parts.iter().enumerate() {
        let part = part.trim();
        if i == 0 {
            // First part: tool=value or just value
            if let Some(eq_pos) = part.find('=') {
                let key = part[..eq_pos].trim();
                let value = part[eq_pos + 1..].trim().trim_matches('"');
                if key.is_empty() {
                    tool_name = value.to_string();
                } else {
                    tool_name = key.to_string();
                    params.push((String::new(), value.to_string()));
                }
            } else {
                tool_name = part.to_string();
            }
        } else if let Some(eq_pos) = part.find('=') {
            let key = part[..eq_pos].trim().trim_start_matches('?').to_string();
            let value = part[eq_pos + 1..].trim().trim_matches('"').to_string();
            params.push((key, value));
        }
    }
    
    println!("[Lexer] Parsed tool: '{}' with {} params", tool_name, params.len());
    
    Token::ToolCallOpen {
        tool: tool_name,
        params,
    }
}

fn split_params(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    
    for ch in input.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            ',' if !in_quotes => {
                parts.push(current.clone());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

fn parse_print(chars: &mut std::iter::Peekable<std::str::Chars>) -> Token {
    let mut parts = Vec::new();
    
    while chars.peek().is_some() {
        if chars.peek() == Some(&'(') {
            chars.next();
            let msg = read_until(chars, ')');
            parts.push(msg.trim_matches('"').to_string());
        } else if chars.peek() == Some(&'+') {
            chars.next();
        } else if chars.peek() == Some(&';') {
            chars.next();
            break;
        } else {
            chars.next();
        }
    }
    
    Token::Print(parts)
}

fn parse_var_decl(chars: &mut std::iter::Peekable<std::str::Chars>) -> Token {
    let mut names = Vec::new();
    
    // Skip whitespace and comma
    while chars.peek().is_some() {
        match chars.peek() {
            Some(&'{') => {
                chars.next();
                let content = read_until(chars, '}');
                for name in content.split(',') {
                    let name = name.trim();
                    if !name.is_empty() {
                        names.push(name.to_string());
                    }
                }
            }
            Some(&';') => {
                chars.next();
                break;
            }
            Some(&',') | Some(&' ') => {
                chars.next();
            }
            _ => {
                chars.next();
            }
        }
    }
    
    Token::VarDecl(names)
}