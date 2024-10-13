use serde::Serialize;
use sqlx::postgres::any::AnyConnectionBackend;
use sqlx::postgres::PgQueryResult;
use sqlx::{Connection, Error, Executor, PgConnection};

pub struct MySQLDB {
    connection: PgConnection,
}

impl MySQLDB {
    pub async fn setup(connection_name: &str, _database_name: &str) -> Self {
        let mut conn = PgConnection::connect(connection_name).await.unwrap();
        conn.commit().await.unwrap();
        Self { connection: conn }
    }

    pub fn list_tables(&self) -> Vec<String> {
        Vec::from(["".to_string()])
    }

    pub async fn run_raw(&mut self, cmd: String) -> PgQueryResult {
        sqlx::query(cmd.as_str())
            .execute(&mut self.connection)
            .await
            .unwrap()
    }

    pub fn insert_one<T: Serialize>(&self, _table: &str, _object: T) -> Option<String> {
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

    pub fn many_to_many_connection(
        &self,
        _table_a: &str,
        _table_b: &str,
        _object_a: &str,
        _object_b: &str,
    ) {
        todo!()
    }
}
