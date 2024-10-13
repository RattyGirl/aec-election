use crate::sql_db::MySQLDB;
use crate::PostGresObj;

pub async fn generate_tables(database: &mut MySQLDB) {
    database.run_raw(ElectionEvent::postgres_create()).await;
    database.run_raw(Election::postgres_create()).await;
}

pub async fn drop_tables(database: &mut MySQLDB) {
    database.run_raw(ElectionEvent::postgres_drop()).await;
    database.run_raw(Election::postgres_drop()).await;
}

#[derive(PostGresObj)]
struct ElectionEvent {
    id: i32,
    name: String,
}

#[derive(PostGresObj)]
struct Election {
    id: String,
    election_event_id: i32,
    name: String,
    category: String,
}
