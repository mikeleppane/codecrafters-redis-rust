use anyhow::Result;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

mod command;
mod response;
use command::Command;
use response::{RespParser, Value};

fn read_from_stream(stream: &mut TcpStream) -> Option<Vec<u8>> {
    let mut buf = [0; 1024];
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

fn process_request(request: &[u8]) -> Option<String> {
    let value = parse(request);
    match value {
        Some(Value::Array(array)) => Command::handle_command(&array),
        _ => {
            eprintln!("unable to parse request");
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
            None => stream
                .write_all(b"-ERR unknown command\r\n")
                .expect("could not write to stream"),
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
