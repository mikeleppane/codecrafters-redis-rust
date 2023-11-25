use std::fmt::Display;

#[derive(Debug)]
pub enum Value {
    String(String),
    Array(Vec<Value>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(string) => write!(f, "{}", string),
            Value::Array(array) => {
                let mut result = String::new();
                result.push('[');
                for (i, value) in array.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&value.to_string());
                }
                result.push(']');
                write!(f, "{}", result)
            }
        }
    }
}

pub struct RespParser<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> RespParser<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        RespParser { buf, pos: 0 }
    }

    pub fn parse(&mut self) -> (Option<Value>, usize) {
        let value = self.parse_value();

        (value, self.pos)
    }

    fn parse_value(&mut self) -> Option<Value> {
        if self.pos < self.buf.len() {
            let byte = self.buf[self.pos];
            self.pos += 1;
            match byte {
                b'+' => self.parse_simple_string(),
                b'*' => self.parse_array(),
                b'$' => self.parse_bulk_string(),
                _ => None,
            }
        } else {
            None
        }
    }

    fn parse_simple_string(&mut self) -> Option<Value> {
        let string_raw = self.until_crlf();
        let string = Self::as_string(string_raw)?;
        Some(Value::String(string))
    }

    fn parse_bulk_string(&mut self) -> Option<Value> {
        let length_raw = self.until_crlf();
        let length = Self::as_usize(length_raw)?;
        let string_raw = &self.buf[self.pos..self.pos + length];
        self.pos += length + 2;
        let string = Self::as_string(string_raw)?;
        Some(Value::String(string))
    }

    fn parse_array(&mut self) -> Option<Value> {
        let length_raw = self.until_crlf();
        let length = Self::as_usize(length_raw)?;
        let mut array: Vec<Value> = Vec::with_capacity(length);
        for _ in 0..length {
            let value = self.parse_value();

            if let Some(value) = value {
                array.push(value);
            }
        }
        Some(Value::Array(array))
    }

    fn as_usize(buf: &[u8]) -> Option<usize> {
        if let Ok(str) = std::str::from_utf8(buf) {
            str.parse::<usize>().ok()
        } else {
            None
        }
    }

    fn as_string(buf: &[u8]) -> Option<String> {
        if let Ok(str) = std::str::from_utf8(buf) {
            Some(str.to_string())
        } else {
            None
        }
    }

    fn until_crlf(&mut self) -> &[u8] {
        let begin = self.pos;

        while self.pos < self.buf.len() {
            let byte = self.buf[self.pos];
            self.pos += 1;
            if byte == b'\r' && self.pos < self.buf.len() && self.buf[self.pos] == b'\n' {
                self.pos += 1;
                break;
            }
        }
        &self.buf[begin..self.pos - 2]
    }
}
