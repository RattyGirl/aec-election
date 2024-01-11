#[macro_use]
extern crate election_derive;

mod election_models;

pub trait PostGresObj {
    fn postgres_create(&self) -> String;
    fn postgres_drop(&self) -> String;
}