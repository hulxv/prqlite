use anyhow::Result;
use prql_compiler::{compile, Options};
use rusqlite::{Connection, Params, Statement};
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
    pub fn execute_with_sql<P: Params>(&self, sql: &str, params: P) -> Result<usize> {
        Ok(self.conn.execute(sql, params)?)
    }
    pub fn execute_batch(&self, prql: &str) -> Result<()> {
        let mut queries: Vec<String> = vec![];
        for query in prql.split(";").into_iter() {
            let sql = compile(query, &Options::default().no_format().no_signature()).unwrap();
            queries.push(sql);
        }
        Ok(self.conn.execute_batch(&queries.join(";"))?)
    }
    pub fn execute_batch_with_sql(&self, sql: &str) -> Result<()> {
        Ok(self.conn.execute_batch(sql)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Alphanumeric, Rng};
    use rusqlite::types::Value;
    use std::{
        fs::{remove_file, File},
        path::Path,
    };
    type TYPE = Vec<(i32, String, String)>;
    pub fn generate_fake_data(i: i32) -> TYPE {
        let mut data: TYPE = vec![];
        let rng = rand::thread_rng();

        for _ in 0..i {
            data.push((
                rng.clone().gen(),
                rng.clone()
                    .sample_iter(&Alphanumeric)
                    .take(7)
                    .map(char::from)
                    .collect(),
                rng.clone()
                    .sample_iter(&Alphanumeric)
                    .take(7)
                    .map(char::from)
                    .collect(),
            ));
        }
        data
    }

    #[test]
    fn test_execute() {
        let db_path = "test.db";
        if Path::new(db_path).exists() {
            remove_file(db_path).unwrap();
        }
        File::create(db_path).unwrap();

        let conn = Prqlite::open("test.db").unwrap();
        conn.execute_with_sql(
            r#"CREATE TABLE Persons (
                    ID integer,
                    Name varchar(255),
                    Address varchar(255)
                );"#,
            [],
        )
        .unwrap();
        let fake_data = generate_fake_data(5);

        for (id, name, address) in fake_data.clone() {
            let sql = &format!(
                "INSERT INTO persons VALUES ('{}', '{}', '{}')",
                id, name, address,
            );
            conn.execute_with_sql(sql, []).unwrap();
        }

        let mut stmt = conn.execute("from persons").unwrap();

        let mut rows = stmt.query([]).unwrap();
        let mut idx = 0;

        while let Some(row) = rows.next().unwrap() {
            let (id, name, address) = fake_data.get(idx).unwrap();
            assert_eq!(row.get::<_, Value>(0).unwrap(), Value::from(*id));
            assert_eq!(
                row.get::<_, Value>(1).unwrap(),
                Value::from(name.to_owned())
            );
            assert_eq!(
                row.get::<_, Value>(2).unwrap(),
                Value::from(address.to_owned())
            );
            idx += 1;
        }
    }
}
