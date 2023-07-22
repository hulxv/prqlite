use crate::{
    repl::commands::{Commands, ExecCommand},
    utils::row_value_parser,
    ReplState,
};
use comfy_table::{
    presets::{NOTHING, UTF8_FULL},
    Cell, ContentArrangement, Table,
};
use prql_compiler::{compile, Options};
use prqlite_rs::Prqlite;

impl ExecCommand for Commands {
    type Output = ();
    fn exec(&self, state: &ReplState) -> Self::Output {
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
                        Cell::new("compile"),
                        Cell::new("<SQL_QUERY>"),
                        Cell::new("Execute SQL query instead of PRQL"),
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
            Sql { input } => {
                match state.prqlite_conn.execute_with_sql(input) {
                    Ok(stmt) => {
                        let mut table = Table::new();
                        let mut stmt = stmt;
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
                            println!("{table}");
                        } else {
                            let effected_rows = stmt.execute([]).unwrap();
                            println!(
                                "{effected_rows} row{} effected",
                                if effected_rows > 1 { "s" } else { "" }
                            )
                        }
                    }
                    Err(err) => eprintln!("{err}"),
                };
            }
        }
    }
}
