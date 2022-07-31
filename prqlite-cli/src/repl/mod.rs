mod commands;
use commands::*;

use std::{
    io::{stdin, stdout, Write},
    str::FromStr,
};

const DEFAULT_PROMPT: &str = ">";
const DEFAULT_COMMAND_PREFIX: &str = ".";

pub struct Repl {
    prompt: String,
    command_prefix: String,
}

pub struct ReplBuilder {
    prompt: Option<String>,
    command_prefix: Option<String>,
}

impl Repl {
    pub fn new() -> ReplBuilder {
        ReplBuilder {
            prompt: None,
            command_prefix: None,
        }
    }

    pub fn run(&self) {
        let stdin = stdin();
        let mut buf = String::new();
        loop {
            print!("{}", self.prompt);
            stdout().flush().unwrap();

            stdin.read_line(&mut buf).unwrap();

            if buf.trim().starts_with(&self.command_prefix) {
                match Commands::from_str(&buf[1..]) {
                    Err(e) => println!("Error: {e}"),
                    Ok(cmd) => cmd.exec(),
                }
            } else {
                // TODO: Using quires here
            }

            buf.clear();
        }
    }
}

impl ReplBuilder {
    pub fn prompt(&mut self, prompt: &str) -> &mut Self {
        self.prompt = Some(prompt.to_string());
        self
    }
    pub fn command_prefix(&mut self, command_prefix: &str) -> &mut Self {
        self.command_prefix = Some(command_prefix.to_string());
        self
    }
    pub fn build(&self) -> Repl {
        Repl {
            prompt: self.prompt.clone().unwrap_or(DEFAULT_PROMPT.to_string()),
            command_prefix: self
                .command_prefix
                .clone()
                .unwrap_or(DEFAULT_COMMAND_PREFIX.to_string()),
        }
    }
}
