use rocket::http::CookieJar;
use crate::models::errors::ErrorResponse;

#[post("/login")]
pub async fn login(cookies: &CookieJar<'_>) -> Result<(), ErrorResponse> {
    cookies.add_private(("user", "dead-beef-user"));
    Ok(())
}