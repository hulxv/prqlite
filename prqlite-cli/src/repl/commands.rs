use comfy_table::{presets::UTF8_FULL, *};
use std::{str::FromStr, string::ToString};

pub enum Commands {
    Help,
    Quit,
    Exit { code: i32 },
}

impl Commands {
    pub fn exec(&self) {
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
                    .set_header(vec![Cell::new("Command"), Cell::new("Description")])
                    .add_row(vec![Cell::new("quit"), Cell::new("Exit PRQLite program")])
                    .add_row(vec![
                        Cell::new("exit <CODE>"),
                        Cell::new("Exit PRQLite program with custom exit code"),
                    ]);

                println!("{table}");
            }
        }
    }
}

impl ToString for Commands {
    fn to_string(&self) -> String {
        use Commands::*;
        match self {
            Quit => "quit".to_owned(),
            Exit { code } => format!("exit {code}"),
            Help => "help".to_owned(),
        }
    }
}

impl FromStr for Commands {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Commands::*;

        let args = s.trim().split_whitespace().collect::<Vec<&str>>();

        match args[0] {
            "quit" | "q" => Ok(Quit),
            "exit" => {
                if args.len() <= 1 {
                    return Err("no args passing, you should passing exit code or use '.q' command to exit program with success exit code.".to_owned());
                }
                if let Ok(code) = args[1].parse() {
                    return Ok(Exit { code });
                }
                return Err("exit code must be integer.".to_owned());
            }
            "help" => Ok(Help),
            e => Err(format!(
                "command not found: '{e}' , type ':help' to show avaliable commands."
            )),
        }
    }
}
