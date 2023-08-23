use clap::Parser;
use prqlite_cli::{Args, Repl};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    Repl::normal().state(&args.open).build().run().await?;
    Ok(())
}
