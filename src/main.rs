pub mod controllers;
pub mod mailer;
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
use actix_http::header;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::{env, process::exit};

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::dev::Service;
use actix_web::{
    cookie::Key,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web::{self},
    App, HttpServer,
};
use diesel::SqliteConnection;

use crate::mailer::send_text_mail;

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
    println!("Address: https://{}:{}", address, port);

    // load TLS keys
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    let pool = db::get_connection_pool(None);

    HttpServer::new(move || generate_app(&pool))
        // .bind((address, port))?
        .bind_openssl((address, port), builder)?
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
                .cookie_secure(true)
                .build(),
        )
        .app_data(web::Data::new(pool.clone()))
        .service(checkhealth)
        .configure(controllers::users::init)
        .configure(controllers::languages::init)
        .wrap_fn(|req, srv| {
            let fut = srv.call(req);
            async {
                let mut res = fut.await?;
                res.headers_mut().insert(
                    header::CACHE_CONTROL,
                    header::HeaderValue::from_static("no-cache"),
                );
                Ok(res)
            }
        })
        .service(
            Files::new("/", "./www")
                .index_file("index.html")
                .show_files_listing(),
        )
}
