use rocket::Config;
use rocket::figment::Figment;
use rocket::figment::providers::{Env, Format, Toml};

pub fn get_port() -> i32 {
    let config = Figment::from(Config::default())
        .merge(Toml::file(Env::var_or("ROCKET_CONFIG", "Rocket.toml")).nested());

    let port = match config.find_value("port") {
        Ok(number) => number.to_i128().unwrap() as i32,
        _ => 0
    };

    port
}