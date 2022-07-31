mod commands;
mod simple;
mod tui;

use self::simple::*;
use self::tui::*;

use std::{
    io::{stdin, stdout, Write},
    str::FromStr,
};

const DEFAULT_PROMPT: &str = ">";
const DEFAULT_COMMAND_PREFIX: &str = ".";

#[derive(Default, Clone, Copy)]
enum ReplMode {
    #[default]
    Simple,
    Tui,
}

pub struct Repl {
    prompt: String,
    command_prefix: String,
    mode: ReplMode,
}

pub struct ReplBuilder {
    prompt: Option<String>,
    command_prefix: Option<String>,
    mode: Option<ReplMode>,
}

impl Repl {
    pub fn new() -> ReplBuilder {
        ReplBuilder {
            prompt: None,
            command_prefix: None,
            mode: None,
        }
    }
    pub fn simple() -> ReplBuilder {
        ReplBuilder {
            prompt: None,
            command_prefix: None,
            mode: Some(ReplMode::Simple),
        }
    }
    pub fn tui() -> ReplBuilder {
        ReplBuilder {
            prompt: None,
            command_prefix: None,
            mode: Some(ReplMode::Tui),
        }
    }
    pub fn run(&self) {
        use ReplMode::*;
        match self.mode {
            Simple => SimpleRepl::new(&self.prompt, &self.command_prefix).run(),
            Tui => TuiRepl::new(&self.command_prefix).run(),
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
            mode: self.mode.unwrap_or_default(),
            command_prefix: self
                .command_prefix
                .clone()
                .unwrap_or(DEFAULT_COMMAND_PREFIX.to_string()),
        }
    }
}
