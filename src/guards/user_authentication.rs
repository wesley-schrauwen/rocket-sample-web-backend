use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::{Request, State};
use rocket::request::{FromRequest, Outcome};
use sqlx::PgPool;
use crate::models::errors::ErrorResponse;
use crate::models::user_auth::{AdminUser, AuthUser};
use crate::models::user_records::UserRecord;
use crate::models::user_roles::UserRoles;
use crate::traits::database_model::DatabaseModel;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ErrorResponse;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {

        let pool = request.guard::<&State<PgPool>>().await.unwrap();
        let cookie = request.cookies().get_private("user");
        let error = Error((Status::Unauthorized, ErrorResponse {
            code: Status::Unauthorized,
            message: "Unauthorized".to_string()
        }));

        match cookie {
            Some(cookie) => match UserRecord::get_by_cookie(&cookie, pool).await {
                Ok(user) => Outcome::Success(AuthUser {}),
                _ => error
            },
            _ => error
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = ErrorResponse;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {

        let pool = request.guard::<&State<PgPool>>().await.unwrap();
        let cookie = request.cookies().get_private("user");
        let error = Error((Status::Forbidden, ErrorResponse {
            code: Status::Forbidden,
            message: "Forbidden".to_string()
        }));

        match cookie {
            Some(cookie) => match UserRecord::get_by_cookie(&cookie, pool).await {
                Ok(user) => match user.role {
                    UserRoles::Admin => Outcome::Success(AdminUser {}),
                    _ => error
                },
                _ => error
            },
            _ => error
        }

    }
}