use crate::database::CustomDB;
use serde::Serialize;
use sqlx::{Connection, PgConnection};

pub struct MySQLDB {
    connection: PgConnection,
}

impl MySQLDB {
    pub async fn setup(connection_name: &str, _database_name: &str) -> Self {
        let conn = PgConnection::connect(connection_name).await.unwrap();
        Self { connection: conn }
    }
}

impl CustomDB for MySQLDB {
    fn list_tables(&self) -> Vec<String> {
        todo!()
    }

    fn insert_one<T: Serialize>(&self, _table: &str, _object: T) -> Option<String> {
        todo!()
    }

    fn drop(&self, _table: &str) {
        todo!()
    }

    // fn find<T: Serialize>(&self, table: &str, filter: T) -> Cursor<T> {
    //     todo!()
    // }
    //
    // fn aggregate(&self, table: &str, pipeline: impl IntoIterator<Item=Document>) -> Cursor<Document> {
    //     todo!()
    // }

    fn many_to_many_connection(
        &self,
        _table_a: &str,
        _table_b: &str,
        _object_a: &str,
        _object_b: &str,
    ) {
        todo!()
    }
}
