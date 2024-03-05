use rocket::http::CookieJar;
use rocket::Route;
use uuid::Uuid;
use crate::models::errors::ErrorResponse;

#[post("/login/<id>")]
async fn login(id: Uuid, cookies: &CookieJar<'_>) -> Result<(), ErrorResponse> {
    cookies.add_private(("user", id.to_string()));
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