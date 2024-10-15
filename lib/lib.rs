#[macro_use]
extern crate election_derive;

use crate::sql_db::MySQLDB;

pub mod database;
pub mod election_models;
#[cfg(feature = "eml-5")]
pub mod eml_5;
pub mod sql_db;

pub trait PostGresObj {
    fn postgres_create() -> String;
    fn postgres_drop() -> String;
}

pub trait SerialiseDB {
    async fn insert(&self, database: &mut MySQLDB) -> String;
}

#[derive(SerialiseDB)]
#[db(table_name = "tests")]
struct TestStruct {
    test: String,
}

#[derive(SerialiseDB)]
#[db(table_name = "tests", insert = "aa")]
struct TestStructMultipleAttr {
    test: String,
}

#[derive(SerialiseDB)]
struct TestStructNoTest {
    test: String,
    #[db(column_name="tests_column", null_value="null".to_string())]
    test2: Option<String>,
}
