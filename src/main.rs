pub mod controllers;
pub mod logger;
pub mod mailer;
pub mod middlewares;
pub mod models;
pub mod resources;
pub mod schema;
pub mod seeders;

mod commands;
mod db;
mod jobs;
mod jwt;

#[cfg(test)]
mod tests;

use crate::logger::{log, LogLevel};

use actix_files::Files;
use actix_http::header;
use commands::parse_arguments;
use jobs::init_jobs;
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

type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

#[actix_web::get("/checkhealth")]
async fn checkhealth() -> &'static str {
    "Hello from langkeeper!! 🦀"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    parse_arguments().await.unwrap_or_else(|err| {
        log(
            &format!("Error parsing arguments: {}", err),
            LogLevel::Error,
        );
        exit(1)
    });

    Ok(())
}

async fn start_server() -> std::io::Result<()> {
    let address = "0.0.0.0";
    let port = match env::args().nth(2) {
        Some(port) => match port.parse::<u16>() {
            Ok(port) => port,
            Err(_) => {
                log(
                    "Invalid port provided, using default port 8000",
                    LogLevel::Warn,
                );
                8000
            }
        },
        None => {
            log(
                "Invalid port provided, using default port 8000",
                LogLevel::Warn,
            );
            8000
        }
    };

    log("Starting server...", LogLevel::Info);
    log(
        &format!("Address: https://{}:{}", address, port),
        LogLevel::Info,
    );

    init_jobs();

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

    log("Server stopped", LogLevel::Info);
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
