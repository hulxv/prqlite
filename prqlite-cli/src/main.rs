use ::prqlite_cli::{
    Repl,
    {Args, ReplMode::*},
};
use clap::Parser;

fn main() {
    let args = Args::parse();
    match args.mode.unwrap_or_default() {
        Simple => Repl::simple(),
        Tui => Repl::tui(),
    }
    .prompt("prqlite >")
    .build()
    .run();
}
