use std::collections::HashMap;
use std::sync::Mutex;

pub trait Database {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: String, value: String);
}

#[derive(Debug)]
pub struct RedisDatabase {
    db: Mutex<HashMap<String, String>>,
}

impl RedisDatabase {
    pub fn new() -> Self {
        Self {
            db: Mutex::new(HashMap::new()),
        }
    }
}

impl Database for RedisDatabase {
    fn set(&mut self, key: String, value: String) {
        let mut db = self.db.lock().unwrap();
        db.insert(key, value);
    }

    fn get(&self, key: &str) -> Option<String> {
        let db = self.db.lock().unwrap();
        db.get(key).cloned()
    }
}
