use std::collections::HashMap;

pub trait Database {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: String, value: String);
}

#[derive(Debug)]
pub struct RedisDatabase {
    pub data: HashMap<String, String>,
}

impl RedisDatabase {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Database for RedisDatabase {
    fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
}
