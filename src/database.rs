use mongodb::bson::Document;
use mongodb::options::ClientOptions;
use mongodb::sync::{Client, Cursor, Database};
use serde::Serialize;

pub trait CustomDB {
    fn setup(connection_name: &str, database_name: &str) -> Self;
    fn insert_one<T: Serialize>(&self, table: &str, object: T) -> Option<String>;
    fn drop<T: Serialize>(&self, table: &str);
    fn find<T: Serialize>(&self, table: &str, filter: impl Into<Option<Document>>) -> Cursor<T>;
    fn aggregate(
        &self,
        table: &str,
        pipeline: impl IntoIterator<Item = Document>,
    ) -> Cursor<Document>;
}

pub struct MongoDB {
    database: Database,
}

impl CustomDB for MongoDB {
    fn setup(connection_name: &str, database_name: &str) -> Self {
        let mut client_options = ClientOptions::parse(connection_name).unwrap();
        client_options.app_name = Some("AEC Election History".to_string());
        let client = Client::with_options(client_options).unwrap();
        Self {
            database: client.database(database_name),
        }
    }

    fn insert_one<T: Serialize>(&self, table: &str, object: T) -> Option<String> {
        let result = self
            .database
            .collection::<T>(table)
            .insert_one(object, None);

        result
            .ok()
            .map(|x| x.inserted_id.as_object_id().unwrap().to_string())
    }

    fn drop<T: Serialize>(&self, table: &str) {
        self.database.collection::<T>(table).drop(None).unwrap();
    }

    fn find<T: Serialize>(&self, table: &str, filter: impl Into<Option<Document>>) -> Cursor<T> {
        self.database
            .collection::<T>(table)
            .find(filter, None)
            .unwrap()
    }

    fn aggregate(
        &self,
        table: &str,
        pipeline: impl IntoIterator<Item = Document>,
    ) -> Cursor<Document> {
        self.database
            .collection::<Document>(table)
            .aggregate(pipeline, None)
            .unwrap()
    }
}
