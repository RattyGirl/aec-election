use serde::Serialize;



use mongodb::bson::{Document};

use mongodb::sync::{Cursor};

pub trait CustomDB {
    fn setup(connection_name: &str, database_name: &str) -> Self;
    fn list_tables(&self) -> Vec<String>;
    fn insert_one<T: Serialize>(&self, table: &str, object: T) -> Option<String>;
    fn drop<T: Serialize>(&self, table: &str);
    fn find<T: Serialize>(&self, table: &str, filter: impl Into<Option<Document>>) -> Cursor<T>;
    fn aggregate(
        &self,
        table: &str,
        pipeline: impl IntoIterator<Item = Document>,
    ) -> Cursor<Document>;

    fn many_to_many_connection(&self, table_a: &str, table_b: &str, object_a: &str, object_b: &str);
}