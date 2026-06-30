use webkit2gtk::{WebView, WebViewExt};
use gtk::prelude::*;
use gio;
use std::rc::Rc;
use std::cell::RefCell;

pub struct AiBridge {
    pub webview: WebView,
    pub model: String,
    last_response: Rc<RefCell<String>>,
    response_callback: Rc<RefCell<Option<Box<dyn Fn(String)>>>>,
}

impl AiBridge {
    pub fn new(model: &str) -> Self {
        let webview = WebView::new();
        webview.set_size_request(800, 600);
        webview.set_opacity(1.0);
        
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
        
        let bridge = AiBridge {
            webview: webview.clone(),
            model: model.to_string(),
            last_response: Rc::new(RefCell::new(String::new())),
            response_callback: Rc::new(RefCell::new(None)),
        };
        
        bridge.inject_observer();
        bridge.start_polling();
        
        bridge
    }
    
    fn inject_observer(&self) {
        let js = match self.model.as_str() {
            "chatgpt" => include_str!("ui/inject/chatgpt.js"),
            "claude" => include_str!("ui/inject/claude.js"),
            "gemini" => include_str!("ui/inject/gemini.js"),
            "deepseek" => include_str!("ui/inject/deepseek.js"),
            "grok" => include_str!("ui/inject/grok.js"),
            "qwen" => include_str!("ui/inject/qwen.js"),
            "kimi" => include_str!("ui/inject/kimi.js"),
            _ => include_str!("ui/inject/chatgpt.js"),
        };
        

        self.webview.connect_load_changed(move |_webview, event| {
            if event == webkit2gtk::LoadEvent::Finished {
                let js = js.to_string();
                webview.run_javascript(&js, None::<&gio::Cancellable>, |_| {});
            }
        });
    }
    
    fn start_polling(&self) {
        let wv = self.webview.clone();
        let last = self.last_response.clone();
        let cb = self.response_callback.clone();
        
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(2000), move || {
            wv.run_javascript("window.__vibi_last || ''", None::<&gio::Cancellable>, move |result| {
                if let Ok(res) = result {
                    if let Some(js_val) = res.to_value() {
                        let text = js_val.to_string();
                        let text = text.trim_matches('"').to_string();
                        if !text.is_empty() && text != *last.borrow() {
                            *last.borrow_mut() = text.clone();
                            if let Some(ref callback) = *cb.borrow() {
                                callback(text);
                            }
                        }
                    }
                }
            });
            gtk::glib::ControlFlow::Continue
        });
    }
    
    pub fn send_message(&self, text: &str) {
        println!("[AiBridge] Sending to {}: {}", self.model, text);
        let escaped = text.replace('\\', "\\\\").replace('\'', "\\'").replace('\n', "\\n").replace('"', "\\\"");
        let js = match self.model.as_str() {
            "chatgpt" => format!(
                "(function() {{ var input = document.querySelector('#prompt-textarea') || document.querySelector('[contenteditable=\"true\"]') || document.querySelector('textarea'); if(input) {{ if(input.tagName === 'DIV' || input.contentEditable === 'true') {{ input.textContent = \"{}\"; }} else {{ input.value = \"{}\"; }} input.dispatchEvent(new Event('input', {{ bubbles: true }})); setTimeout(function() {{ var btn = document.querySelector('[data-testid=\"send-button\"]') || document.querySelector('button[type=\"submit\"]'); if(btn) btn.click(); }}, 300); }} }})()",
                escaped, escaped
            ),
            _ => format!(
                "(function() {{ var input = document.querySelector('textarea, [contenteditable=\"true\"]'); if(input) {{ if(input.tagName === 'DIV' || input.contentEditable === 'true') {{ input.textContent = \"{}\"; }} else {{ input.value = \"{}\"; }} input.dispatchEvent(new Event('input', {{ bubbles: true }})); }} }})()",
                escaped, escaped
            ),
        };
        self.webview.run_javascript(&js, None::<&gio::Cancellable>, |_| {});
    }
    
    pub fn on_response<F: Fn(String) + 'static>(&self, callback: F) {
        *self.response_callback.borrow_mut() = Some(Box::new(callback));
    }
}