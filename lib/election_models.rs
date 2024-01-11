use crate::PostGresObj;

#[derive(PostGresObj)]
struct A {
    id: i32,
}

#[derive(PostGresObj)]
struct B {
    id: i32,
    name: String
}