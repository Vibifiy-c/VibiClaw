use sysinfo::System;
use tungstenite::accept;
use std::net::TcpListener;
use serde_json::json;
use std::thread;
use std::time::Duration;

pub fn start_hardware_server() {
    thread::spawn(move || {
        let server = TcpListener::bind("127.0.0.1:8765").expect("Failed to bind hardware server");
        println!("[Hardware] Server listening on ws://127.0.0.1:8765");

        for stream in server.incoming() {
            thread::spawn(move || {
                let mut websocket = accept(stream.unwrap()).unwrap();
                let mut sys = System::new_all();

                loop {
            sys.refresh_cpu();
            sys.refresh_memory();
            thread::sleep(Duration::from_millis(500));

            let cpu_usage: f32 = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;
                    let total_mem = sys.total_memory();
                    let used_mem = sys.used_memory();
                    let ram_usage = if total_mem > 0 { (used_mem * 100 / total_mem) as f32 } else { 0.0 };

                    let payload = json!({
                        "cpu": cpu_usage.round(),
                        "ram": ram_usage.round()
                    }).to_string();

                    if websocket.send(tungstenite::Message::Text(payload)).is_err() {
                        break;
                    }
                    thread::sleep(Duration::from_millis(1500));
                }
                
            });
        }
    });
}