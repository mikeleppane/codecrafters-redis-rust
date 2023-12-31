use anyhow::Result;
use clap::Parser;
use std::{
    fs::File,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

mod command;
mod config;
mod db;
mod encoding;
mod parser;
mod response;
use crate::config::Config;
use command::{Command, SetCommand};
use db::{Database, GetValue, RedisDatabase};
use encoding::{encode_response_as_simple_string, to_bulk_string, to_list_of_bulk_strings};
use parser::{RDBParser, Rdb};
use response::{RespParser, Value};

#[derive(Parser, Debug)]
#[clap(
    author = "Mikko Leppänen <mleppan23@gmail.com>",
    version = "0.1",
    about
)]
pub struct Args {
    /// The directory where RDB files are stored
    #[arg(short, long)]
    pub dir: Option<PathBuf>,
    /// The name of the RDB files
    #[arg(short, long)]
    pub dbfilename: Option<String>,
}

fn read_from_stream(stream: &mut TcpStream) -> Option<Vec<u8>> {
    let mut buf = [0; 1024];
    let size = stream.read(&mut buf);
    match size {
        Ok(size) => Some(buf[..size].to_vec()),
        Err(_) => None,
    }
}

fn parse(data: &[u8]) -> Option<Value> {
    let mut parser = RespParser::new(data);
    let (value, _) = parser.parse();
    value
}

fn process_request(request: &[u8]) -> Option<Command> {
    let value = parse(request);
    match value {
        Some(Value::Array(array)) => Command::handle_command(&array),
        _ => {
            eprintln!("unable to parse request");
            None
        }
    }
}

fn read_rdb_file(path: PathBuf) -> Result<Rdb> {
    let file = File::open(path).expect("Unable to open file");
    let mut reader = io::BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    let mut parser = RDBParser::new(&buffer);
    let rdb = parser.parse()?;
    Ok(rdb)
}

async fn handle_connection<T: Database>(
    mut stream: TcpStream,
    db: Arc<Mutex<T>>,
    config: Arc<Mutex<Config>>,
) -> Result<()> {
    while let Some(request) = read_from_stream(&mut stream) {
        if request.is_empty() {
            break;
        }
        let response = process_request(&request);
        match response {
            Some(Command::Ping(response)) => {
                stream
                    .write_all(encode_response_as_simple_string(response.as_bytes()).as_slice())
                    .unwrap();
            }

            Some(Command::Echo(response)) => {
                stream
                    .write_all(encode_response_as_simple_string(response.as_bytes()).as_slice())
                    .unwrap();
            }

            Some(Command::Set(set_command)) => {
                let SetCommand { key, value, px } = set_command;
                let mut db = db.lock().unwrap();

                db.set(&key, &value, px);
                stream.write_all(encode_response_as_simple_string(b"OK").as_slice())?;
            }

            Some(Command::Get(key)) => {
                let _db = db.lock().unwrap();
                let value = _db.get(&key);
                match value {
                    GetValue::Error(_) => {
                        stream.write_all(b"$-1\r\n").unwrap();
                        let mut db = db.lock().unwrap();
                        db.delete(&key);
                    }
                    GetValue::Ok(value) => stream
                        .write_all(encode_response_as_simple_string(value.as_bytes()).as_slice())?,
                    GetValue::None => {
                        let mut rdb = Rdb::new();
                        if let Some(path) = config.lock().unwrap().to_file_path() {
                            match read_rdb_file(path) {
                                Ok(new_rdb) => rdb = new_rdb,
                                Err(e) => {
                                    panic!("Unable to read and parse rdb: {}", e)
                                }
                            }
                        }
                        let value = rdb.get(&key);
                        match value {
                            Some(value) => {
                                if value.expiry.is_some() {
                                    let start = SystemTime::now();
                                    let since_the_epoch =
                                        start.duration_since(UNIX_EPOCH).unwrap().as_millis();
                                    if value.expiry.unwrap() as u128 <= since_the_epoch {
                                        stream.write_all(b"$-1\r\n").unwrap();
                                        rdb.delete(&key);
                                        continue;
                                    } else {
                                        stream
                                            .write_all(
                                                to_bulk_string(&value.value.to_string()).as_bytes(),
                                            )
                                            .unwrap();
                                    }
                                } else {
                                    stream
                                        .write_all(
                                            to_bulk_string(&value.value.to_string()).as_bytes(),
                                        )
                                        .unwrap();
                                }
                            }
                            None => {
                                stream.write_all(b"$-1\r\n").unwrap();
                            }
                        }
                    }
                }
            }

            Some(Command::Config(config_key)) => {
                let config = config.lock().unwrap();
                if let Some(config_value) = config.get(&config_key) {
                    stream.write_all(
                        to_list_of_bulk_strings(&[
                            config_key.to_string(),
                            config_value.to_string(),
                        ])
                        .as_bytes(),
                    )?
                } else {
                    stream.write_all(b"$-1\r\n").unwrap();
                }
            }

            Some(Command::Keys(keys)) => {
                if keys.as_str() == "*" {
                    let mut rdb = Rdb::new();
                    if let Some(path) = config.lock().unwrap().to_file_path() {
                        match read_rdb_file(path) {
                            Ok(new_rdb) => rdb = new_rdb,
                            Err(e) => {
                                panic!("Unable to read and parse rdb: {}", e)
                            }
                        }
                    }
                    stream.write_all(to_list_of_bulk_strings(&rdb.get_keys()).as_bytes())?
                }
            }

            None => stream.write_all(b"-ERR unknown command\r\n")?,
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap_or_else(|e| {
        panic!("failed to bind to socket: {}", e);
    });

    let args = Args::parse();

    let db = Arc::new(Mutex::new(RedisDatabase::new()));
    let config = Arc::new(Mutex::new(Config::new(args.dir, args.dbfilename)));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let db = Arc::clone(&db);
                let config = Arc::clone(&config);
                tokio::task::spawn(async move {
                    match handle_connection(stream, db, config).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Failure while handling connection: {}", e);
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("failed to read stream:  {e}");
            }
        }
    }
    Ok(())
}
