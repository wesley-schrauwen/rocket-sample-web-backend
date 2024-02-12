#[macro_use] extern crate rocket;

use std::sync::{RwLock};
use std::collections::HashMap;
use std::fmt::{Debug};
use rocket::futures::AsyncWriteExt;
use rocket::response::status;
use rocket::response::status::{BadRequest, Custom, NotFound};
use rocket::State;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use sqlx::postgres::{PgPoolOptions};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Hash)]
struct User {
    name: String,
    age: i32, /*
        postgres actually doesnt support i8 which would be closer to a reasonable age but because
        its signed we need to deal with negatives anyway so may as well make this i32 and then do
        validations
    */
    last_name: String
}

impl Clone for User {
    fn clone(&self) -> Self {
        User {
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
    fn internal_error() -> Json<Error>;
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

    fn internal_error() -> Json<Error> {
        Json(Error {
                code: 500,
                message: String::from("Internal server error")
            })
    }
}

struct KeyValueStore {
    store: RwLock<HashMap<String, User>>
}

impl KeyValueStore {
    fn new () -> Self {
        KeyValueStore {
            store: RwLock::new(HashMap::new()),
        }
    }

    fn insert(&self, person: User) -> Option<User> {
        let name = person.name.clone();
        self.store.write().unwrap().insert(name.clone(), person);

        self.get(name.as_str())
    }

    fn get(&self, key: &str) -> Option<User> {
        let store = self.store.read().unwrap();
        store.get(key).cloned()
    }

    fn delete(&self, key: &str) -> Option<User> {
        self.store.write().unwrap().remove(key)
    }
}

#[put("/<name>", format="json", data="<person>")]
fn put_index(name: &str, person: Json<User>, cache: &State<KeyValueStore>) -> status::Created<Json<User>> {
    let person = cache.insert(User {
        name: name.to_string(),
        age: person.age,
        last_name: person.into_inner().last_name
    }).unwrap();

    status::Created::new(format!("localhost:8000/{name}")).tagged_body(Json(person.clone()))
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
fn index(name: &str, cache: &State<KeyValueStore>) -> Result<Json<User>, NotFound<Json<Error>>> {
    if let Some(person) = cache.get(name) {
        Ok(Json(person.clone()))
    } else {
        Err(NotFound(<Error as PersonErrors>::not_found(name.to_string())))
    }
}

#[post("/person", format="json", data="<person_data>")]
async fn post_index(pool: &State<PgPool>, person_data: Json<User>) -> Result<(), BadRequest<Json<Error>>> {
    let person = person_data.into_inner();

    let query = sqlx::query("INSERT INTO USERS (name, age, last_name) VALUES ($1,$2, $3) RETURNING id")
        .bind(person.name.clone())
        .bind(person.age)
        .bind(person.last_name.clone())
        .fetch_one(pool.inner())
        .await;

    match query {
        Ok(record) => {
            let id: Uuid = record.get("id");
            println!("{:?}", id);
            Ok(())
        },
        Err(_) => Err(BadRequest(<Error as PersonErrors>::internal_error()))
    }

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