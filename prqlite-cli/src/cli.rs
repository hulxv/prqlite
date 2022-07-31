use clap::{Parser, ValueEnum};
#[derive(Parser)]
#[clap(author, version, about)]
pub struct Args {
    /// Select repl mode (default: simple)
    #[clap(long, arg_enum)]
    pub mode: Option<ReplMode>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default)]
// #[clap(ValueEnum)]
pub enum ReplMode {
    #[default]
    Simple,
    Tui,
}
impl std::str::FromStr for ReplMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ReplMode::*;
        match s.to_lowercase().as_str() {
            "simple" => Ok(Simple),
            "tui" => Ok(Tui),
            _ => Err("type help to show avaliable option.".to_owned()),
        }
    }
}
