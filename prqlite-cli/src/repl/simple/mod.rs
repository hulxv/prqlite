mod commands;

use anyhow::Result;
use commands::Commands;
use std::{
    io::{stdin, stdout, Write},
    str::FromStr,
};
pub struct SimpleRepl {
    prompt: String,
    command_prefix: String,
}

impl SimpleRepl {
    pub fn new<T: ToString>(prompt: T, command_prefix: T) -> Self {
        Self {
            prompt: prompt.to_string(),
            command_prefix: command_prefix.to_string(),
        }
    }

    pub fn run(&self) -> Result<()> {
        println!(
            r#"                           Welcome to PRQLite!   
type ".help" to show avaliable commands, or start typing queries and ENJOY !
"#
        );

        let stdin = stdin();
        let mut buf = String::new();
        loop {
            print!("{} ", self.prompt);
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
