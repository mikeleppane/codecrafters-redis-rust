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
