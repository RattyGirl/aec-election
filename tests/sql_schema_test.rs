#[macro_use]
extern crate election_derive;

use election::PostGresObj;

#[derive(PostGresObj)]
struct A {
    id: i32,
}
#[derive(PostGresObj)]
struct B {
    id: i32,
    name: String,
}

#[test]
fn create() {
    let a = A { id: 4 };
    let b = B {
        id: 12,
        name: "Stradsadasdasd".to_string(),
    };
    assert_eq!("CREATE TABLE A (id INTEGER);", a.postgres_create());
    assert_eq!(
        "CREATE TABLE B (id INTEGER, name VARCHAR);",
        b.postgres_create()
    );
}

#[test]
fn drop() {
    let a = A { id: 4 };
    let b = B {
        id: 12,
        name: "Stradsadasdasd".to_string(),
    };
    assert_eq!("DROP TABLE A;", a.postgres_drop());
    assert_eq!("DROP TABLE B;", b.postgres_drop());
}
