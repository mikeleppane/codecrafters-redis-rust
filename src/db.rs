use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

#[derive(Debug)]
pub enum GetValue<'a> {
    Ok(&'a str),
    None,
    Error(&'a str),
}

pub trait Database {
    fn get(&self, key: &str) -> GetValue;
    fn set(&mut self, key: &str, value: &str, expires_at: Option<u64>);
    fn delete(&mut self, key: &str) -> Option<String>;
}

#[derive(Debug)]
pub struct DbValue {
    value: String,
    expires_at: Option<SystemTime>,
}

impl DbValue {
    pub fn new(value: String, expires_at: Option<SystemTime>) -> Self {
        Self { value, expires_at }
    }
}

#[derive(Debug)]
pub struct RedisDatabase {
    pub data: HashMap<String, DbValue>,
}

impl RedisDatabase {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Database for RedisDatabase {
    fn set(&mut self, key: &str, value: &str, expires_at: Option<u64>) {
        if let Some(expires_at) = expires_at {
            let now = SystemTime::now();
            let expiry_duration = Duration::from_millis(expires_at);
            let expires_at = now + expiry_duration;
            self.data.insert(
                key.to_owned(),
                DbValue::new(value.to_owned(), Some(expires_at)),
            );
        } else {
            self.data
                .insert(key.to_owned(), DbValue::new(value.to_owned(), None));
        }
    }

    fn get(&self, key: &str) -> GetValue {
        match self.data.get(key) {
            Some(DbValue {
                value,
                expires_at: Some(expires_at),
            }) => {
                if expires_at <= &SystemTime::now() {
                    GetValue::Error(value)
                } else {
                    GetValue::Ok(value)
                }
            }
            Some(DbValue {
                value,
                expires_at: None,
            }) => GetValue::Ok(value),
            None => GetValue::None,
        }
    }

    fn delete(&mut self, key: &str) -> Option<String> {
        self.data.remove(key).map(|v| v.value)
    }
}
