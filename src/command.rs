use crate::response::Value;

pub struct SetCommand {
    pub key: String,
    pub value: String,
    pub px: Option<u64>,
}

impl SetCommand {
    pub fn new(key: String, value: String, px: Option<u64>) -> Self {
        Self { key, value, px }
    }
}

pub enum Command {
    Ping(String),
    Echo(String),
    Set(SetCommand),
    Get(String),
}

impl Command {
    pub fn process(name: &str, args: &[Value]) -> Option<Command> {
        match name.to_uppercase().as_str() {
            "PING" => Some(Command::Ping("PONG".to_string())),
            "ECHO" => Some(Command::Echo(
                args.iter()
                    .map(|arg| match arg {
                        Value::String(string) => string.clone(),
                        _ => "".to_string(),
                    })
                    .collect::<Vec<String>>()
                    .join(" "),
            )),
            "SET" => {
                if args.len() > 4 {
                    eprintln!("wrong number of arguments for 'set' command");
                    return None;
                }
                if args.len() < 2 {
                    eprintln!("wrong number of arguments for 'set' command");
                    return None;
                }

                if args.len() == 2 {
                    match (&args[0], &args[1]) {
                        (Value::String(key), Value::String(value)) => {
                            return Some(Command::Set(SetCommand::new(
                                key.clone(),
                                value.clone(),
                                None,
                            )))
                        }
                        _ => {
                            eprintln!("wrong type of arguments for 'set' command");
                            return None;
                        }
                    }
                }
                if args.len() == 4 {
                    match (&args[0], &args[1], &args[2], &args[3]) {
                        (
                            Value::String(key),
                            Value::String(value),
                            Value::String(px),
                            Value::String(expiry_in_ms),
                        ) => {
                            if px.to_uppercase() != "PX" {
                                eprintln!(
                                    "wrong type of arguments for 'set' command; expecting PX, got {}", px
                                );
                                return None;
                            }
                            let px = match expiry_in_ms.parse::<u64>() {
                                Ok(px) => px,
                                Err(_) => {
                                    eprintln!("Unable to parse expiry_in_ms; got {}", expiry_in_ms);
                                    return None;
                                }
                            };
                            return Some(Command::Set(SetCommand::new(
                                key.clone(),
                                value.clone(),
                                Some(px),
                            )));
                        }
                        _ => {
                            eprintln!("wrong type of arguments for 'set' command");
                            return None;
                        }
                    }
                }
                eprintln!("wrong type of arguments for 'set' command; got {:?}", args);
                None
            }
            "GET" => {
                if args.len() != 1 {
                    eprintln!("wrong number of arguments for 'get' command");
                    return None;
                }
                match &args[0] {
                    Value::String(key) => Some(Command::Get(key.clone())),
                    _ => {
                        eprintln!("wrong type of arguments for 'get' command");
                        None
                    }
                }
            }
            _ => {
                eprintln!("unknown command '{}'", name);
                None
            }
        }
    }

    pub fn handle_command(value: &[Value]) -> Option<Command> {
        let command_name = &value[0];
        let command_args = &value[1..];
        match command_name {
            Value::String(name) => Command::process(name, command_args),
            _ => {
                eprintln!("unexpected token: {:?}", command_name);
                None
            }
        }
    }
}
