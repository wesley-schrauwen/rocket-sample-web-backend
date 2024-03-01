use rocket::request::{FromRequest};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{State};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::errors::ErrorResponse;
use crate::models::users::{AuthUser, DatabaseModel, UserDTO, UserRecord};

#[put("/person/<id>", format="json", data="<person>")]
pub async fn update_person(id: Uuid, person: Json<UserDTO>, pool: &State<PgPool>) -> Result<UserRecord, ErrorResponse> {
    UserRecord::update(&id, &person.into_inner(), pool).await
}

#[delete("/person/<id>")]
pub async fn delete_person_by_id(id: Uuid, pool: &State<PgPool>) -> Result<status::NoContent, ErrorResponse> {
    match UserRecord::delete_by_id(&id, &pool.inner()).await {
        Ok(_) => Ok(status::NoContent),
        Err(error) => Err(error)
    }
}

#[get("/person/<id>")]
pub async fn get_person_by_id(id: Uuid, pool: &State<PgPool>, _auth_user: AuthUser) -> Result<UserRecord, ErrorResponse> {
    UserRecord::get_by_id(&id, pool.inner()).await
}

#[post("/person", format="json", data="<person_data>")]
pub async fn create_person(pool: &State<PgPool>, person_data: Json<UserDTO>) -> Result<UserRecord, ErrorResponse> {
    let person = person_data.into_inner();
    println!("{:?}", person);
    UserRecord::insert(&person, &pool.inner()).await
}