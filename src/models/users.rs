use rocket::http::{ContentType, Header, Method, Status};
use rocket::{Config, Request, Response};
use rocket::figment::Figment;
use rocket::figment::providers::{Env, Format, Toml};
use rocket::response::Responder;
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::to_string;
use serde::{Deserializer, Serializer};
use sqlx::{Encode, Error, FromRow, PgPool, Postgres, Row, Type};
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
use sqlx::postgres::{PgRow};
use uuid::Uuid;
use crate::models::errors::ErrorResponse;

#[derive(Debug, Clone, Hash)]
pub enum UserRoles {
    Admin,
    User
}

impl Serialize for UserRoles {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            UserRoles::User => serializer.serialize_str("user"),
            UserRoles::Admin => serializer.serialize_str("admin"),
        }
    }
}

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

#[derive(Serialize, Deserialize, Hash, Debug)]
pub struct UserDTO {
    name: String,
    age: i32, /*
        postgres actually doesnt support i8 which would be closer to a reasonable age but because
        its signed we need to deal with negatives anyway so may as well make this i32 and then do
        validations
    */
    last_name: String,
    role: UserRoles
}

impl<'q> Encode<'q, Postgres> for UserRoles {
    fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'q>>::ArgumentBuffer) -> IsNull {

        let text: String = match self {
            UserRoles::Admin => "admin".to_string(),
            UserRoles::User => "user".to_string()
        };

        buf.extend(text.as_bytes().iter());

        IsNull::No
    }
}

impl Type<Postgres> for UserRoles {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <Postgres as sqlx::Database>::TypeInfo::with_name("varchar")
    }
}

#[derive(Debug)]
pub struct AuthUser {}

impl Clone for UserDTO {
    fn clone(&self) -> Self {
        UserDTO {
            name: self.name.clone(),
            age: self.age,
            last_name: self.last_name.clone(),
            role: self.role.clone()
        }
    }
}

impl FromRow<'_, PgRow> for UserRecord {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        Ok(Self {
            name: row.get::<String, &str>("name"),
            age: row.get("age"),
            last_name: row.get::<String, &str>("last_name"),
            id: row.get::<Uuid, &str>("id"),
            role: match row.get::<&str, &str>("role") {
                "admin" => UserRoles::Admin,
                "user" => UserRoles::User,
                _ => panic!()
            }
        })
    }
}

pub trait DatabaseModel {
    async fn get_by_id(id: &Uuid, pool: &PgPool) -> Result<UserRecord, ErrorResponse>;
    async fn insert(user: &UserDTO, pool: &PgPool) -> Result<UserRecord, ErrorResponse>;
    async fn delete_by_id(id: &Uuid, pool: &PgPool) -> Result<(), ErrorResponse>;
    async fn update(id: &Uuid, user: &UserDTO, pool: &PgPool) -> Result<UserRecord, ErrorResponse>;
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

impl<'d> Deserialize<'d> for UserRoles {
    fn deserialize<D>(deserializer: D) -> Result<UserRoles, D::Error> where D: Deserializer<'d> {
        let value = Deserialize::deserialize(deserializer).unwrap();
        match value {
            "admin" => Ok(UserRoles::Admin),
            "user" => Ok(UserRoles::User),
            _ => Err(serde::de::Error::custom("Error"))
        }
    }
}