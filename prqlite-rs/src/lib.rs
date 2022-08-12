use anyhow::Result;
use prql_compiler::compile;
use rusqlite::{Connection, Params};
pub struct Prqlite {
    conn: Connection,
}
impl Prqlite {
    pub fn open(path: &str) -> Result<Self> {
        Ok(Self {
            conn: Connection::open(path)?,
        })
    }
    pub fn execute(&self, prql: &str) -> Result<usize> {
        Ok(self.conn.execute(&compile(prql)?, [])?)
    }
    pub fn execute_with_sql<P: Params>(&self, sql: &str, params: P) -> Result<usize> {
        Ok(self.conn.execute(sql, params)?)
    }
    pub fn execute_batch(&self, prql: &str) -> Result<()> {
        let mut queries: Vec<String> = vec![];
        for query in prql.split(";").into_iter() {
            queries.push(compile(query)?);
        }
        Ok(self.conn.execute_batch(&queries.join(";"))?)
    }
    pub fn execute_batch_with_sql(&self, sql: &str) -> Result<()> {
        Ok(self.conn.execute_batch(sql)?)
    }
}
