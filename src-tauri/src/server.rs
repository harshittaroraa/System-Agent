use axum::{
    routing::get,
    Router,
    Json,
    extract::State,
    http::{Method, HeaderValue},
};
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use crate::Device;
use tower_http::cors::{CorsLayer, Any};

// Shared state to hold device list
#[derive(Clone)]
struct AppState {
    devices: Arc<Mutex<Vec<Device>>>,
}

// Handler to return the list of devices
async fn get_devices(State(state): State<AppState>) -> Json<Vec<Device>> {
    let devices = state.devices.lock().unwrap().clone();
    Json(devices)
}

// Handler for root path
async fn root() -> &'static str {
    "Device API Server"
}

// Function to start the server
pub async fn start_server(devices: Vec<Device>) -> std::io::Result<String> {
    // Create shared state
    let state = AppState {
        devices: Arc::new(Mutex::new(devices)),
    };
    
    // Configure CORS middleware
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);
    
    // Create router with endpoints
    let app = Router::new()
        .route("/", get(root))
        .route("/api/devices", get(get_devices))
        .with_state(state)
        .layer(cors);
    
    // Try binding to ports in the range 9000-9010
    let ports = vec![9000, 9001, 9002, 9003, 9004, 9005, 9006, 9007, 9008, 9009, 9010];
    let mut listener = None;
    
    for port in ports {
        print!("Trying port {}... ", port);
        match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
            Ok(l) => {
                listener = Some(l);
                println!("Server started on http://localhost:{}", port);
                break;
            }
            Err(_) => continue,
        }
    }
    
    let listener = match listener {
        Some(l) => l,
        None => return Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "All ports in range 9000-9010 are in use")),
    };
    
    let addr = listener.local_addr()?;
    
    // Start the server
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    Ok(format!("http://{}", addr))
}
