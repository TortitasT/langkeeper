use std::env;

use diesel::{r2d2::ConnectionManager, SqliteConnection};
use dotenvy::dotenv;

pub fn get_connection_pool(
    path: Option<String>,
) -> r2d2::Pool<ConnectionManager<SqliteConnection>> {
    let manager = get_connection_manager(path);

    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to SQLite DB file")
}

fn get_connection_manager(path: Option<String>) -> ConnectionManager<SqliteConnection> {
    dotenv().ok();

    let database_url = match path {
        Some(path) => path,
        None => get_database_url(),
    };

    ConnectionManager::<SqliteConnection>::new(database_url)
}

fn get_database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}
