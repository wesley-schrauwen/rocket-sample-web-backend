#[macro_use] extern crate rocket;

use std::sync::{RwLock};
use std::collections::HashMap;
use rocket::response::status;
use rocket::response::status::NotFound;
use rocket::State;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;

#[derive(Deserialize, Serialize, Hash)]
struct Person {
    name: String,
    age: u8,
    last_name: String
}

impl Clone for Person {
    fn clone(&self) -> Self {
        Person {
            name: self.name.clone(),
            age: self.age,
            last_name: self.last_name.clone(),
        }
    }
}

#[derive(Serialize, Hash)]
struct Error {
    code: u16,
    message: String
}

trait Errors {
    fn not_found(message: String) -> Json<Error>;
}

trait PersonErrors {
    fn not_found(id: String) -> Json<Error>;
}

impl Errors for Error {
    fn not_found(message: String) -> Json<Error> {
        Json(Error {
            message,
            code: 404
        })
    }
}

impl PersonErrors for Error {
    fn not_found(id: String) -> Json<Error> {
        Json(
            Error {
                message: format!("person with id: {id} not found"),
                code: 404
            }
        )
    }
}

struct KeyValueStore {
    store: RwLock<HashMap<String, Person>>
}

impl KeyValueStore {
    fn new () -> Self {
        KeyValueStore {
            store: RwLock::new(HashMap::new()),
        }
    }

    fn insert(&self, person: Person) -> Option<Person> {
        self.store.write().unwrap().insert(person.name.to_string(), person)
    }

    fn get(&self, key: &str) -> Option<Person> {
        let store = self.store.read().unwrap();
        store.get(key).cloned()
    }

    fn delete(&self, key: &str) -> Option<Person> {
        self.store.write().unwrap().remove(key)
    }
}

#[put("/<name>", format="json", data="<person>")]
fn put_index(name: &str, person: Json<Person>, cache: &State<KeyValueStore>) -> status::Created<Json<Person>> {
    cache.insert(Person {
        name: name.to_string(),
        age: person.age,
        last_name: person.into_inner().last_name
    });

    status::Created::new(format!("localhost:8000/{name}")).tagged_body(Json(cache.get(name).unwrap()))
}

#[delete("/<name>")]
fn delete_index(name: &str, cache: &State<KeyValueStore>) -> Result<status::NoContent, NotFound<Json<Error>>> {
    if let Some(person) = cache.delete(name) {
        Ok(status::NoContent)
    } else {
        Err(NotFound(<Error as PersonErrors>::not_found(name.to_string())))
    }
}

#[get("/<name>")]
fn index(name: &str, cache: &State<KeyValueStore>) -> Result<Json<Person>, NotFound<Json<Error>>> {
    if let Some(person) = cache.get(name) {
        Ok(Json(person))
    } else {
        Err(NotFound(<Error as PersonErrors>::not_found(name.to_string())))
    }
}

#[post("/person", format="json", data="<person>")]
fn post_index(person: Json<Person>, cache: &State<KeyValueStore>) -> status::Created<Json<Person>> {
    let person = person.into_inner();
    cache.insert(Person {
        name: person.name.clone(),
        last_name: person.last_name.clone(),
        age: person.age
    });

    status::Created::new(format!("localhost:8000/{}", person.name)).tagged_body(Json(cache.get(person.name.as_str()).unwrap()))
}

#[launch]
async fn rocket() -> _ {
    let store = KeyValueStore::new();
    let database_url = "postgres://postgres:password@localhost:5432";

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await.expect("Failed to init postgres");

    rocket::build().manage(store).manage(pool).mount("/", routes![index, post_index, delete_index, put_index])
}