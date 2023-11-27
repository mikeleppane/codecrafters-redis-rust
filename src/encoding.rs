pub fn to_bulk_string(s: &str) -> String {
    let len = s.len();
    format!("${}\r\n{}\r\n", len, s)
}

pub fn to_list_of_bulk_strings(list: &[String]) -> String {
    let mut buffer = String::new();
    buffer.push_str(&format!("*{}\r\n", list.len()));
    for item in list {
        buffer.push_str(&to_bulk_string(item));
    }
    buffer
}

pub fn encode_response_as_simple_string(response: &[u8]) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(b"+");
    buffer.extend_from_slice(response);
    buffer.extend_from_slice(b"\r\n");
    buffer
}
