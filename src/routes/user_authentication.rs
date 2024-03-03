use rocket::http::CookieJar;
use rocket::Route;
use crate::models::errors::ErrorResponse;

#[post("/login")]
async fn login(cookies: &CookieJar<'_>) -> Result<(), ErrorResponse> {
    cookies.add_private(("user", "dead-beef-user"));
    Ok(())
}

#[post("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> Result<(), ()> {
    cookies.remove("user");
    Ok(())
}

pub fn authentication_routes() -> Vec<Route> {
    routes![login, logout]
}