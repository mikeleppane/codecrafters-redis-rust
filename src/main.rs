use anyhow::Result;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

mod response;

use response::{RespParser, Value};

fn read_from_stream(stream: &mut TcpStream) -> Option<Vec<u8>> {
    let mut buf = [0; 512];
    let size = stream.read(&mut buf);
    match size {
        Ok(size) => Some(buf[..size].to_vec()),
        Err(_) => None,
    }
}

fn encode_response(response: &[u8]) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(b"+");
    buffer.extend_from_slice(response);
    buffer.extend_from_slice(b"\r\n");
    buffer
}

fn parse(data: &[u8]) -> Option<Value> {
    let mut parser = RespParser::new(data);
    let (value, _) = parser.parse();
    value
}

fn process_command(command: Vec<Value>) -> Option<String> {
    let command_name = &command[0];
    let command_args = &command[1..];
    match command_name {
        Value::String(string) => match string.to_lowercase().as_str() {
            "ping" => Some("PONG".to_string()),
            "echo" => command_args
                .iter()
                .map(|arg| match arg {
                    Value::String(string) => string.clone(),
                    _ => "".to_string(),
                })
                .collect::<Vec<String>>()
                .join(" ")
                .into(),
            _ => {
                eprintln!("unknown command: {}", string);
                None
            }
        },
        _ => {
            eprintln!("unexpected token: {:?}", command_name);
            None
        }
    }
}

fn process_request(request: &[u8]) -> Option<String> {
    let value = parse(request);
    match value {
        Some(Value::Array(array)) => process_command(array),
        _ => {
            println!("unable to parse request");
            None
        }
    }
}

async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    while let Some(request) = read_from_stream(&mut stream) {
        if request.is_empty() {
            break;
        }
        let response = process_request(&request);
        match response {
            Some(response) => {
                stream
                    .write_all(encode_response(response.as_bytes()).as_slice())
                    .unwrap();
            }
            None => {
                stream.write_all(b"-ERR unknown command\r\n").unwrap();
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap_or_else(|e| {
        panic!("failed to bind to socket: {}", e);
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                tokio::task::spawn(async move {
                    let _ = handle_connection(stream).await;
                });
            }
            Err(e) => {
                eprintln!("failed to read stream:  {e}");
            }
        }
    }
    Ok(())
}
