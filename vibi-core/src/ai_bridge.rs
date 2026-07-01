use webkit2gtk::{WebView, WebViewExt};
use gtk::prelude::*;
use gio;
use std::rc::Rc;
use std::cell::RefCell;

pub struct AiBridge {
    pub webview: WebView,
    pub model: String,
    response_callback: Rc<RefCell<Option<Box<dyn Fn(String)>>>>,
    page_loaded: Rc<RefCell<bool>>,
}

impl AiBridge {
        pub fn from_webview(webview: WebView, model: &str) -> Self {
        let bridge = AiBridge {
            webview: webview.clone(),
            model: model.to_string(),
            response_callback: Rc::new(RefCell::new(None)),
            page_loaded: Rc::new(RefCell::new(true)), // already loaded in notebook
        };
        
        let cb = bridge.response_callback.clone();
        webview.connect_uri_notify(move |wv| {
            if let Some(uri) = wv.uri() {
                let uri_str = uri.to_string();
                if let Some(hash_pos) = uri_str.find("#vibi-") {
                    let hex = &uri_str[hash_pos + "#vibi-".len()..];
                    let hex = hex.split('&').next().unwrap_or(hex).split('?').next().unwrap_or(hex);
                    if let Ok(text) = hex_decode(hex) {
                        if !text.is_empty() {
                            if let Some(ref callback) = *cb.borrow() {
                                callback(text);
                            }
                        }
                    }
                }
            }
        });
        
        bridge.inject_observer();
        
        bridge
    }
    pub fn new(model: &str) -> Self {
        let webview = WebView::new();
        webview.set_size_request(100, 100);
        webview.set_opacity(0.01);
        
        let bridge = AiBridge {
            webview: webview.clone(),
            model: model.to_string(),
            response_callback: Rc::new(RefCell::new(None)),
            page_loaded: Rc::new(RefCell::new(false)),
        };
        
        let cb = bridge.response_callback.clone();
        webview.connect_uri_notify(move |wv| {
            if let Some(uri) = wv.uri() {
                let uri_str = uri.to_string();
                if let Some(hash_pos) = uri_str.find("#vibi-") {
                    let hex = &uri_str[hash_pos + "#vibi-".len()..];
                    let hex = hex.split('&').next().unwrap_or(hex).split('?').next().unwrap_or(hex);
                    if let Ok(text) = hex_decode(hex) {
                        if !text.is_empty() {
                            println!("[AiBridge] Response: {}", &text[..text.len().min(100)]);
                            if let Some(ref callback) = *cb.borrow() {
                                callback(text);
                            }
                        }
                    }
                }
            }
        });
        
        let url = match model {
            "chatgpt" => "https://chat.openai.com",
            "claude" => "https://claude.ai",
            "gemini" => "https://gemini.google.com",
            "deepseek" => "https://chat.deepseek.com",
            "grok" => "https://grok.com",
            "qwen" => "https://tongyi.aliyun.com/qianwen",
            "kimi" => "https://kimi.moonshot.cn",
            _ => "https://chat.openai.com",
        };
        
        webview.load_uri(url);
        
        let loaded = bridge.page_loaded.clone();
        let wv_vis = webview.clone();
        webview.connect_load_changed(move |_, event| {
            if event == webkit2gtk::LoadEvent::Finished {
                *loaded.borrow_mut() = true;
                wv_vis.run_javascript(
                    "Object.defineProperty(document, 'hidden', {value:false,writable:false}); Object.defineProperty(document, 'visibilityState', {value:'visible',writable:false});",
                    None::<&gio::Cancellable>, |_| {}
                );
            }
        });
        
        bridge.inject_observer();
        
        bridge
    }
    
    fn inject_observer(&self) {
        let visibility_js = r#"
            Object.defineProperty(document, 'hidden', { value: false, writable: false });
            Object.defineProperty(document, 'visibilityState', { value: 'visible', writable: false });
            Object.defineProperty(document, 'webkitHidden', { value: false, writable: false });
            Object.defineProperty(document, 'webkitVisibilityState', { value: 'visible', writable: false });
        "#;
        let js = include_str!("ui/inject/chatgpt.js");
        let full_js = format!("{}\n{}", visibility_js, js);
        let wv = self.webview.clone();
        self.webview.connect_load_changed(move |_, event| {
            if event == webkit2gtk::LoadEvent::Finished {
                wv.run_javascript(&full_js, None::<&gio::Cancellable>, |_| {});
            }
        });
    }
    
