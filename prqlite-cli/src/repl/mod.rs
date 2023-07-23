mod commands;
mod consts;
mod simple;
mod traits;
mod tui;

use std::str::FromStr;

use crate::utils::row_value_parser;

use self::commands::Commands;
use self::commands::ExecCommand;
use self::simple::*;
use self::traits::*;
use self::tui::*;

use anyhow::Result;
use comfy_table::presets::UTF8_FULL;
use comfy_table::ContentArrangement;
use comfy_table::Table;
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

pub struct ReplInputEvent<'a> {
    pub state: &'a ReplState,
}

impl<'a> ReplInputEvent<'a> {
    pub fn new(state: &'a ReplState) -> Self {
        Self { state }
    }
    pub fn on_command(&self, buf: &str) -> Result<String> {
        match Commands::from_str(&buf[1..]) {
            Err(e) => return Err(e),
            Ok(cmd) => match cmd.exec(self.state) {
                Ok(out) => Ok(out),
                Err(e) => return Err(e),
            },
        }
    }
    pub fn on_regular_input(&self, buf: &str) -> Result<String> {
        match self.state.prqlite_conn.execute(buf) {
            Ok(stmt) => {
                let mut table = Table::new();
                let mut stmt = stmt;
                let column_names = stmt.column_names();
                let column_count = stmt.column_count();

                table
                    .load_preset(UTF8_FULL)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_width(80)
                    .set_header(column_names);

                let mut rows = stmt.query([]).unwrap();

                while let Some(row) = rows.next()? {
                    let mut idx = 0;
                    let mut row_content: Vec<String> = vec![];

                    while idx < column_count {
                        row_content.push(row_value_parser(row, idx).unwrap());
                        idx += 1;
                    }
                    table.add_row(row_content);
                }
                Ok(table.to_string())
            }
            Err(err) => Err(err),
        }
    }
}
