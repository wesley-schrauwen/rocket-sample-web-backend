use rocket::http::{Cookie, Status};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::errors::ErrorResponse;
use crate::models::user_dtos::UserDTO;
use crate::models::user_records::UserRecord;

pub trait DatabaseModel {
    async fn get_by_id(id: &Uuid, pool: &PgPool) -> Result<UserRecord, ErrorResponse>;
    async fn insert(user: &UserDTO, pool: &PgPool) -> Result<UserRecord, ErrorResponse>;
    async fn delete_by_id(id: &Uuid, pool: &PgPool) -> Result<(), ErrorResponse>;
    async fn update(id: &Uuid, user: &UserDTO, pool: &PgPool) -> Result<UserRecord, ErrorResponse>;
    async fn get_by_cookie(cookie: &Cookie, pool: &PgPool) -> Result<UserRecord, ErrorResponse>;
}