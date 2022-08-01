use ::prqlite_cli::{
    Repl,
    {Args, ReplMode::*},
};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let repl = match args.mode.unwrap_or_default() {
        Simple => Repl::simple().prompt("prqlite>").clone(),
        Tui => Repl::tui().prompt(">").clone(),
    };
    repl.build().run().await?;
    Ok(())
}
