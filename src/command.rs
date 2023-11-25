use crate::response::Value;

pub enum Command {
    Ping,
    Echo,
}

impl Command {
    fn from_str(string: &str) -> Option<Command> {
        match string.to_uppercase().as_str() {
            "PING" => Some(Command::Ping),
            "ECHO" => Some(Command::Echo),
            _ => None,
        }
    }

    pub fn process(&self, args: &[Value]) -> Option<String> {
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
        }
    }

    pub fn handle_command(value: &[Value]) -> Option<String> {
        let command_name = &value[0];
        let command_args = &value[1..];
        match command_name {
            Value::String(string) => match Command::from_str(string) {
                Some(command) => command.process(command_args),
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
