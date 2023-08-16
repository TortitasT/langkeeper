use std::{env, process::exit};

use diesel::{r2d2::ConnectionManager, SqliteConnection};
use dotenvy::dotenv;

pub fn get_connection_pool(
    path: Option<String>,
) -> r2d2::Pool<ConnectionManager<SqliteConnection>> {
    let manager = get_connection_manager(path);

    match r2d2::Pool::builder().max_size(1).build(manager) {
        Ok(pool) => pool,
        Err(e) => {
            println!("Failed to create pool: {}", e);
            exit(1);
        }
    }
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
    match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("DATABASE_URL not set, remember to create a `.env` file and run `diesel migration run`");
            exit(1);
        }
    }
}
