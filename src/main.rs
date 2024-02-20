#[macro_use] extern crate rocket;

use std::sync::{RwLock};
use std::collections::HashMap;

use std::str::FromStr;

use rocket::response::{Responder, status};
use rocket::{Request, Response, State};
use rocket::http::{ContentType, Header, Method, Status};
use rocket::request::FromParam;

use rocket::serde::json::{Json, to_string};
use serde::{Deserialize, Serialize};

use sqlx::{Error, FromRow, PgPool, Row};
use sqlx::postgres::{PgPoolOptions, PgRow};
use uuid::Uuid;
use rocket::Config;
use rocket::figment::{Figment};
use rocket::figment::providers::{Env, Format, Toml};

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

#[derive(Serialize, Hash, Clone, Debug)]
struct UserRecord {
    name: String,
    age: i32, /*
        postgres actually doesnt support i8 which would be closer to a reasonable age but because
        its signed we need to deal with negatives anyway so may as well make this i32 and then do
        validations
    */
    last_name: String,
    id: Uuid,
}

impl FromRow<'_, PgRow> for UserRecord {
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
    code: Status,
    message: String
}

trait DatabaseModel {
    async fn get_by_id(id: &Uuid, pool: &PgPool) -> Result<UserRecord, ErrorResponse>;
    async fn insert(user: &UserDTO, pool: &PgPool) -> Result<UserRecord, ErrorResponse>;
    async fn delete_by_id(id: &Uuid, pool: &PgPool) -> Result<(), ErrorResponse>;
    async fn update(id: &Uuid, user: &UserDTO, pool: &PgPool) -> Result<UserRecord, ErrorResponse>;
}

impl DatabaseModel for UserRecord {
    async fn get_by_id(id: &Uuid, pool: &PgPool) -> Result<UserRecord, ErrorResponse> {
        let query = "select * from users where id = $1";

        match sqlx::query_as::<_, UserRecord>(query).bind(id).fetch_optional(pool).await {
            Ok(Some(user)) => Ok(user),
            Ok(_) => Err(ErrorResponse {
                code: Status::NotFound,
                message: format!("User with id {id} not found")
            }),
            Err(error) => {
                Err(ErrorResponse {
                    code: Status::InternalServerError,
                    message: error.to_string()
                })
            }
        }
    }

    async fn insert(user: &UserDTO, pool: &PgPool) -> Result<UserRecord, ErrorResponse> {
        let query = "INSERT INTO USERS (name, age, last_name) VALUES ($1, $2, $3) RETURNING *";

        match sqlx::query_as::<_, UserRecord>(query)
            .bind(&user.name)
            .bind(&user.age)
            .bind(&user.last_name)
            .fetch_one(pool)
            .await {
            Ok(user) => Ok(user),
            Err(error) => Err(
                ErrorResponse {
                    code: Status::InternalServerError,
                    message: error.to_string(),
                }
            )
        }
    }

    async fn delete_by_id(id: &Uuid, pool: &PgPool) -> Result<(), ErrorResponse> {
        let query = "DELETE FROM USERS where id = $1";

        match sqlx::query(query).bind(id).execute(pool).await {
            Ok(_) => Ok(()),
            Err(error) => Err(
                ErrorResponse {
                    code: Status::InternalServerError,
                    message: error.to_string()
                }
            ),
        }
    }

    async fn update(id: &Uuid, user: &UserDTO, pool: &PgPool) -> Result<UserRecord, ErrorResponse> {
        let query = "UPDATE USERS SET name = $1, last_name = $2, age = $3 WHERE id = $4 RETURNING *";

        match sqlx::query_as::<_, UserRecord>(query)
            .bind(&user.name)
            .bind(&user.last_name)
            .bind(&user.age)
            .bind(id)
            .fetch_one(pool).await {
                Ok(user) => Ok(user),
                Err(error) => Err(
                    ErrorResponse {
                        code: Status::InternalServerError,
                        message: error.to_string()
                    }
                )
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
            code: Status::NotFound
        })
    }
}

impl PersonErrors for ErrorResponse {
    fn not_found(id: String) -> Json<ErrorResponse> {
        Json(
            ErrorResponse {
                message: format!("person with id: {id} not found"),
                code: Status::NotFound
            }
        )
    }

    fn internal_error() -> Json<ErrorResponse> {
        Json(ErrorResponse {
                code: Status::InternalServerError,
                message: String::from("Internal server error")
            })
    }
}

impl<'r> Responder<'r, 'static> for UserRecord {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let serialized = to_string(&self).unwrap();

        let mut response = Response::build();

        response.header(ContentType::JSON);
        response.sized_body(serialized.len(), std::io::Cursor::new(serialized));

        let config = Figment::from(Config::default())
            .merge(Toml::file(Env::var_or("ROCKET_CONFIG", "Rocket.toml")).nested());

        let port = match config.find_value("port") {
            Ok(number) => number.to_i128().unwrap() as i32,
            _ => 0
        };

        let status = match request.method() {
            Method::Post | Method::Put | Method::Patch => {
                response.header(Header::new("location",format!("http://localhost:{}/person/{}", port, &self.id)));
                Status::Created
            },
            _ => Status::Ok
        };

        Ok(response.status(status).finalize())
    }
}

impl<'r> Responder<'r, 'static> for ErrorResponse {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let serialized = to_string(&self).unwrap();

        Ok(
            Response::build()
                .status(self.code)
                .header(ContentType::JSON)
                .sized_body(serialized.len(), std::io::Cursor::new(serialized))
                .finalize()
        )
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

#[put("/person/<id>", format="json", data="<person>")]
async fn update_person(id: Uuid, person: Json<UserDTO>, pool: &State<PgPool>) -> Result<UserRecord, ErrorResponse> {
    UserRecord::update(&id, &person.into_inner(), pool).await
}

#[delete("/person/<id>")]
async fn delete_person_by_id(id: Uuid, pool: &State<PgPool>) -> Result<status::NoContent, ErrorResponse> {
    match UserRecord::delete_by_id(&id, &pool.inner()).await {
        Ok(_) => Ok(status::NoContent),
        Err(error) => Err(error)
    }
}

#[get("/person/<id>")]
async fn get_person_by_id(id: Uuid, pool: &State<PgPool>) -> Result<UserRecord, ErrorResponse> {
    UserRecord::get_by_id(&id, pool.inner()).await
}

#[post("/person", format="json", data="<person_data>")]
async fn create_person(pool: &State<PgPool>, person_data: Json<UserDTO>) -> Result<UserRecord, ErrorResponse> {
    let person = person_data.into_inner();
    UserRecord::insert(&person, &pool.inner()).await
}

#[launch]
async fn rocket() -> _ {
    let cache = KeyValueStore::new();
    let database_url = "postgres://postgres:password@localhost:5432";

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to init postgres");

    rocket::build()
        .manage(cache)
        .manage(pool)
        .mount("/", routes![get_person_by_id, create_person, delete_person_by_id, update_person])
}