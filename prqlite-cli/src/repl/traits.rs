use anyhow::Result;
pub trait Runner {
    fn run(&self) -> Result<()>;
}
