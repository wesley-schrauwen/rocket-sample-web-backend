use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::Request;
use rocket::request::{FromRequest, Outcome};
use crate::models::errors::ErrorResponse;
use crate::models::users::AuthUser;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ErrorResponse;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(_) = request.cookies().get("user") {
            Success(AuthUser {})
        } else {
            Error((Status::Unauthorized, ErrorResponse {
                code: Status::Unauthorized,
                message: "Unauthorized".to_string()
            }))
        }
    }
}