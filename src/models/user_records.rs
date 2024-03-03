use std::str::FromStr;
use rocket::http::{ContentType, Cookie, Header, Method, Status};
use rocket::{Config, Request, Response};
use rocket::figment::Figment;
use rocket::figment::providers::{Env, Format, Toml};
use rocket::response::Responder;
use rocket::serde::json::to_string;
use rocket::serde::Serialize;
use sqlx::{Error, FromRow, PgPool, Row};
use sqlx::postgres::PgRow;
use uuid::Uuid;
use crate::models::errors::ErrorResponse;
use crate::models::user_dtos::UserDTO;
use crate::models::user_roles::UserRoles;
use crate::traits::database_model::DatabaseModel;
use crate::utils::port::get_port;

#[derive(Serialize, Hash, Clone, Debug)]
pub struct UserRecord {
    pub name: String,
    pub age: i32, /*
        postgres actually doesnt support i8 which would be closer to a reasonable age but because
        its signed we need to deal with negatives anyway so may as well make this i32 and then do
        validations
    */
    pub last_name: String,
    pub id: Uuid,
    pub role: UserRoles
}

impl FromRow<'_, PgRow> for UserRecord {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        Ok(Self {
            name: row.get::<String, &str>("name"),
            age: row.get("age"),
            last_name: row.get::<String, &str>("last_name"),
            id: row.get::<Uuid, &str>("id"),
            role: row.get::<UserRoles, &str>("role")
        })
    }
}

impl DatabaseModel for UserRecord {
    async fn  get_by_id(id: &Uuid, pool: &PgPool) -> Result<UserRecord, ErrorResponse> {
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
        let query = "INSERT INTO USERS (name, age, last_name, role) VALUES ($1, $2, $3, $4) RETURNING *";

        match sqlx::query_as::<_, UserRecord>(query)
            .bind(&user.name)
            .bind(&user.age)
            .bind(&user.last_name)
            .bind(&user.role)
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
        let query = "UPDATE USERS SET name = $1, last_name = $2, age = $3, role = $4 WHERE id = $5 RETURNING *";

        match sqlx::query_as::<_, UserRecord>(query)
            .bind(&user.name)
            .bind(&user.last_name)
            .bind(&user.age)
            .bind(&user.role)
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

    async fn get_by_cookie(cookie: &Cookie<'_>, pool: &PgPool) -> Result<UserRecord, ErrorResponse> {
        if let Ok(uuid) = Uuid::from_str(cookie.value()) {
            UserRecord::get_by_id(&uuid, pool).await
        } else {
            Err(ErrorResponse {
                code: Status::BadRequest,
                message: "Invalid uuid".to_string(),
            })
        }
    }
}

impl<'r> Responder<'r, 'static> for UserRecord {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let serialized = to_string(&self).unwrap();

        let mut response = Response::build();

        response.header(ContentType::JSON);
        response.sized_body(serialized.len(), std::io::Cursor::new(serialized));

        let status = match request.method() {
            Method::Post | Method::Put | Method::Patch => {
                response.header(Header::new("location",format!("http://localhost:{}/person/{}", get_port(), &self.id)));
                Status::Created
            },
            _ => Status::Ok
        };

        Ok(response.status(status).finalize())
    }
}