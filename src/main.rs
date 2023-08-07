mod controllers;
mod db;
pub mod models;
pub mod schema;

#[cfg(test)]
mod test;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    web::{self},
    App, HttpServer,
};
use diesel::SqliteConnection;

type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = db::get_connection_pool(None);

    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false) // TODO: Remove this in production so only HTTPS is allowed
                    .build(),
            )
            .app_data(web::Data::new(pool.clone()))
            .service(controllers::users::user_controller_create)
            .service(controllers::users::user_controller_list)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
