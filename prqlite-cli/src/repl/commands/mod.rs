use anyhow::{anyhow, Error, Result};
use comfy_table::{
    presets::{NOTHING, UTF8_FULL},
    Cell, ContentArrangement, Table,
};
use prql_compiler::{compile, Options};
use rusqlite::Statement;
use std::{cell::RefCell, rc::Rc, str::FromStr, string::ToString};

use crate::{utils::row_value_parser, ReplState};

pub trait ExecCommands {
    type Output;
    fn exec(&self, state: &ReplState) -> Result<Self::Output>;
}
pub trait ExecSingleCommand {
    type Output;
    fn exec(&self) -> Result<Self::Output>;
}

pub enum Commands {
    Help,
    Quit,
    Exit { code: i32 },
    Compile { input: String },
    Sql { input: String },
}

impl ToString for Commands {
    fn to_string(&self) -> String {
        use Commands::*;
        match self {
            Quit => "quit".to_owned(),
            Exit { code } => format!("exit {code}"),
            Compile { input } => format!("compile {input}"),
            Sql { input } => format!("sql {input}"),
            Help => "help".to_owned(),
        }
    }
}

impl FromStr for Commands {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Commands::*;

        let mut args = s.split_whitespace().collect::<Vec<&str>>();
        if args.is_empty() {
            return Err(anyhow!(
                "no commands passed, type .help to show available commands."
            ));
        }
        match args[0] {
            "quit" | "q" => Ok(Quit),
            "compile" => {
                if args.len() <= 1 {
                    return Err(anyhow!(
                        "no args was passed, you should pass PRQL query to compile to into SQL."
                    ));
                }

                Ok(Compile {
                    input: args.drain(1..).map(|s| s.to_string() + " ").collect(),
                })
            }
            "sql" => {
                if args.len() <= 1 {
                    return Err(anyhow!(
                        "no args was passed, you should pass SQL query to execute it."
                    ));
                }

                Ok(Sql {
                    input: args.drain(1..).map(|s| s.to_string() + " ").collect(),
                })
            }
            "exit" => {
                if args.len() <= 1 {
                    return Err(anyhow!("no args was passed, you should pass exit code or use '.q' command to exit program with success exit code."));
                }
                if let Ok(code) = args[1].parse() {
                    return Ok(Exit { code });
                }
                Err(anyhow!("exit code must be integer."))
            }
            "help" => Ok(Help),
            e => Err(anyhow!(
                "command not found: '{e}' , type ':help' to show avaliable commands."
            )),
        }
    }
}

impl ExecCommands for Commands {
    type Output = String;
    fn exec(&self, state: &ReplState) -> Result<Self::Output> {
        match self {
            Commands::Help => Help.exec(),
            Commands::Compile { input } => {
                Compile::new(input).exec()
            }
            Commands::Sql { input } => match state.get_prqlite_conn()?.execute_with_sql(input) {
                Ok(stmt) => Sql::new(stmt).exec(),
                Err(err) => Err(anyhow!("{err}")),
            },
            Commands::Exit { code } => {
                println!("Program exit with {code}");
                std::process::exit(*code);
            }
            Commands::Quit => Quit.exec(),
        }
    }
}

struct Help;

impl ExecSingleCommand for Help {
    type Output = String;
    fn exec(&self) -> Result<Self::Output> {
        let mut table = Table::new();
        table
            .load_preset(NOTHING)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_width(80)
            .set_header(vec![
                Cell::new("Command"),
                Cell::new("Args"),
                Cell::new("Description"),
            ])
            .add_row(vec![
                Cell::new("quit"),
                Cell::new(""),
                Cell::new("Exit PRQLite program"),
            ])
            .add_row(vec![
                Cell::new("compile"),
                Cell::new("<PRQL_QUERY>"),
                Cell::new("Compile PRQL into SQL"),
            ])
            .add_row(vec![
                Cell::new("exit"),
                Cell::new("<CODE>"),
                Cell::new("Exit PRQLite program with custom exit code"),
            ])
            .add_row(vec![
                Cell::new("sql"),
                Cell::new("<SQL_QUERY>"),
                Cell::new("Execute SQL query instead of PRQL"),
            ]);

        Ok(format!("{table}"))
    }
}

struct Quit;
impl ExecSingleCommand for Quit {
    type Output = String;
    fn exec(&self) -> Result<Self::Output> {
        std::process::exit(0);
    }
}

struct Sql<'a> {
    stmt: Rc<RefCell<Statement<'a>>>,
}
impl<'a> Sql<'a> {
    pub fn new(stmt: Statement<'a>) -> Self {
        Self { stmt: Rc::new(RefCell::new(stmt)) }
    }
}

impl<'a> ExecSingleCommand for Sql<'a> {
    type Output = String;
    fn exec(&self) -> Result<Self::Output> {
        let mut table = Table::new();
        let stmt = Rc::clone(&self.stmt);
        let mut stmt = stmt.borrow_mut();
        let column_names = stmt.column_names();
        let column_count = stmt.column_count();

        if stmt.column_count() > 0 && stmt.readonly() {
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_width(80)
                .set_header(column_names);

            let mut rows = stmt.query([]).unwrap();

            while let Some(row) = rows.next().unwrap() {
                let mut idx = 0;
                let mut row_content: Vec<String> = vec![];

                while idx < column_count {
                    row_content.push(row_value_parser(row, idx).unwrap());
                    idx += 1;
                }
                table.add_row(row_content);
            }
            Ok(format!("{table}"))
        } else {
            let effected_rows = stmt.execute([]).unwrap();
            Ok(format!(
                "{effected_rows} row{} effected",
                if effected_rows > 1 { "s" } else { "" }
            ))
        }
    }
}

struct Compile {
    input: String,
}

impl Compile {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_owned(),
        }
    }
}
impl ExecSingleCommand for Compile {
    type Output = String;
    fn exec(&self) -> Result<Self::Output> {
        let opt = Options::default().no_format().no_signature();
        match compile(&self.input, &opt) {
            Err(e) => Err(anyhow!("Cannot compile your query into SQL: \n{e}")),
            Ok(sql) => Ok(format!(
                "{}",
                sql.replace("\n", " ")
                    .split_whitespace()
                    .filter_map(|e| {
                        if e.is_empty() {
                            return None;
                        }
                        let mut e = e.to_string();
                        e.push_str(" ");
                        Some(e)
                    })
                    .collect::<String>(),
            )),
        }
    }
}
