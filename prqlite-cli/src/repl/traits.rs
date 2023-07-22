use anyhow::Result;
pub trait Runner {
    fn run(&self) -> Result<()>;
    fn on_command(&self, buf: &str) -> Result<()>;
    fn on_regular_input(&self, buf: &str) -> Result<()>;
}
