#[macro_use]
extern crate election_derive;

pub mod database;
pub mod election_models;
#[cfg(feature = "eml-5")]
pub mod eml_5;
pub mod sql_db;

pub trait PostGresObj {
    fn postgres_create() -> String;
    fn postgres_drop() -> String;
}
