use super::commands::{Commands, ExecCommand};
use anyhow::Result;
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use prql_compiler::compile;
use std::{
    io::{stdin, stdout, Write},
    str::FromStr,
};
impl ExecCommand for Commands {
    type Output = ();
    fn exec(&self) -> Self::Output {
        use Commands::*;
        match self {
            Exit { code } => {
                println!("Program exit with {code}");
                std::process::exit(*code);
            }
            Quit => std::process::exit(0),
            Help => {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
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
                        Cell::new("<PRQL>"),
                        Cell::new("Compile PRQL into SQL"),
                    ])
                    .add_row(vec![
                        Cell::new("exit"),
                        Cell::new("<CODE>"),
                        Cell::new("Exit PRQLite program with custom exit code"),
                    ]);

                println!("{table}");
            }
            Compile { input } => {
                match compile(&input) {
                    Err(e) => eprintln!("{e}"),
                    Ok(sql) => println!(
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
                    ),
                };
            }
        }
    }
}
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
