mod test_utils;
use test_utils::*;

use super::*;
use rand::{distributions::Alphanumeric, Rng};
use rusqlite::types::Value;
use std::{
    fs::{remove_file, File},
    path::Path,
};
#[test]
fn test_execute() {
    let db_path = "test.db";
    let fake_data = generate_fake_data(5);

    reset_database(db_path, fake_data.clone());

    let conn = Prqlite::open("test.db").unwrap();
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

#[test]
fn test_execute_with_sql() {
    let db_path = "test.db";
    let fake_data = generate_fake_data(5);

    reset_database(db_path, fake_data.clone());

    let conn = Prqlite::open("test.db").unwrap();
    let mut stmt = conn.execute_with_sql("SELECT * FROM persons").unwrap();

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
