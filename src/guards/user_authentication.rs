use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::{Request, State};
use rocket::log::private::kv::Source;
use rocket::request::{FromRequest, Outcome};
use sqlx::PgPool;
use crate::models::errors::ErrorResponse;
use crate::models::user_auth::AuthUser;
use crate::models::user_records::UserRecord;
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
                Ok(user) => {
                    println!("{:?}", user);
                    Outcome::Success(AuthUser {})
                },
                _ => error
            },
            _ => error
        }
    }
}