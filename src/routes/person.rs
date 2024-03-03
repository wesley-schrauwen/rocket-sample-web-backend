use rocket::request::{FromRequest};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{Route, State};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::errors::ErrorResponse;
use crate::models::user_auth::*;
use crate::models::user_dtos::UserDTO;
use crate::models::user_records::UserRecord;
use crate::traits::database_model::DatabaseModel;

#[put("/person/<id>", format="json", data="<person>")]
async fn update_person(id: Uuid, person: Json<UserDTO>, pool: &State<PgPool>, _auth_user: AuthUser) -> Result<UserRecord, ErrorResponse> {
    UserRecord::update(&id, &person.into_inner(), pool).await
}

#[delete("/person/<id>")]
async fn delete_person_by_id(id: Uuid, pool: &State<PgPool>, _auth_user: AuthUser) -> Result<status::NoContent, ErrorResponse> {
    match UserRecord::delete_by_id(&id, &pool.inner()).await {
        Ok(_) => Ok(status::NoContent),
        Err(error) => Err(error)
    }
}

#[get("/person/<id>")]
async fn get_person_by_id(id: Uuid, pool: &State<PgPool>, _auth_user: AuthUser) -> Result<UserRecord, ErrorResponse> {
    UserRecord::get_by_id(&id, pool.inner()).await
}

#[post("/person", format="json", data="<person_data>")]
async fn create_person(pool: &State<PgPool>, person_data: Json<UserDTO>, _auth_user: AuthUser) -> Result<UserRecord, ErrorResponse> {
    let person = person_data.into_inner();
    println!("{:?}", person);
    UserRecord::insert(&person, &pool.inner()).await
}

pub fn person_routes() -> Vec<Route> {
    routes![update_person, delete_person_by_id, get_person_by_id, create_person]
}