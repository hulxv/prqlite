use clap::Parser;
#[derive(Parser)]
#[clap(author, version, about)]
pub struct Args {
    /// Open database file
    #[arg(short, long, default_value_t = String::from(":memory:"))]
    pub open: String,
}

// #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default)]
// pub enum ReplMode {
//     #[default]
//     Simple,
// }
// impl std::str::FromStr for ReplMode {
//     type Err = String;
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         use ReplMode::*;
//         match s.to_lowercase().as_str() {
//             "simple" => Ok(Simple),
//             "tui" => Ok(Tui),
//             _ => Err("type help to show avaliable option.".to_owned()),
//         }
//     }
// }
