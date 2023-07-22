use crate::repl::commands::{Commands, ExecCommand};
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use prql_compiler::{compile, Options};

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
                let opt = Options::default().no_format().no_signature();
                match compile(&input, &opt) {
                    Err(e) => eprintln!("Cannot compile your query into SQL: \n{e}"),
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
