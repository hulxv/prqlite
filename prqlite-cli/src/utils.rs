use anyhow::Result;
/// Random public functions used in different parts
use rusqlite::{types::ValueRef::*, Row};
use std::str::from_utf8;

/// Parse Sqlite value into string to display it.
pub fn row_value_parser(row: &Row, idx: usize) -> Result<String> {
    let column_type = row.get_ref_unwrap(idx);
    let out: String = match column_type {
        Null => "-".to_owned(),
        Integer(v) => v.to_string(),
        Blob(v) => format!("{:?}", v),
        Text(v) => from_utf8(v).unwrap().to_owned(),
        Real(v) => v.to_string(),
    };
    Ok(out)
}
