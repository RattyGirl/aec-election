use mongodb::bson::Bson::ObjectId;
use mongodb::bson::{oid, Document};
use mongodb::options::ClientOptions;
use mongodb::sync::{Client, Cursor, Database};
use serde::Serialize;
use std::str::FromStr;

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

    fn list_tables(&self) -> Vec<String> {
        self.database.list_collection_names(None).unwrap()
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

    fn many_to_many_connection(
        &self,
        table_a: &str,
        table_b: &str,
        object_a: &str,
        object_b: &str,
    ) {
        let mut relation = Document::new();
        relation.insert(
            table_a,
            ObjectId(oid::ObjectId::from_str(object_a).unwrap()),
        );
        relation.insert(
            table_b,
            ObjectId(oid::ObjectId::from_str(object_b).unwrap()),
        );
        self.insert_one(format!("{}_{}", table_a, table_b).as_str(), relation);
    }
}
