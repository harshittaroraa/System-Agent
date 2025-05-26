// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use local_ip_address::list_afinet_netifas;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tauri::Manager;
use std::sync::{Arc, Mutex};

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

    // List all network interfaces and their MAC addresses
    if let Ok(network_interfaces) = list_afinet_netifas() {
        for (name, ip) in network_interfaces {
            let macaddress = mac_address::mac_address_by_name(&name);
            let mac = match macaddress {
                Ok(Some(mac)) => mac.to_string(),
                Ok(None) => "No MAC address found".to_string(),
                Err(e) => format!("Error: {}", e),
            };
            
            // Create device object
            let device = Device {
                name: name.clone(),
                ip: ip.to_string(),
                mac: mac.clone(),
            };
            
            // Check if device already exists in the list
            let device_exists = devices.iter().any(|d| d.name == device.name || d.mac == device.mac);
            
            // Only add the device if it doesn't already exist
            if !device_exists && device.name == "Ethernet 3" {
                // Add to vector array
                devices.push(device);
                
                // Print to console
                println!("Interface: {}, IP: {}, Mac: {}", name, ip, mac);
            } else {
                println!("Skipped duplicate device: {} ({})", name, mac);
            }
        }
    } else {
        println!("Failed to retrieve network interfaces.");
    }

    // Get the primary MAC address
    match mac_address::get_mac_address() {
        Ok(Some(mac)) => {
            println!("Primary MAC Address: {}", mac);
        },
        Ok(None) => {
            println!("No primary MAC address found");
        },
        Err(e) => {
            println!("Error getting primary MAC address: {}", e);
        },
    };

    devices
}

#[tauri::command]
async fn start_api_server(window: tauri::Window) -> Result<String, String> {
    let devices = get_mac_address();
    
    // Start the server
    match server::start_server(devices).await {
        Ok(url) => {
            println!("API server started at: {}", url);
            Ok(url)
        },
        Err(e) => {
            eprintln!("Failed to start API server: {}", e);
            Err(format!("Failed to start server: {}", e))
        }
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, get_mac_address, start_api_server])
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();
            
            // Start the API server on app startup
            let window_clone = main_window.clone();
            tauri::async_runtime::spawn(async move {
                match start_api_server(window_clone).await {
                    Ok(url) => {
                        println!("API server started successfully at: {}", url);
                        // Optionally emit an event to the frontend
                       
                    },
                    Err(e) => {
                        eprintln!("Failed to start API server on startup: {}", e);
                    }
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
