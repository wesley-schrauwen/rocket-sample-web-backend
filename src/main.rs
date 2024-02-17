#[macro_use] extern crate rocket;

use std::sync::{RwLock};
use std::collections::HashMap;

use std::str::FromStr;

use rocket::response::status;
use rocket::response::status::{BadRequest, NotFound};
use rocket::State;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use sqlx::{Error, FromRow, PgPool, Row};
use sqlx::postgres::{PgPoolOptions, PgRow};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Hash)]
struct UserDTO {
    name: String,
    age: i32, /*
        postgres actually doesnt support i8 which would be closer to a reasonable age but because
        its signed we need to deal with negatives anyway so may as well make this i32 and then do
        validations
    */
    last_name: String
}

#[derive(Serialize, Hash, Clone)]
struct UserDBO {
    name: String,
    age: i32, /*
        postgres actually doesnt support i8 which would be closer to a reasonable age but because
        its signed we need to deal with negatives anyway so may as well make this i32 and then do
        validations
    */
    last_name: String,
    id: Uuid,
}

impl FromRow<'_, PgRow> for UserDBO {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        Ok(Self {
            name: row.get::<String, &str>("name"),
            age: row.get("age"),
            last_name: row.get::<String, &str>("last_name"),
            id: row.get::<Uuid, &str>("id"),
        })
    }
}

impl Clone for UserDTO {
    fn clone(&self) -> Self {
        UserDTO {
            name: self.name.clone(),
            age: self.age,
            last_name: self.last_name.clone(),
        }
    }
}

#[derive(Serialize, Hash)]
struct ErrorResponse {
    code: u16,
    message: String
}

trait DatabaseModel {
    // fn insert(user: User) -> Uuid;
    async fn get_by_id(id: &Uuid, pool: &PgPool) -> Result<UserDBO, ErrorResponse>;
}

impl DatabaseModel for UserDTO {
    async fn get_by_id(id: &Uuid, pool: &PgPool) -> Result<UserDBO, ErrorResponse> {
        match sqlx::query_as::<_, UserDBO>("select * from users where id = $1").bind(id).fetch_optional(pool).await {
            Ok(Some(user)) => Ok(user),
            Ok(_) => Err(ErrorResponse {
                code: 400,
                message: format!("User with id {id} not found")
            }),
            Err(error) => {
                Err(ErrorResponse {
                    code: 500,
                    message: error.to_string()
                })
            }
        }
    }
}

trait Errors {
    fn not_found(message: String) -> Json<ErrorResponse>;
}

trait PersonErrors {
    fn not_found(id: String) -> Json<ErrorResponse>;
    fn internal_error() -> Json<ErrorResponse>;
}

impl Errors for ErrorResponse {
    fn not_found(message: String) -> Json<ErrorResponse> {
        Json(ErrorResponse {
            message,
            code: 404
        })
    }
}

impl PersonErrors for ErrorResponse {
    fn not_found(id: String) -> Json<ErrorResponse> {
        Json(
            ErrorResponse {
                message: format!("person with id: {id} not found"),
                code: 404
            }
        )
    }

    fn internal_error() -> Json<ErrorResponse> {
        Json(ErrorResponse {
                code: 500,
                message: String::from("Internal server error")
            })
    }
}

struct KeyValueStore {
    store: RwLock<HashMap<String, UserDTO>>
}

impl KeyValueStore {
    fn new () -> Self {
        KeyValueStore {
            store: RwLock::new(HashMap::new()),
        }
    }

    fn insert(&self, person: UserDTO) -> Option<UserDTO> {
        let name = person.name.clone();
        self.store.write().unwrap().insert(name.clone(), person);

        self.get(name.as_str())
    }

    fn get(&self, key: &str) -> Option<UserDTO> {
        let store = self.store.read().unwrap();
        store.get(key).cloned()
    }

    fn delete(&self, key: &str) -> Option<UserDTO> {
        self.store.write().unwrap().remove(key)
    }
}

#[put("/<name>", format="json", data="<person>")]
fn put_index(name: &str, person: Json<UserDTO>, cache: &State<KeyValueStore>) -> status::Created<Json<UserDTO>> {
    let person = cache.insert(UserDTO {
        name: name.to_string(),
        age: person.age,
        last_name: person.into_inner().last_name,
    }).unwrap();

    status::Created::new(format!("localhost:8000/{name}")).tagged_body(Json(person.clone()))
}

#[delete("/<name>")]
fn delete_index(name: &str, cache: &State<KeyValueStore>) -> Result<status::NoContent, NotFound<Json<ErrorResponse>>> {
    if let Some(_person) = cache.delete(name) {
        Ok(status::NoContent)
    } else {
        Err(NotFound(<ErrorResponse as PersonErrors>::not_found(name.to_string())))
    }
}

#[get("/person/<id>")]
async fn index(id: &str, pool: &State<PgPool>) -> Result<Json<UserDBO>, NotFound<Json<ErrorResponse>>> {
    let uuid = Uuid::from_str(id).unwrap();
    if let Ok(user) = UserDTO::get_by_id(&uuid, pool.inner()).await {
        Ok(Json(user.clone()))
    } else {
        Err(NotFound(<ErrorResponse as PersonErrors>::not_found(id.to_string())))
    }
}

#[post("/person", format="json", data="<person_data>")]
async fn post_index(pool: &State<PgPool>, person_data: Json<UserDTO>) -> Result<(), BadRequest<Json<ErrorResponse>>> {
    let person = person_data.into_inner();

    let query = sqlx::query("INSERT INTO USERS (name, age, last_name) VALUES ($1, $2, $3) RETURNING id")
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
        Err(_) => Err(BadRequest(<ErrorResponse as PersonErrors>::internal_error()))
    }

}

#[launch]
async fn rocket() -> _ {
    let cache = KeyValueStore::new();
    let database_url = "postgres://postgres:password@localhost:5432";

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await.expect("Failed to init postgres");

    rocket::build().manage(cache).manage(pool).mount("/", routes![index, post_index, delete_index, put_index])
}