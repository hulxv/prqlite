#[cfg(test)]
mod tests;

use anyhow::Result;
use prql_compiler::{compile, Options};
use rusqlite::{Connection, Statement};

#[derive(Debug)]
pub struct Prqlite {
    conn: Connection,
}
impl Prqlite {
    pub fn open(path: &str) -> Result<Self> {
        Ok(Self {
            conn: Connection::open(path)?,
        })
    }
    pub fn execute(&self, prql: &str) -> Result<Statement> {
        let sql = compile(prql, &Options::default().no_format().no_signature())?;
        let stmt = self.conn.prepare(&sql)?;
        Ok(stmt)
    }
    pub fn execute_with_sql(&self, sql: &str) -> Result<Statement> {
        let stmt = self.conn.prepare(sql)?;
        Ok(stmt)
    }
    pub fn execute_batch(&self, prql: &str) -> Result<()> {
        let mut queries: Vec<String> = vec![];
        for query in prql.split(";").into_iter() {
            let sql = compile(query, &Options::default().no_format().no_signature())?;
            queries.push(sql);
        }
        Ok(self.conn.execute_batch(&queries.join(";"))?)
    }
    pub fn execute_batch_with_sql(&self, sql: &str) -> Result<()> {
        Ok(self.conn.execute_batch(sql)?)
    }
    pub fn get_conn(&self) -> Option<&str> {
        self.conn.path()
    }
}
