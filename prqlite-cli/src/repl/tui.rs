pub struct TuiRepl {
    command_prefix: String,
}

impl TuiRepl {
    pub fn new<T: ToString>(command_prefix: T) -> Self {
        Self {
            command_prefix: command_prefix.to_string(),
        }
    }

    pub fn run(&self) {
        todo!("implement tui repl runner")
    }
}
