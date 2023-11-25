use crate::response::Value;

pub enum Command {
    Ping(String),
    Echo(String),
    Set(String, String),
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
                if args.len() != 2 {
                    eprintln!("wrong number of arguments for 'set' command");
                    return None;
                }
                match (&args[0], &args[1]) {
                    (Value::String(key), Value::String(value)) => {
                        Some(Command::Set(key.clone(), value.clone()))
                    }
                    _ => {
                        eprintln!("wrong type of arguments for 'set' command");
                        None
                    }
                }
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
