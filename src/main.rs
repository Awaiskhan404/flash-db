mod in_memory_db;
use serde_json::Value;
use std::time::Duration;
use in_memory_db::InMemoryDB;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::Arc;
use std::env;
use std::thread;
use env_logger;
use log;

/// Handles a client connection, performing authentication and processing commands.
///
/// # Parameters
/// - `stream`: The TCP stream connected to the client.
/// - `db`: A shared reference to the in-memory database.
/// - `auth_token`: The authentication token required for client access.
fn handle_client(mut stream: TcpStream, db: Arc<InMemoryDB>, auth_token: String) {
    let client_addr = stream.peer_addr().unwrap();
    let mut buffer = [0; 512];

    // Read the authentication token sent by the client
    let bytes_read = match stream.read(&mut buffer) {
        Ok(0) => {
            log::info!("Client {} disconnected", client_addr);
            return;
        }
        Ok(bytes) => bytes,
        Err(e) => {
            log::error!("Error reading from client {}: {}", client_addr, e);
            return;
        }
    };

    // Extract and verify the client's token
    let client_token = String::from_utf8_lossy(&buffer[..bytes_read]).trim().to_string();
    if client_token != auth_token {
        log::error!("Client {} provided invalid token", client_addr);
        let _ = stream.write_all("Invalid authentication token.\n".as_bytes());
        return;
    }
    log::info!("Client {} authenticated successfully", client_addr);

    // Process commands from the client
    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(0) => {
                log::info!("Client {} disconnected", client_addr);
                break;
            }
            Ok(bytes) => bytes,
            Err(e) => {
                log::error!("Error reading from client {}: {}", client_addr, e);
                break;
            }
        };

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let response = process_command(&request, &db);

        if let Err(e) = stream.write_all(response.as_bytes()) {
            log::error!("Error writing to client {}: {}", client_addr, e);
            break;
        }
    }
}

/// Processes a command from the client, such as setting, retrieving, or deleting a key.
///
/// # Parameters
/// - `command`: The raw command string from the client.
/// - `db`: A shared reference to the in-memory database.
///
/// # Returns
/// A response string for the client, indicating the result of the command.
fn process_command(command: &str, db: &Arc<InMemoryDB>) -> String {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    if parts.is_empty() {
        log::warn!("Empty command received");
        return "Invalid command\n".to_string();
    }

    match parts[0] {
        "set" => {
            if parts.len() < 3 {
                log::warn!("'set' command requires a key and a JSON value.");
                return "Usage: set <key> <json_value> [ttl]\n".to_string();
            }
            let key = parts[1].to_string();
            let json_value = parts[2..].join(" ");
            
            // Parse JSON data
            let value: Value = match serde_json::from_str(&json_value) {
                Ok(val) => val,
                Err(_) => return "Invalid JSON format\n".to_string(),
            };
            
            let ttl = if parts.len() > 3 {
                parts[3].parse::<u64>().ok().map(Duration::from_secs)
            } else {
                None
            };
            
            db.set(key.clone(), value.to_string(), ttl);
            log::info!("Set key '{}' with JSON value '{}', TTL: {:?}", key, value, ttl);
            format!("Set key '{}' with JSON value '{}'\n", key, value)
        }
        "get" => {
            if parts.len() < 2 {
                log::warn!("'get' command requires a key.");
                return "Usage: get <key>\n".to_string();
            }
            let key = parts[1];
            match db.get(key) {
                Some(value) => {
                    log::info!("Retrieved key '{}': '{}'", key, value);
                    format!("Value for '{}': {}\n", key, value)
                }
                None => {
                    log::info!("Key '{}' not found or expired", key);
                    format!("Key '{}' not found or expired\n", key)
                }
            }
        }
        "delete" => {
            if parts.len() < 2 {
                log::warn!("'delete' command requires a key.");
                return "Usage: delete <key>\n".to_string();
            }
            let key = parts[1];
            if db.delete(key) {
                log::info!("Deleted key '{}'", key);
                format!("Deleted key '{}'\n", key)
            } else {
                log::info!("Key '{}' not found", key);
                format!("Key '{}' not found\n", key)
            }
        }
        "exit" => {
            log::info!("Client requested to disconnect.");
            "Goodbye!\n".to_string()
        }
        _ => {
            log::warn!("Unknown command '{}'", command);
            "Unknown command\n".to_string()
        }
    }
}

/// Main function to initialize the server and handle incoming connections.
fn main() {
    env::set_var("RUST_LOG", "info"); // Set default log level to info
    env_logger::init(); // Initialize logging

    // Get the hardcoded token from the environment
    let auth_token = env::var("AUTH_TOKEN").expect("AUTH_TOKEN must be set");

    let db = Arc::new(InMemoryDB::new());
    db.start_expiration_thread(); // Start background expiration thread

    let listener = TcpListener::bind("0.0.0.0:7878").expect("Failed to bind to address");
    log::info!("Server running on 0.0.0.0:7878");

    // Accept and handle incoming connections
    for stream in listener.incoming() {
        let stream = stream.expect("Failed to accept connection");
        let db = Arc::clone(&db);

        log::info!("New client connected from {}", stream.peer_addr().unwrap());

        let auth_token = auth_token.clone();
        thread::spawn(move || {
            handle_client(stream, db, auth_token);
        });
    }
}