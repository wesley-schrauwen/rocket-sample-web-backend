#[macro_use] extern crate rocket;

use std::sync::{RwLock};
use std::collections::HashMap;
use std::fmt::format;
use rocket::response::status;
use rocket::State;

struct Person {
    name: String,
    age: String,
    last_name: String
}

impl Clone for Person {
    fn clone(&self) -> Self {
        Person {
            name: self.name.to_string(),
            age: self.age.to_string(),
            last_name: self.last_name.to_string(),
        }
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

    fn insert(&self, key: &str, person: Person) -> Option<Person> {
        self.store.write().unwrap().insert(key.into(), person)
    }

    fn get(&self, key: &str) -> Option<Person> {
        let store = self.store.read().unwrap();
        store.get(key).cloned()
    }

    fn delete(&self, key: &str) -> Option<Person> {
        self.store.write().unwrap().remove(key)
    }
}

#[put("/<name>/<age>/<last_name>")]
fn put_index(name: &str, age: i8, last_name: &str, cache: &State<KeyValueStore>) -> status::Created<String> {
    cache.insert(name, Person {
        name: name.to_string(),
        age: age.to_string(),
        last_name: last_name.to_string()
    });

    status::Created::new(format!("localhost:8000/{name}")).tagged_body("person".to_string())
}

#[delete("/<name>")]
fn delete_index(name: &str, cache: &State<KeyValueStore>) -> String {
    if let Some(person) = cache.delete(name) {
        format!("{} {} deleted", person.name, person.last_name)
    } else {
        "No entry".to_string()
    }
}

#[get("/<name>")]
fn index(name: &str, cache: &State<KeyValueStore>) -> String {
    if let Some(person) = cache.get(name) {
        format!("Hello there {} {} {}", person.name, person.age, person.last_name)
    } else {
        "no one found!".to_string()
    }
}

#[post("/<name>/<age>/<last_name>")]
fn post_index(name: &str, age: i8, last_name: &str, cache: &State<KeyValueStore>) -> status::Created<String> {
    cache.insert(name, Person {
        age: age.to_string(),
        name: name.to_string(),
        last_name: last_name.to_string()
    });

    status::Created::new(format!("localhost:8000/{name}")).tagged_body(String::from("written"))
}

#[launch]
fn rocket() -> _ {
    let store = KeyValueStore::new();
    rocket::build().manage(store).mount("/", routes![index, post_index, delete_index, put_index])
}