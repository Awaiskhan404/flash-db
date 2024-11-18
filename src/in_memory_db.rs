use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;

#[derive(Debug, Clone)]
struct ExpiringValue {
    value: String,
    expires_at: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct InMemoryDB {
    store: Arc<Mutex<HashMap<String, ExpiringValue>>>,
}

impl InMemoryDB {
    // Create a new instance of the in-memory database
    pub fn new() -> Self {
        InMemoryDB {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Set a key-value pair with an optional expiration duration
    pub fn set(&self, key: String, value: String, ttl: Option<Duration>) {
        let expires_at = ttl.map(|duration| Instant::now() + duration);
        let expiring_value = ExpiringValue { value, expires_at };

        self.store.lock().unwrap().insert(key, expiring_value);
    }

    // Get a value by key, removing it if expired
    pub fn get(&self, key: &str) -> Option<String> {
        let mut store = self.store.lock().unwrap();

        if let Some(expiring_value) = store.get(key) {
            // Check if the value is expired
            if let Some(expires_at) = expiring_value.expires_at {
                if Instant::now() >= expires_at {
                    log::info!("Key '{}' has expired", key);
                    store.remove(key); // Remove expired key
                    return None;
                }
            }
            return Some(expiring_value.value.clone());
        }
        None
    }

    // Delete a key-value pair
    pub fn delete(&self, key: &str) -> bool {
        self.store.lock().unwrap().remove(key).is_some()
    }

    // Periodically remove expired keys (runs in a background thread)
    pub fn start_expiration_thread(&self) {
        let store = Arc::clone(&self.store);

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(1));
            let mut store = store.lock().unwrap();
            let now = Instant::now();

            // Remove all expired keys and log the removal
            store.retain(|key, v| {
                if let Some(expiration) = v.expires_at {
                    if expiration <= now {
                        log::info!("Key '{}' expired and removed", key);
                        return false; // Remove the expired key
                    }
                }
                true // Retain the key if it's not expired
            });
        });
    }
}