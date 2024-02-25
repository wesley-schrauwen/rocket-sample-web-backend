mod models;

#[macro_use] extern crate rocket;

use std::str::FromStr;

use rocket::response::{Responder, status};
use rocket::{State};
use rocket::request::FromParam;

use rocket::serde::json::{Json};
use serde::{Deserialize, Serialize};

use sqlx::{FromRow, PgPool, Row};
use sqlx::postgres::{PgPoolOptions};
use uuid::Uuid;
use rocket::figment::providers::{Format};

use models::users::UserRecord;
use crate::models::errors::ErrorResponse;
use crate::models::users::{DatabaseModel, UserDTO};

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
    // let cache = KeyValueStore::new();
    let database_url = "postgres://postgres:password@localhost:5432";

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to init postgres");

    rocket::build()
        // .manage(cache)
        .manage(pool)
        .mount("/", routes![get_person_by_id, create_person, delete_person_by_id, update_person])
}