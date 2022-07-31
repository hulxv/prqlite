use ::prqlite_cli::Repl;

fn main() {
    Repl::new().prompt("prqlite >").build().run();
}
