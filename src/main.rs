pub mod controllers;
pub mod logger;
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

use crate::logger::{log, LogLevel};
use crate::mailer::send_verification_email;

use actix_files::Files;
use actix_http::header;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;
use std::thread;
use std::time::Duration;
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
    match env::args().nth(1) {
        Some(arg) => match arg.as_str() {
            "migrate" => {
                log("Running migrations...", LogLevel::Info);
                Command::new("diesel")
                    .args(&["migrations", "run"])
                    .output()
                    .expect("failed to execute process");
                log("Migrations completed", LogLevel::Info);

                exit(0)
            }
            "seed" => {
                let pool = db::get_connection_pool(None);

                log("Seeding database...", LogLevel::Info);
                db::seed_database(&pool);
                log("Database seeded", LogLevel::Info);

                exit(0)
            }
            "key:generate" => {
                use rand::Rng;
                const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*%#@!~";
                const PASSWORD_LEN: usize = 30;
                let mut rng = rand::thread_rng();

                let password: String = (0..PASSWORD_LEN)
                    .map(|_| {
                        let idx = rng.gen_range(0..CHARSET.len());
                        CHARSET[idx] as char
                    })
                    .collect();

                let mut file = File::open(".env")?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                let mut new_contents = String::new();
                for line in contents.lines() {
                    if line.starts_with("JWT_SECRET=") {
                        new_contents.push_str(&format!("JWT_SECRET={}\n", password));
                    } else {
                        new_contents.push_str(&format!("{}\n", line));
                    }
                }

                let mut file = File::create(".env")?;
                file.write_all(new_contents.as_bytes())?;
                file.sync_all()?;

                log("JWT secret generated", LogLevel::Info);
                exit(0)
            }
            "email:verify_current" => {
                use crate::schema::users::dsl::*;
                use diesel::prelude::*;

                let pool = db::get_connection_pool(None);
                let mut conn = pool.get().unwrap();

                let unverified_users = schema::users::dsl::users
                    .filter(verified.eq(0))
                    .load::<models::User>(&mut conn)
                    .unwrap();

                for user in unverified_users {
                    send_verification_email(&user);

                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
            "serve" => {
                start_server().await?;
            }
            _ => {
                log("Invalid argument provided", LogLevel::Error);
                log("Usage: cargo run [migrate|seed|serve]", LogLevel::Error);

                exit(1)
            }
        },
        None => {
            log("No argument provided", LogLevel::Error);
            log("Usage: cargo run [migrate|seed|serve]", LogLevel::Error);

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
