use std::env;

use diesel::{r2d2::ConnectionManager, SqliteConnection};
use dotenvy::dotenv;

pub fn get_connection_pool() -> r2d2::Pool<ConnectionManager<SqliteConnection>> {
    let manager = get_connection_manager();

    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to SQLite DB file")
}

fn get_connection_manager() -> ConnectionManager<SqliteConnection> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    ConnectionManager::<SqliteConnection>::new(database_url)
}
