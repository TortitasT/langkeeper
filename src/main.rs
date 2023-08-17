pub mod controllers;
pub mod middlewares;
pub mod models;
pub mod resources;
pub mod schema;

mod db;
mod jwt;

#[cfg(test)]
mod tests;

use std::env;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web::{self},
    App, HttpServer,
};
use diesel::SqliteConnection;

type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

#[actix_web::get("/")]
async fn index() -> &'static str {
    "Hello world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let address = "0.0.0.0";
    let port = match env::args().nth(1) {
        Some(port) => match port.parse::<u16>() {
            Ok(port) => port,
            Err(_) => {
                println!("Invalid port provided, using default port 8000");
                8000
            }
        },
        None => {
            println!("No port provided, using default port 8000");
            8000
        }
    };

    println!("Starting server...");
    println!("Address: http://{}:{}", address, port);

    let pool = db::get_connection_pool(None);

    HttpServer::new(move || generate_app(&pool))
        .bind((address, port))?
        .run()
        .await
}

pub fn generate_app(
    pool: &DbPool,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .wrap(
            SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                .cookie_secure(false) // TODO: Remove this in production so only HTTPS is allowed
                .build(),
        )
        .app_data(web::Data::new(pool.clone()))
        .service(index)
        .configure(controllers::users::init)
        .configure(controllers::languages::init)
}
