use serde::Serialize;

pub trait CustomDB {
    fn list_tables(&self) -> Vec<String>;
    fn insert_one<T: Serialize>(&self, table: &str, object: T) -> Option<String>;
    fn drop(&self, table: &str);
    // fn find<T: Serialize>(&self, table: &str, filter: T) -> Cursor<T>;
    // fn aggregate(
    //     &self,
    //     table: &str,
    //     pipeline: impl IntoIterator<Item = Document>,
    // ) -> Cursor<Document>;

    fn many_to_many_connection(&self, table_a: &str, table_b: &str, object_a: &str, object_b: &str);
}

#[derive(Serialize)]
pub struct ElectionContests<'a> {
    pub(crate) election: &'a str,
    pub(crate) contests: &'a str,
}
