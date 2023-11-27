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
    Config(String),
    Keys(String),
}

impl Command {
    pub fn process(name: &str, args: &[Value]) -> Option<Command> {
        match name.to_uppercase().as_str() {
            "PING" => Some(Command::Ping("PONG".to_string())),
            "ECHO" => Some(Command::Echo(
                args.iter()
                    .map(|arg| match arg {
                        Value::String(string) => string,
                        _ => "",
                    })
                    .collect::<Vec<&str>>()
                    .join(" "),
            )),
            "SET" => {
                if args.len() > 4 || args.len() < 2 {
                    eprintln!("SET command requires 2 or 4 arguments; got {}", args.len());
                    return None;
                }

                if args.len() == 2 {
                    match (&args[0], &args[1]) {
                        (Value::String(key), Value::String(value)) => {
                            return Some(Command::Set(SetCommand::new(
                                key.to_string(),
                                value.to_string(),
                                None,
                            )))
                        }
                        _ => {
                            eprintln!("Wrong type of arguments for 'SET' command; got {:?}", args);
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
                                    "Wrong type of arguments for 'SET' command; expecting PX, got {}", px
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
                                key.to_string(),
                                value.to_string(),
                                Some(px),
                            )));
                        }
                        _ => {
                            eprintln!("Wrong type of arguments for 'SET' command; got {:?}", args);
                            return None;
                        }
                    }
                }
                None
            }
            "GET" => {
                if args.len() != 1 {
                    eprintln!(
                        "Wrong number of arguments for 'GET' command; got {}",
                        args.len()
                    );
                    return None;
                }
                match &args[0] {
                    Value::String(key) => Some(Command::Get(key.clone())),
                    _ => {
                        eprintln!("Wrong type of arguments for 'GET' command; got {:?}", args);
                        None
                    }
                }
            }
            "CONFIG" => match args[0].to_string().to_uppercase().as_str() {
                "GET" => match &args[1] {
                    Value::String(config_value) => Some(Command::Config(config_value.clone())),
                    _ => {
                        eprintln!(
                            "Wrong type of arguments for 'CONFIG' command; got {:?}",
                            args
                        );
                        None
                    }
                },
                _ => {
                    eprintln!("Unknown command '{}'; expecting CONFIG GET", name);
                    None
                }
            },

            "KEYS" => {
                if args.len() != 1 {
                    eprintln!(
                        "Wrong number of arguments for 'KEYS' command; got {}",
                        args.len()
                    );
                    return None;
                }
                match &args[0] {
                    Value::String(key) => Some(Command::Keys(key.clone())),
                    _ => {
                        eprintln!("Wrong type of arguments for 'KEYS' command; got {:?}", args);
                        None
                    }
                }
            }

            _ => {
                eprintln!(
                    "Unknown command '{}'; expecting PING, ECHO, SET, GET, CONFIG, KEYS",
                    name
                );
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
                eprintln!("Unexpected command: {:?}", command_name);
                None
            }
        }
    }
}
