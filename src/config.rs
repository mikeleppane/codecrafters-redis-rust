use std::path::PathBuf;

pub struct Config {
    pub dir: Option<PathBuf>,
    pub dbfilename: Option<String>,
}

impl Config {
    pub fn new(dir: Option<PathBuf>, dbfilename: Option<String>) -> Self {
        Config { dir, dbfilename }
    }

    pub fn to_file_path(&self) -> Option<PathBuf> {
        match (&self.dir, &self.dbfilename) {
            (Some(dir), Some(dbfilename)) => Some(dir.join(dbfilename)),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        match key {
            "dir" => self.dir.as_ref().and_then(|path| path.to_str()),
            "dbfilename" => self.dbfilename.as_deref(),
            _ => None,
        }
    }

    pub fn encode_to_resp(&self, key: &str, value: &str) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(b"*2\r\n");
        buffer.extend_from_slice(to_bulk_string(key).as_bytes());
        buffer.extend_from_slice(to_bulk_string(value).as_bytes());
        buffer
    }
}

fn to_bulk_string(s: &str) -> String {
    let len = s.len();
    format!("${}\r\n{}\r\n", len, s)
}
