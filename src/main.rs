mod models;
mod routes;
mod guards;
mod traits;
mod utils;

#[macro_use] extern crate rocket;

use std::str::FromStr;

use rocket::response::{Responder};
use rocket::request::FromParam;

use sqlx::postgres::{PgPoolOptions};
use rocket::figment::providers::{Format};
use crate::routes::person::person_routes;
use crate::routes::user_authentication::authentication_routes;

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
        .mount("/", [person_routes(), authentication_routes()])
}