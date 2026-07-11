use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, Orientation, Align, ScrolledWindow, PolicyType, Separator};
use pulldown_cmark::{Parser, Event, Tag, TagEnd, HeadingLevel, CodeBlockKind};

pub fn render_markdown(markdown: &str) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 4);
    container.set_hexpand(true);
    
    let parser = Parser::new(markdown);
    let mut in_code_block = false;
    let mut code_buffer = String::new();
    let mut code_language = String::new();
    let mut in_paragraph = false;
    let mut paragraph_text = String::new();
    
    for event in parser {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::Heading { level, .. } => {
                        finish_paragraph(&mut paragraph_text, &mut in_paragraph, &container);
                    }
                    Tag::CodeBlock(kind) => {
                        finish_paragraph(&mut paragraph_text, &mut in_paragraph, &container);
                        in_code_block = true;
                        code_buffer.clear();
                        code_language = match kind {
                            CodeBlockKind::Fenced(lang) => lang.to_string(),
                            _ => String::new(),
                        };
                    }
                    Tag::Paragraph => {
                        in_paragraph = true;
                        paragraph_text.clear();
                    }
                    _ => {}
                }
            }
            Event::End(tag) => {
                match tag {
                    TagEnd::Heading(level) => {
                        let label = Label::new(None);
                        let size = match level {
                            HeadingLevel::H1 => 24,
                            HeadingLevel::H2 => 20,
                            HeadingLevel::H3 => 18,
                            HeadingLevel::H4 => 16,
                            _ => 14,
                        };
                        label.set_markup(&format!(
                            "<span font_weight=\"bold\" size=\"{}\">{}</span>",
                            size * 1000,
                            glib::markup_escape_text(&paragraph_text)
                        ));
                        label.set_halign(Align::Start);
                        label.set_wrap(true);
                        label.set_margin_top(8);
                        label.set_margin_bottom(4);
                        container.pack_start(&label, false, false, 0);
                        paragraph_text.clear();
                    }
                    TagEnd::CodeBlock => {
                        in_code_block = false;
                        let code_view = create_code_block(&code_buffer, &code_language);
                        container.pack_start(&code_view, false, false, 0);
                        code_buffer.clear();
                    }
                    TagEnd::Paragraph => {
                        finish_paragraph(&mut paragraph_text, &mut in_paragraph, &container);
                    }
                    _ => {}
                }
            }
            Event::Text(text) => {
                if in_code_block {
                    code_buffer.push_str(&text);
                } else {
                    paragraph_text.push_str(&text);
                }
            }
            Event::Code(code) => {
                paragraph_text.push_str(&format!("<tt>{}</tt>", glib::markup_escape_text(&code)));
            }
            Event::SoftBreak => {
                paragraph_text.push(' ');
            }
            Event::HardBreak => {
                paragraph_text.push('\n');
            }
            _ => {}
        }
    }
    
    finish_paragraph(&mut paragraph_text, &mut in_paragraph, &container);
    container
}

fn finish_paragraph(text: &mut String, in_para: &mut bool, container: &GtkBox) {
    if *in_para && !text.is_empty() {
        let label = Label::new(None);
        label.set_markup(&glib::markup_escape_text(text));
        label.set_halign(Align::Start);
        label.set_wrap(true);
        label.set_margin_bottom(2);
        container.pack_start(&label, false, false, 0);
        text.clear();
        *in_para = false;
    }
}

fn create_code_block(code: &str, _language: &str) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.style_context().add_class("code-block");
    container.set_margin_top(8);
    container.set_margin_bottom(8);
    
    let header = GtkBox::new(Orientation::Horizontal, 0);
    header.style_context().add_class("code-block-header");
    header.set_margin_start(8);
    header.set_margin_end(8);
    header.set_margin_top(4);
    header.set_margin_bottom(4);
    
    let lang_label = Label::new(Some(if _language.is_empty() { "Code" } else { _language }));
    lang_label.style_context().add_class("code-block-lang");
    lang_label.set_halign(Align::Start);
    header.pack_start(&lang_label, true, true, 0);
    
    container.pack_start(&header, false, false, 0);
    container.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);
    
    let scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroll.set_policy(PolicyType::Automatic, PolicyType::Automatic);
    scroll.set_max_content_height(300);
    
    let code_label = Label::new(Some(code));
    code_label.set_halign(Align::Start);
    code_label.set_wrap(false);
    code_label.set_margin_start(12);
    code_label.set_margin_end(12);
    code_label.set_margin_top(8);
    code_label.set_margin_bottom(8);
    code_label.set_selectable(true);
    code_label.style_context().add_class("code-block-text");
    
    scroll.add(&code_label);
    container.pack_start(&scroll, false, false, 0);
    
    container
}