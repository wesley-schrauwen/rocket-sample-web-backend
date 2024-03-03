use rocket::http::CookieJar;
use crate::models::errors::ErrorResponse;

#[post("/login")]
pub async fn login(cookies: &CookieJar<'_>) -> Result<(), ErrorResponse> {
    cookies.add_private(("user", "dead-beef-user"));
    Ok(())
}

#[post("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Result<(), ()> {
    cookies.remove("user");
    Ok(())
}