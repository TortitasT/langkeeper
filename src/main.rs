pub mod controllers;
pub mod middlewares;
pub mod models;
pub mod resources;
pub mod schema;

mod db;
mod jwt;

#[cfg(test)]
mod test;

use std::env;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
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
            Err(_) => panic!("Port must be a number"),
        },
        None => {
            println!("No port provided, using default port 8000");
            8000
        }
    };

    println!("Starting server...");
    println!("Address: http://{}:{}", address, port);

    let pool = db::get_connection_pool(None);

    HttpServer::new(move || {
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
    })
    .bind((address, port))?
    .run()
    .await
}
