use anyhow::{anyhow, Error};
use std::{str::FromStr, string::ToString};
pub trait ExecCommand {
    type Output;
    fn exec(&self) -> Self::Output;
}
pub enum Commands {
    Help,
    Quit,
    Exit { code: i32 },
    Compile { input: String },
}

impl ToString for Commands {
    fn to_string(&self) -> String {
        use Commands::*;
        match self {
            Quit => "quit".to_owned(),
            Exit { code } => format!("exit {code}"),
            Compile { input } => format!("compile {input}"),
            Help => "help".to_owned(),
        }
    }
}

impl FromStr for Commands {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Commands::*;

        let mut args = s.split_whitespace().collect::<Vec<&str>>();

        match args[0] {
            "quit" | "q" => Ok(Quit),
            "compile" => {
                if args.len() <= 1 {
                    return Err(anyhow!(
                        "no args passing, you should passing PRQL query to compile to into SQL."
                    ));
                }

                Ok(Compile {
                    input: args.drain(1..).map(|s| s.to_string() + " ").collect(),
                })
            }
            "exit" => {
                if args.len() <= 1 {
                    return Err(anyhow!("no args passing, you should passing exit code or use '.q' command to exit program with success exit code."));
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
