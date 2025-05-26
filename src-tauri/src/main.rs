// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use local_ip_address::list_afinet_netifas;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
mod server;
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Device {
    name: String,
    ip: String,
    mac: String,
}
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[tauri::command]
fn get_mac_address() -> Vec<Device> {
    let mut devices: Vec<Device> = Vec::new();
    if let Ok(network_interfaces) = list_afinet_netifas() {
        for (name, ip) in network_interfaces {
            let macaddress = mac_address::mac_address_by_name(&name);
            let mac = match macaddress {
                Ok(Some(mac)) => mac.to_string(),
                Ok(None) => "No MAC address found".to_string(),
                Err(e) => format!("Error: {}", e),
            };
            let device = Device {
                name: name.clone(),
                ip: ip.to_string(),
                mac: mac.clone(),
            };
            let device_exists = devices.iter().any(|d| d.name == device.name || d.mac == device.mac);
            if !device_exists {
                devices.push(device);
                println!("Interface: {}, IP: {}, Mac: {}", name, ip, mac);
            } else {
                println!("Skipped duplicate device: {} ({})", name, mac);
            }
        }
    } else {
        println!("Failed to retrieve network interfaces.");
    }
    match mac_address::get_mac_address() {
        Ok(Some(mac)) => println!("Primary MAC Address: {}", mac),
        Ok(None) => println!("No primary MAC address found"),
        Err(e) => println!("Error getting primary MAC address: {}", e),
    };
    devices
}
#[tauri::command]
async fn start_api_server(window: tauri::Window) -> Result<String, String> {
    let devices = get_mac_address();
    match server::start_server(devices).await {
        Ok(url) => {
            println!("API server started at: {}", url);
            Ok(url)
        }
        Err(e) => {
            eprintln!("Failed to start API server: {}", e);
            Err(format!("Failed to start server: {}", e))
        }
    }
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]) // Enable autostart by default
        ))
        .invoke_handler(tauri::generate_handler![greet, get_mac_address, start_api_server])
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();
            let window_clone = main_window.clone();
            // Spawn the server task
            tauri::async_runtime::spawn(async move {
                match start_api_server(window_clone).await {
                    Ok(url) => println!("API server started successfully at: {}", url),
                    Err(e) => eprintln!("Failed to start API server on startup: {}", e),
                }
            });
            // Handle close event to hide window
            let main_window_clone = main_window.clone();
            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close(); // Prevent default close behavior
                    main_window_clone.hide().unwrap(); // Hide the window instead of quitting
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
