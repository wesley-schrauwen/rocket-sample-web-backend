use rocket::http::{ContentType, Status};
use rocket::{Request, Response};
use rocket::response::Responder;
use rocket::serde::json::{Json, to_string};
use rocket::serde::Serialize;

trait Errors {
    fn not_found(message: String) -> Json<ErrorResponse>;
}

#[derive(Serialize, Hash)]
pub struct ErrorResponse {
    pub code: Status,
    pub message: String
}

trait PersonErrors {
    fn not_found(id: String) -> Json<ErrorResponse>;
    fn internal_error() -> Json<ErrorResponse>;
}

impl Errors for ErrorResponse {
    fn not_found(message: String) -> Json<ErrorResponse> {
        Json(ErrorResponse {
            message,
            code: Status::NotFound
        })
    }
}

impl PersonErrors for ErrorResponse {
    fn not_found(id: String) -> Json<ErrorResponse> {
        Json(
            ErrorResponse {
                message: format!("person with id: {id} not found"),
                code: Status::NotFound
            }
        )
    }

    fn internal_error() -> Json<ErrorResponse> {
        Json(ErrorResponse {
            code: Status::InternalServerError,
            message: String::from("Internal server error")
        })
    }
}

impl<'r> Responder<'r, 'static> for ErrorResponse {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        let serialized = to_string(&self).unwrap();

        Ok(
            Response::build()
                .status(self.code)
                .header(ContentType::JSON)
                .sized_body(serialized.len(), std::io::Cursor::new(serialized))
                .finalize()
        )
    }
}