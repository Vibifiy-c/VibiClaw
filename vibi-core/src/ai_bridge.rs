use webkit2gtk::{WebView, WebViewExt};
use gtk::prelude::*;
use glib::translate::ToGlibPtr;
use gio;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Instant;
use std::ffi::CString;

pub struct AiBridge {
    pub webview: WebView,
    response_callback: Rc<RefCell<Option<Box<dyn Fn(String)>>>>,
    last_activity: Rc<RefCell<Instant>>,
    sleep_enabled: Rc<RefCell<bool>>,
    model: Rc<RefCell<String>>,
    page_loaded: Rc<RefCell<bool>>,
    chunk_buffer: Rc<RefCell<Vec<String>>>,
}

fn setup_persistent_cookies(webview: &WebView) {
    unsafe {
        let cookie_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("vibi-ai")
            .join("webkit");
        std::fs::create_dir_all(&cookie_dir).ok();
        
        let cookie_path = cookie_dir.join("cookies.db");
        let path_str = cookie_path.to_str().unwrap();
        let c_path = CString::new(path_str).unwrap();
        
        use webkit2gtk_sys::{webkit_web_view_get_context, webkit_web_context_get_cookie_manager, webkit_cookie_manager_set_persistent_storage, WEBKIT_COOKIE_PERSISTENT_STORAGE_SQLITE};
        let wv_ptr: *mut webkit2gtk_sys::WebKitWebView = webview.to_glib_none().0;
        let context = webkit_web_view_get_context(wv_ptr);
        if !context.is_null() {
            let manager = webkit_web_context_get_cookie_manager(context);
            if !manager.is_null() {
                webkit_cookie_manager_set_persistent_storage(
                    manager,
                    c_path.as_ptr(),
                    WEBKIT_COOKIE_PERSISTENT_STORAGE_SQLITE,
                );
                println!("[AiBridge] Persistent cookie storage set: {}", path_str);
            }
        }
    }
}

impl AiBridge {
    pub fn new() -> Self {
        let webview = WebView::new();
        webview.set_size_request(1, 1);
        webview.set_opacity(0.0);
        webview.load_uri("about:blank");
        
        // Set up persistent cookies via FFI
        setup_persistent_cookies(&webview);
        
        let page_loaded = Rc::new(RefCell::new(false));
        let model = Rc::new(RefCell::new(String::new()));
        
        let bridge = AiBridge {
            webview: webview.clone(),
            response_callback: Rc::new(RefCell::new(None)),
            last_activity: Rc::new(RefCell::new(Instant::now())),
            sleep_enabled: Rc::new(RefCell::new(true)),
            model: model.clone(),
            page_loaded: page_loaded.clone(),
            chunk_buffer: Rc::new(RefCell::new(Vec::new())),
        };
        
        let cb = bridge.response_callback.clone();
        let loaded = page_loaded.clone();
        let current_model = model.clone();
        webview.connect_load_changed(move |webview, event| {
            match event {
                webkit2gtk::LoadEvent::Committed => {
                    *loaded.borrow_mut() = false;
                }
                webkit2gtk::LoadEvent::Finished => {
                    *loaded.borrow_mut() = true;
                    let model_str = current_model.borrow().clone();
                    println!("[AiBridge] Page loaded for {}", model_str);
                    
                    // Reset observer state so new model's JS can run
                    webview.run_javascript(
                        "delete window.__vibi_obs; delete window.__vibi_last; delete window.__vibi_send;",
                        None::<&gio::Cancellable>, |_| {}
                    );
                    
                    let js = match model_str.as_str() {
                        "chatgpt" => include_str!("ui/inject/chatgpt.js"),
                        "claude" => include_str!("ui/inject/claude.js"),
                        "gemini" => include_str!("ui/inject/gemini.js"),
                        "deepseek" => include_str!("ui/inject/deepseek.js"),
                        "grok" => include_str!("ui/inject/grok.js"),
                        "qwen" => include_str!("ui/inject/qwen.js"),
                        "kimi" => include_str!("ui/inject/kimi.js"),
                        _ => include_str!("ui/inject/chatgpt.js"),
                    };
                    webview.run_javascript(js, None::<&gio::Cancellable>, |_| {});
                    println!("[AiBridge] Observer JS injected for {}", model_str);
                }
                _ => {}
            }
        });
        
        let chunk_buf = bridge.chunk_buffer.clone();
        webview.connect_uri_notify(move |wv| {
            if let Some(uri) = wv.uri() {
                let uri_str = uri.to_string();
                if let Some(hash_pos) = uri_str.find("#vibi-") {
                    let payload = &uri_str[hash_pos + "#vibi-".len()..];
                    let payload = payload.split('&').next().unwrap_or(payload).split('?').next().unwrap_or(payload);
                    
                    if payload == "done" {
                        let full_hex: String = chunk_buf.borrow().iter().map(|s| s.as_str()).collect();
                        chunk_buf.borrow_mut().clear();
                        if let Ok(text) = hex_decode(&full_hex) {
                            if !text.is_empty() {
                                println!("[AiBridge] Response ({} chars): {}...", text.len(), &text[..text.len().min(100)]);
                                if let Some(ref callback) = *cb.borrow() {
                                    callback(text);
                                }
                            }
                        }
                    } else if let Some(dash_pos) = payload.find('-') {
                        let idx_str = &payload[..dash_pos];
                        let data = &payload[dash_pos + 1..];
                        if let Ok(idx) = idx_str.parse::<usize>() {
                            while chunk_buf.borrow().len() <= idx {
                                chunk_buf.borrow_mut().push(String::new());
                            }
                            chunk_buf.borrow_mut()[idx] = data.to_string();
                        }
                    }
                }
            }
        });
        
        let last = bridge.last_activity.clone();
        let sleep = bridge.sleep_enabled.clone();
        let wv_sleep = webview.clone();
        gtk::glib::timeout_add_local(std::time::Duration::from_secs(60), move || {
            if *sleep.borrow() {
                let elapsed = last.borrow().elapsed();
                if elapsed > std::time::Duration::from_secs(600) {
                    println!("[AiBridge] Sleeping webview");
                    wv_sleep.load_uri("about:blank");
                }
            }
            gtk::glib::ControlFlow::Continue
        });
        
        bridge
    }
    
