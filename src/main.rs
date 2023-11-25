use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

use anyhow::Result;

fn handle_connections(stream: &mut TcpStream) -> Result<()> {
    let res = "+PONG\r\n";
    stream.write_all(res.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap_or_else(|e| {
        panic!("failed to bind to socket: {}", e);
    });

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                handle_connections(&mut stream).unwrap_or_else(|e| {
                    println!("failed to handle connection: {}", e);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
