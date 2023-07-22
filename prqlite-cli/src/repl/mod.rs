mod commands;
mod consts;
mod simple;
mod traits;
mod tui;

use self::simple::*;
use self::traits::*;
use self::tui::*;

use anyhow::Result;
use prqlite_rs::Prqlite;

const DEFAULT_PROMPT: &str = ">";
const DEFAULT_COMMAND_PREFIX: &str = ".";

#[derive(Default, Clone, Copy)]
enum ReplMode {
    #[default]
    Simple,
    Tui,
}

// #[derive(Clone)]
pub struct Repl<'a> {
    prompt: String,
    command_prefix: String,
    mode: ReplMode,
    pub state: &'a ReplState,
}
// #[derive(Clone)]

pub struct ReplBuilder {
    prompt: Option<String>,
    command_prefix: Option<String>,
    mode: Option<ReplMode>,
    state: Option<ReplState>,
}

impl<'a> Repl<'a> {
    pub fn new() -> ReplBuilder {
        ReplBuilder {
            prompt: None,
            command_prefix: None,
            mode: None,
            state: None,
        }
    }
    pub fn simple() -> ReplBuilder {
        ReplBuilder {
            prompt: None,
            command_prefix: None,
            mode: Some(ReplMode::Simple),
            state: None,
        }
    }
    pub fn tui() -> ReplBuilder {
        ReplBuilder {
            prompt: None,
            command_prefix: None,
            mode: Some(ReplMode::Tui),
            state: None,
        }
    }
    pub async fn run(&self) -> Result<()> {
        use ReplMode::*;
        match self.mode {
            Simple => SimpleRepl::new(&self.prompt, &self.command_prefix, self.state.clone()).run(),
            Tui => TuiRepl::new(&self.prompt, &self.command_prefix).run().await,
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
    pub fn state(&mut self, conn: &str) -> &mut Self {
        self.state = Some(
            ReplState::new(conn)
                .map_err(|err| {
                    eprintln!("Error: cannot open database: {err}");
                    std::process::exit(1);
                })
                .unwrap(),
        );
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
            state: self.state.as_ref().unwrap(),
        }
    }
}
pub struct ReplState {
    pub prqlite_conn: Prqlite,
}
impl ReplState {
    pub fn new(path: &str) -> Result<Self> {
        Ok(Self {
            prqlite_conn: Prqlite::open(path)?,
        })
    }
}
