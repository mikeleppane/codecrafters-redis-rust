use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use anyhow::Result;

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn parse_request(request: &[u8]) -> bool {
    match request[0] {
        b'*' => {
            let size_end = request.iter().position(|&x| x == b'\r').unwrap();
            let _array_size = std::str::from_utf8(&request[1..size_end])
                .unwrap()
                .parse::<usize>()
                .unwrap();

            contains(request, b"PING") || contains(request, b"ping")
        }
        _ => false,
    }
}

fn read_from_stream(stream: &mut TcpStream) -> Option<Vec<u8>> {
    let mut buf = [0; 512];
    let size = stream.read(&mut buf);
    match size {
        Ok(size) => Some(buf[..size].to_vec()),
        Err(_) => None,
    }
}

async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    while let Some(request) = read_from_stream(&mut stream) {
        if request.is_empty() {
            break;
        }
        let is_ping = parse_request(&request);
        match is_ping {
            true => {
                stream.write_all(b"+PONG\r\n").unwrap();
            }
            false => {
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
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