       pub fn send_message(&self, text: &str) {
        println!("[AiBridge] Sending to {}: {}", self.model, text);
        let escaped = text.replace('\\', "\\\\").replace('\'', "\\'").replace('\n', "\\n").replace('"', "\\\"");
        
        let js = match self.model.as_str() {
            "chatgpt" => format!(
                "(function() {{ var input = document.querySelector('#prompt-textarea') || document.querySelector('[contenteditable=\"true\"]'); if(input) {{ if(input.tagName === 'DIV' || input.contentEditable === 'true') {{ input.textContent = '{}'; }} else {{ input.value = '{}'; }} input.dispatchEvent(new Event('input', {{ bubbles: true }})); setTimeout(function() {{ var btn = document.querySelector('[data-testid=\"send-button\"]') || document.querySelector('button[type=\"submit\"]'); if(btn) btn.click(); }}, 500); }} }})()",
                escaped, escaped
            ),
            "claude" => format!(
                "(function() {{ var input = document.querySelector('[contenteditable=\"true\"]') || document.querySelector('textarea'); if(input) {{ input.textContent = '{}'; input.dispatchEvent(new Event('input', {{ bubbles: true }})); setTimeout(function() {{ var btn = document.querySelector('button[type=\"submit\"]') || document.querySelector('[aria-label=\"Send Message\"]'); if(btn) btn.click(); }}, 500); }} }})()",
                escaped
            ),
            "gemini" => format!(
                "(function() {{ var input = document.querySelector('rich-textarea') || document.querySelector('[contenteditable=\"true\"]') || document.querySelector('textarea'); if(input) {{ if(input.tagName === 'DIV' || input.contentEditable === 'true') {{ input.textContent = '{}'; }} else {{ input.value = '{}'; }} input.dispatchEvent(new Event('input', {{ bubbles: true }})); setTimeout(function() {{ var btn = document.querySelector('button[type=\"submit\"]') || document.querySelector('[aria-label=\"Send\"]'); if(btn) btn.click(); }}, 500); }} }})()",
                escaped, escaped
            ),
            "deepseek" => format!(
                "(function() {{ var input = document.querySelector('#chat-input') || document.querySelector('textarea') || document.querySelector('[contenteditable=\"true\"]'); if(input) {{ if(input.tagName === 'DIV' || input.contentEditable === 'true') {{ input.textContent = '{}'; }} else {{ input.value = '{}'; }} input.dispatchEvent(new Event('input', {{ bubbles: true }})); setTimeout(function() {{ var btn = document.querySelector('.send-btn') || document.querySelector('button[type=\"submit\"]'); if(btn) btn.click(); }}, 500); }} }})()",
                escaped, escaped
            ),
            "grok" => format!(
                "(function() {{ var input = document.querySelector('textarea') || document.querySelector('[contenteditable=\"true\"]'); if(input) {{ if(input.tagName === 'DIV' || input.contentEditable === 'true') {{ input.textContent = '{}'; }} else {{ input.value = '{}'; }} input.dispatchEvent(new Event('input', {{ bubbles: true }})); setTimeout(function() {{ var btn = document.querySelector('button[type=\"submit\"]'); if(btn) btn.click(); }}, 500); }} }})()",
                escaped, escaped
            ),
            "qwen" => format!(
                "(function() {{ var input = document.querySelector('textarea') || document.querySelector('[contenteditable=\"true\"]'); if(input) {{ if(input.tagName === 'DIV' || input.contentEditable === 'true') {{ input.textContent = '{}'; }} else {{ input.value = '{}'; }} input.dispatchEvent(new Event('input', {{ bubbles: true }})); setTimeout(function() {{ var btn = document.querySelector('button[type=\"submit\"]') || document.querySelector('.send-btn'); if(btn) btn.click(); }}, 500); }} }})()",
                escaped, escaped
            ),
            "kimi" => format!(
                "(function() {{ var input = document.querySelector('textarea') || document.querySelector('[contenteditable=\"true\"]'); if(input) {{ if(input.tagName === 'DIV' || input.contentEditable === 'true') {{ input.textContent = '{}'; }} else {{ input.value = '{}'; }} input.dispatchEvent(new Event('input', {{ bubbles: true }})); setTimeout(function() {{ var btn = document.querySelector('button[type=\"submit\"]') || document.querySelector('.send-btn'); if(btn) btn.click(); }}, 500); }} }})()",
                escaped, escaped
            ),
            _ => format!(
                "(function() {{ var input = document.querySelector('textarea, [contenteditable=\"true\"]'); if(input) {{ if(input.tagName === 'DIV' || input.contentEditable === 'true') {{ input.textContent = '{}'; }} else {{ input.value = '{}'; }} input.dispatchEvent(new Event('input', {{ bubbles: true }})); }} }})()",
                escaped, escaped
            ),
        };
        
        let wv = self.webview.clone();
        let loaded = self.page_loaded.clone();
        if *loaded.borrow() {
            wv.run_javascript(&js, None::<&gio::Cancellable>, |_| {});
        } else {
            let wv = wv.clone();
            let js = js.clone();
            let loaded = loaded.clone();
            gtk::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                if *loaded.borrow() {
                    wv.run_javascript(&js, None::<&gio::Cancellable>, |_| {});
                    gtk::glib::ControlFlow::Break
                } else {
                    gtk::glib::ControlFlow::Continue
                }
            });
        }
    }
    
    pub fn on_response<F: Fn(String) + 'static>(&self, callback: F) {
        *self.response_callback.borrow_mut() = Some(Box::new(callback));
    }
}

fn hex_decode(hex: &str) -> Result<String, ()> {
    if hex.len() % 2 != 0 { return Err(()); }
    let bytes: Result<Vec<u8>, _> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i+2], 16).map_err(|_| ()))
        .collect();
    bytes.and_then(|b| String::from_utf8(b).map_err(|_| ()))
}