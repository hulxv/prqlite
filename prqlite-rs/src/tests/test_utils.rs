use super::*;
use rand::{distributions::Alphanumeric, Rng};
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

pub fn reset_database(db_path: &str, data: TYPE) {
    if Path::new(db_path).exists() {
        remove_file(db_path).unwrap();
    }
    File::create(db_path).unwrap();
    let conn = Prqlite::open(db_path).unwrap();
    match conn.execute_with_sql(
        r#"CREATE TABLE Persons (
                    ID integer,
                    Name varchar(255),
                    Address varchar(255)
                );"#,
    ) {
        Ok(mut stmt) => {
            let i = stmt.execute([]).unwrap();
            println!("{i}");
        }
        Err(err) => println!("error : {err}"),
    };

    for (id, name, address) in data.clone() {
        let sql = &format!(
            "INSERT INTO persons VALUES ('{}', '{}', '{}')",
            id, name, address,
        );
        let mut stmt = conn.execute_with_sql(sql).unwrap();
        stmt.execute([]).unwrap();
    }
}
