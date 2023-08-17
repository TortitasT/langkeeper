#![recursion_limit = "256"]

pub mod controllers;
pub mod middlewares;
pub mod models;
pub mod resources;
pub mod schema;
pub mod seeders;

mod db;
mod jwt;

#[cfg(test)]
mod tests;

use actix_files::Files;
use std::{env, process::exit};

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web::{self},
    App, HttpServer,
};
use diesel::SqliteConnection;

type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

#[actix_web::get("/checkhealth")]
async fn checkhealth() -> &'static str {
    "Hello from langkeeper!! 🦀"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match env::args().nth(1) {
        Some(arg) => match arg.as_str() {
            "migrate" => {
                println!("Running migrations...");
                println!("TODO: handle via `diesel migrations run` command for now");
                // db::run_migrations().await?;
                println!("Migrations completed");

                exit(0)
            }
            "seed" => {
                let pool = db::get_connection_pool(None);

                println!("Seeding database...");
                db::seed_database(&pool);
                println!("Database seeded");

                exit(0)
            }
            "serve" => {
                start_server().await?;
            }
            _ => {
                println!("Invalid argument provided");
                println!("Usage: cargo run [migrate|seed|serve]");

                exit(1)
            }
        },
        None => {
            println!("No argument provided");
            println!("Usage: cargo run [migrate|seed|serve]");

            exit(1)
        }
    }

    Ok(())
}

async fn start_server() -> std::io::Result<()> {
    let address = "0.0.0.0";
    let port = match env::args().nth(2) {
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
        .await?;

    println!("Server stopped");
    Ok(())
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
        .service(checkhealth)
        .configure(controllers::users::init)
        .configure(controllers::languages::init)
        .service(
            Files::new("/", "./www")
                .index_file("index.html")
                .show_files_listing(),
        )
}
