use std::sync::Arc;

use crate::{db::Database, response::Value};

pub enum Command {
    Ping,
    Echo,
    Set,
    Get,
}

impl Command {
    fn from_str(string: &str) -> Option<Command> {
        match string.to_uppercase().as_str() {
            "PING" => Some(Command::Ping),
            "ECHO" => Some(Command::Echo),
            "SET" => Some(Command::Set),
            "GET" => Some(Command::Get),
            _ => None,
        }
    }

    pub fn process<T: Database>(&self, args: &[Value], db: &mut Arc<T>) -> Option<String> {
        match self {
            Command::Ping => Some("PONG".to_string()),
            Command::Echo => Some(
                args.iter()
                    .map(|arg| match arg {
                        Value::String(string) => string.clone(),
                        _ => "".to_string(),
                    })
                    .collect::<Vec<String>>()
                    .join(" "),
            ),
            Command::Set => {
                if args.len() != 2 {
                    eprintln!("wrong number of arguments for 'set' command");
                    return None;
                }
                match (&args[0], &args[1]) {
                    (Value::String(key), Value::String(value)) => {
                        let db = Arc::get_mut(db).unwrap();
                        db.set(key.to_owned(), value.to_owned());
                        Some("OK".to_string())
                    }
                    _ => {
                        eprintln!("wrong type of arguments for 'set' command");
                        None
                    }
                }
            }
            Command::Get => {
                if args.len() != 1 {
                    eprintln!("wrong number of arguments for 'get' command");
                    return None;
                }
                match &args[0] {
                    Value::String(key) => {
                        let value = db.get(key);
                        match value {
                            Some(value) => Some(value),
                            None => Some("".to_string()),
                        }
                    }
                    _ => {
                        eprintln!("wrong type of arguments for 'get' command");
                        None
                    }
                }
            }
        }
    }

    pub fn handle_command<T: Database>(value: &[Value], db: &mut Arc<T>) -> Option<String> {
        let command_name = &value[0];
        let command_args = &value[1..];
        match command_name {
            Value::String(string) => match Command::from_str(string) {
                Some(command) => command.process(command_args, db),
                None => {
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
}
