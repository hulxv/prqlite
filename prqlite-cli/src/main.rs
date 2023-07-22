use clap::Parser;
use prqlite_cli::{
    Repl,
    {Args, ReplMode::*},
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    match args.mode.unwrap_or_default() {
        Simple => Repl::simple(),
        Tui => Repl::tui(),
    }
    .state(&args.open)
    .build()
    .run()
    .await?;
    Ok(())
}
