use std::collections::HashMap;

use crate::response::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RDBError {
    #[error("invalid magic number")]
    InvalidMagicNumber,
    #[error("invalid version")]
    InvalidVersion,
    #[error("invalid length")]
    InvalidLength,
    #[error("invalid string")]
    InvalidString,
    #[error("invalid type")]
    InvalidType,
    #[error("unexpected EOF")]
    UnexpectedEOF,
}

#[derive(Debug)]
pub struct Rdb {
    version: u8,
    db: u32,
    data: HashMap<String, Value>,
}

impl Rdb {
    pub fn new() -> Rdb {
        Rdb {
            version: 0,
            db: 0,
            data: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn set_db(&mut self, db: u32) {
        self.db = db;
    }

    #[allow(dead_code)]
    pub fn current_db(&self) -> u32 {
        self.db
    }

    pub fn add_object(&mut self, key: String, value: Value) {
        self.data.insert(key, value);
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

pub struct RDBParser<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl RDBParser<'_> {
    pub fn new(buf: &[u8]) -> RDBParser {
        RDBParser { buf, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Rdb, RDBError> {
        let mut rdb = Rdb::new();
        self.parse_header(&mut rdb)?;
        self.parse_body(&mut rdb)?;
        Ok(rdb)
    }

    fn parse_header(&mut self, rdb: &mut Rdb) -> Result<(), RDBError> {
        let mut buf = [0u8; 9];
        self.read(&mut buf)?;
        if &buf[0..5] != b"REDIS" {
            return Err(RDBError::InvalidMagicNumber);
        }
        let version = &buf[5..];
        if version != b"0003" {
            return Err(RDBError::InvalidVersion);
        }
        rdb.version = version[1] - b'0';
        Ok(())
    }

    fn parse_body(&mut self, rdb: &mut Rdb) -> Result<(), RDBError> {
        loop {
            let byte = self.read_byte()?;
            if byte == 0xFF {
                break;
            }
            if byte == 0xFA {
                while let Ok(byte) = self.read_byte() {
                    if byte == 0xFF || byte == 0xFA || byte == 0xFE {
                        self.pos -= 1;
                        break;
                    }
                }
                continue;
            }

            if byte == 0xFE {
                let db_number = self.read_length()?;
                rdb.set_db(db_number);
                continue;
            }

            if byte == 0xFB {
                let mut buf = [0u8; 2];
                self.read(&mut buf)?;
                continue;
            }

            if byte == 0xFD {
                continue;
            }

            if byte == 0xFC {
                continue;
            }
            println!("{:#04X?}", byte);
            let key = self.read_string()?;
            let value = self.read_object(byte)?;
            println!("key: {}, value: {:?}", key, value);
            rdb.add_object(key, value);
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn read_until(&mut self, byte: u8) -> Result<Vec<u8>, RDBError> {
        let mut buf = Vec::new();
        loop {
            let b = self.read_byte()?;
            if b == byte {
                break;
            }
            buf.push(b);
        }
        Ok(buf)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), RDBError> {
        if self.pos + buf.len() > self.buf.len() {
            return Err(RDBError::UnexpectedEOF);
        }
        buf.copy_from_slice(&self.buf[self.pos..self.pos + buf.len()]);
        self.pos += buf.len();
        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8, RDBError> {
        let mut buf = [0u8; 1];
        self.read(&mut buf)?;
        Ok(buf[0])
    }

    fn read_length(&mut self) -> Result<u32, RDBError> {
        let byte = self.read_byte()?;
        match byte {
            0..=253 => Ok(byte as u32),
            254 => {
                let mut buf = [0u8; 4];
                self.read(&mut buf)?;
                Ok(u32::from_le_bytes(buf))
            }
            _ => Err(RDBError::InvalidLength),
        }
    }

    #[allow(dead_code)]
    fn read_expiry(&mut self) -> Result<(u32, u64), RDBError> {
        let db = self.read_length()?;
        let mut buf = [0u8; 8];
        self.read(&mut buf)?;
        let expires = u64::from_le_bytes(buf);
        Ok((db, expires))
    }

    fn read_string(&mut self) -> Result<String, RDBError> {
        let length = self.read_length()?;
        let mut buf = vec![0u8; length as usize];
        self.read(&mut buf)?;
        let string = String::from_utf8(buf).map_err(|_| RDBError::InvalidString)?;
        Ok(string)
    }

    fn read_object(&mut self, object_type: u8) -> Result<Value, RDBError> {
        match object_type {
            0 => Ok(Value::String(self.read_string()?)),
            _ => Err(RDBError::InvalidType), /* 1 => Ok(Value::List(self.read_list()?)),
                                             2 => Ok(Value::Set(self.read_set()?)),
                                             3 => Ok(Value::SortedSet(self.read_sorted_set()?)),
                                             4 => Ok(Value::Hash(self.read_hash()?)),
                                             9 => Ok(Value::ZipList(self.read_zip_list()?)),
                                             10 => Ok(Value::IntSet(self.read_int_set()?)),
                                             11 => Ok(Value::SortedSetAsZipList(
                                                 self.read_sorted_set_as_zip_list()?,
                                             )),
                                             12 => Ok(Value::HashmapAsZipList(self.read_hashmap_as_zip_list()?)),
                                             _ => Err(RDBError::InvalidType), */
        }
    }
}
