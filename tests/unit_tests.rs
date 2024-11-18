// tests/unit_tests.rs
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

// Import functions and modules to be tested
use nanodb::in_memory_db::InMemoryDB;  // Adjust based on your project structure
use nanodb::{process_command};         // Adjust based on your project structure

/// Helper function to create a new shared instance of the database.
fn setup_db() -> Arc<InMemoryDB> {
    Arc::new(InMemoryDB::new())
}

#[test]
fn test_set_command() {
    let db = setup_db();
    let key = "test_key";
    let value = json!({"name": "Alice", "age": 30});
    let command = format!("set {} {}", key, value.to_string());

    let response = process_command(&command, &db);
    assert!(response.contains("Set key 'test_key'"));

    // Check that the key was actually set in the database
    let stored_value = db.get(key).expect("Value should be present");
    assert_eq!(stored_value, value.to_string());
}

#[test]
fn test_get_command_existing_key() {
    let db = setup_db();
    let key = "existing_key";
    let value = json!({"data": "sample"});
    db.set(key.to_string(), value.to_string(), None);

    let command = format!("get {}", key);
    let response = process_command(&command, &db);
    assert!(response.contains("Value for 'existing_key': {\"data\":\"sample\"}"));
}

#[test]
fn test_get_command_non_existing_key() {
    let db = setup_db();
    let command = "get non_existing_key";
    let response = process_command(&command, &db);
    assert!(response.contains("Key 'non_existing_key' not found or expired"));
}

#[test]
fn test_delete_command_existing_key() {
    let db = setup_db();
    let key = "delete_key";
    db.set(key.to_string(), "value".to_string(), None);

    let command = format!("delete {}", key);
    let response = process_command(&command, &db);
    assert!(response.contains("Deleted key 'delete_key'"));

    // Ensure the key is actually deleted
    let deleted_value = db.get(key);
    assert!(deleted_value.is_none());
}

#[test]
fn test_delete_command_non_existing_key() {
    let db = setup_db();
    let command = "delete non_existing_key";
    let response = process_command(&command, &db);
    assert!(response.contains("Key 'non_existing_key' not found"));
}

#[test]
fn test_expire_command() {
    let db = setup_db();
    let key = "expire_key";
    db.set(key.to_string(), "temporary_value".to_string(), Some(Duration::from_secs(1)));

    // Check that the key exists initially
    let command = format!("get {}", key);
    let response = process_command(&command, &db);
    assert!(response.contains("Value for 'expire_key'"));

    // Wait for the key to expire
    std::thread::sleep(Duration::from_secs(2));
    
    // Check that the key is now expired
    let response = process_command(&command, &db);
    assert!(response.contains("Key 'expire_key' not found or expired"));
}