mod models;
mod routes;
mod guards;

#[macro_use] extern crate rocket;

use std::str::FromStr;

use rocket::response::{Responder};
use rocket::request::FromParam;

use sqlx::postgres::{PgPoolOptions};
use rocket::figment::providers::{Format};
use crate::routes::person::*;
use crate::routes::user_authentication::*;

#[launch]
async fn rocket() -> _ {
    let database_url = "postgres://postgres:password@localhost:5432";

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to init postgres");

    rocket::build()
        .manage(pool)
        .mount("/", routes![get_person_by_id, create_person, delete_person_by_id, update_person, login])
}