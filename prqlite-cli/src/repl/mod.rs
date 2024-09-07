mod commands;
mod consts;
mod normal;
mod traits;


use std::str::FromStr;

use crate::utils::row_value_parser;

use commands::Commands;
use commands::ExecCommands;
use normal::*;
use traits::*;

use anyhow::{anyhow, Result};
use comfy_table::presets::UTF8_FULL;
use comfy_table::ContentArrangement;
use comfy_table::Table;
use prqlite_rs::Prqlite;

const DEFAULT_PROMPT: &str = ">";
const DEFAULT_COMMAND_PREFIX: &str = ".";

#[derive(Default, Clone, Copy)]
enum ReplMode {
    #[default]
    Normal,
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
    pub fn normal() -> ReplBuilder {
        ReplBuilder {
            prompt: None,
            command_prefix: None,
            mode: Some(ReplMode::Normal),
            state: None,
        }
    }

    pub async fn run(&self) -> Result<()> {
        match self.mode {
            ReplMode::Normal => {
                NormalRepl::new(&self.prompt, &self.command_prefix, self.state).run()
            }
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
        let mut repl_state = ReplState::new();
        repl_state.set_conn(conn).unwrap();
        self.state = Some(repl_state);
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

#[derive(Debug)]
pub struct ReplState {
    pub prqlite_conn: Option<Prqlite>,
}
impl<'a> ReplState {
    pub fn new() -> Self {
        ReplState { prqlite_conn: None }
    }
    pub fn set_conn(&mut self, path: &str) -> Result<&mut Self> {
        self.prqlite_conn = Some(Prqlite::open(path)?);
        Ok(self)
    }
    pub fn get_prqlite_conn(&self) -> Result<&Prqlite> {
        if let Some(conn) = self.prqlite_conn.as_ref() {
            return Ok(conn);
        }
        Err(anyhow!(
            "Didn't connected with database, please restart program with '--open <DATABASE_FILE>' flag."
        ))
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
        let conn = self.state.get_prqlite_conn();
        if let Err(err) = conn {
            return Err(err);
        }
        match conn.unwrap().execute(buf) {
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
                Ok(table.lines().collect::<Vec<String>>().join("\n"))
            }
            Err(err) => Err(err),
        }
    }
}