    pub fn load_model(&self, new_model: &str) {
        let current = self.model.borrow().clone();
        if current == new_model && *self.page_loaded.borrow() {
            println!("[AiBridge] Model {} already loaded", new_model);
            return;
        }
        
        *self.model.borrow_mut() = new_model.to_string();
        *self.last_activity.borrow_mut() = Instant::now();
        *self.page_loaded.borrow_mut() = false;
        
        let url = match new_model {
            "chatgpt" => "https://chat.openai.com",
            "claude" => "https://claude.ai",
            "gemini" => "https://gemini.google.com",
            "deepseek" => "https://chat.deepseek.com",
            "grok" => "https://grok.com",
            "qwen" => "https://tongyi.aliyun.com/qianwen",
            "kimi" => "https://www.kimi.com",
            _ => "https://chat.openai.com",
        };
        
        println!("[AiBridge] Loading model: {} -> {}", new_model, url);
        self.webview.load_uri(url);
    }
    
    pub fn send_message(&self, text: &str) {
        *self.last_activity.borrow_mut() = Instant::now();
        let model = self.model.borrow().clone();
        println!("[AiBridge] Sending to {}: {}", model, text);
        
        let escaped = text.replace('\\', "\\\\").replace('\'', "\\'").replace('\n', "\\n").replace('"', "\\\"");
        let js = match model.as_str() {
            "chatgpt" => format!(
                "(function() {{ var input = document.querySelector('#prompt-textarea') || document.querySelector('[contenteditable=\"true\"]'); if(input) {{ input.textContent = '{}'; input.dispatchEvent(new Event('input', {{ bubbles: true }})); setTimeout(function() {{ var btn = document.querySelector('[data-testid=\"send-button\"]'); if(btn) btn.click(); }}, 800); }} }})()",
                escaped
            ),
            "gemini" => format!(
                "window.__vibi_send && window.__vibi_send('{}');",
                escaped
            ),
            "deepseek" => format!(
                "(function() {{ var input = document.querySelector('textarea._27c9245') || document.querySelector('textarea'); if(input) {{ input.value = '{}'; input.dispatchEvent(new Event('input', {{ bubbles: true }})); var attempts = 0; var trySend = setInterval(function() {{ attempts++; var btn = document.querySelector('.ds-button--icon') || document.querySelector('[class*=\"ds-button\"]'); if(btn && !btn.disabled) {{ btn.click(); clearInterval(trySend); }} if(attempts > 20) clearInterval(trySend); }}, 300); }} }})()",
                escaped
            ),
            _ => format!(
                "(function() {{ var input = document.querySelector('[contenteditable=\"true\"]') || document.querySelector('textarea'); if(input) {{ input.textContent = '{}'; input.dispatchEvent(new Event('input', {{ bubbles: true }})); setTimeout(function() {{ var btn = document.querySelector('button[type=\"submit\"]'); if(btn) btn.click(); }}, 800); }} }})()",
                escaped
            ),
        };
        
        let wv = self.webview.clone();
        let loaded = self.page_loaded.clone();
        
        if *loaded.borrow() {
            wv.run_javascript(&js, None::<&gio::Cancellable>, |_| {});
            println!("[AiBridge] JS sent (page was loaded)");
        } else {
            println!("[AiBridge] Page not loaded, reloading model...");
            let model_str = self.model.borrow().clone();
            let url = match model_str.as_str() {
                "chatgpt" => "https://chat.openai.com",
                "claude" => "https://claude.ai",
                "gemini" => "https://gemini.google.com",
                "deepseek" => "https://chat.deepseek.com",
                "grok" => "https://grok.com",
                "qwen" => "https://tongyi.aliyun.com/qianwen",
                "kimi" => "https://www.kimi.com",
                _ => "https://chat.openai.com",
            };
            self.webview.load_uri(url);
            gtk::glib::timeout_add_local(std::time::Duration::from_millis(300), move || {
                if *loaded.borrow() {
                    wv.run_javascript(&js, None::<&gio::Cancellable>, |_| {});
                    println!("[AiBridge] JS sent (after reload)");
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
    
    pub fn set_sleep_enabled(&self, enabled: bool) {
        *self.sleep_enabled.borrow_mut() = enabled;
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