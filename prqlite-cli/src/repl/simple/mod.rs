mod command;

use super::{
    commands::{Commands, ExecCommand},
    consts::PRQLITE_VERSION,
    traits::Runner,
};
use crate::ReplState;
use anyhow::Result;

use comfy_table::{presets::UTF8_FULL, ContentArrangement, Table};
use crossterm::style::Stylize;
// use rusqlite::;
use prql_compiler::PRQL_VERSION;
use prqlite_rs::Prqlite;

use rusqlite::{types::ValueRef::*, Row};
use std::{
    io::{stdin, stdout, Write},
    rc::Rc,
    str::{from_utf8, FromStr},
};

pub struct SimpleRepl<'a> {
    prompt: String,
    command_prefix: String,
    state: &'a ReplState,
}

impl<'a> SimpleRepl<'a> {
    pub fn new<T: ToString>(prompt: T, command_prefix: T, state: &'a ReplState) -> Self {
        Self {
            prompt: prompt.to_string(),
            command_prefix: command_prefix.to_string(),
            state,
        }
    }
}

impl<'a> Runner for SimpleRepl<'a> {
    fn run(&self) -> Result<()> {
        println!(
            r#"                           W
            Welcome to PRQLite!   
type ".help" to show avaliable commands, or start typing queries and ENJOY !
PRQL version: {:?}
Prqlite version: {}

"#,
            PRQL_VERSION.to_string(),
            PRQLITE_VERSION
        );

        let stdin = stdin();
        let mut buf = String::new();
        loop {
            print!("{} ", self.prompt);
            stdout().flush().unwrap();

            stdin.read_line(&mut buf).unwrap();

            if buf.trim().chars().last().unwrap() != ';' {
                print!("..~");
                continue;
            }
            buf = buf
                .trim()
                .to_owned()
                .replacen(";", "", buf.rfind(";").unwrap());

            let exec = match buf.trim().starts_with(&self.command_prefix) {
                true => self.on_command(&buf),
                false => self.on_regular_input(&buf),
            };

            if let Err(err) = exec {
                eprintln!("\x1b[93m{}\x1b[0m", err.to_string());
            }

            buf.clear();
        }
    }

    fn on_command(&self, buf: &str) -> Result<()> {
        match Commands::from_str(&buf[1..]) {
            Err(e) => return Err(e),
            Ok(cmd) => cmd.exec(),
        }
        Ok(())
    }
    fn on_regular_input(&self, buf: &str) -> Result<()> {
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
                println!("{table}");
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}

fn row_value_parser(row: &Row, idx: usize) -> Result<String> {
    let column_type = row.get_ref_unwrap(idx);
    let out: String = match column_type {
        Null => "-".to_owned(),
        Integer(v) => v.to_string(),
        Blob(v) => format!("{:?}", v),
        Text(v) => from_utf8(v).unwrap().to_owned(),
        Real(v) => v.to_string(),
    };
    Ok(out)
}
