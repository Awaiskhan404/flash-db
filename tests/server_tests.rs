// tests/server_tests.rs

use std::process::{Command, Child};
use std::thread;
use std::time::Duration;
use std::net::TcpStream;
use std::io::{Read, Write};

/// Helper function to start the server as a separate process
fn start_server() -> Child {
    Command::new("cargo")
        .args(&["run", "--", "--bin", "your_server_binary"])
        .spawn()
        .expect("Failed to start server process")
}

/// Helper function to connect to the server
fn connect_to_server(auth_token: &str) -> TcpStream {
    let mut stream = TcpStream::connect("127.0.0.1:7878").expect("Failed to connect to server");
    stream.write_all(auth_token.as_bytes()).expect("Failed to send auth token");
    stream
}

#[test]
fn test_authentication_success() {
    let mut server = start_server();
    thread::sleep(Duration::from_secs(1)); // Give server time to start

    // Connect to server with the correct token
    let mut stream = connect_to_server("DEFAULT_AUTH_TOKEN");

    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).expect("Failed to read from server");
    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
    assert!(response.contains("Authenticated successfully"));

    server.kill().expect("Failed to kill server process");
}

#[test]
fn test_authentication_failure() {
    let mut server = start_server();
    thread::sleep(Duration::from_secs(1)); // Give server time to start

    // Connect to server with an incorrect token
    let mut stream = connect_to_server("WRONG_TOKEN");

    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).expect("Failed to read from server");
    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
    assert!(response.contains("Invalid authentication token"));

    server.kill().expect("Failed to kill server process");
}