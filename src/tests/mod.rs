mod controllers;
mod utils;

use std::{fs::remove_file, process::exit};

use actix_http::Request;
use actix_service::Service;
use actix_web::{dev::ServiceResponse, http::StatusCode, test};
use diesel::RunQueryDsl;

use crate::{generate_app, logger::log, DbPool};

async fn run_migrations(_pool: &DbPool) {
    // let conn = _pool.get().unwrap();
    //
    // TODO: handle via command for now
    match std::process::Command::new("diesel")
        .args(&["migration", "run", "--database-url", "sqlite://test.sqlite"])
        .output()
    {
        Ok(_) => (),
        Err(e) => {
            log(
                format!("Error running migrations: {}", e).as_str(),
                crate::logger::LogLevel::Error,
            );
            exit(1);
        }
    }
}

async fn seed_database(pool: &DbPool) {
    crate::seeders::languages::seed(&pool);
}

async fn clear_database(pool: &DbPool) {
    diesel::delete(crate::schema::users::table)
        .execute(&mut pool.get().unwrap())
        .unwrap();
}

pub async fn init_service() -> (
    impl actix_service::Service<Request, Response = ServiceResponse, Error = actix_web::Error>,
    DbPool,
) {
    remove_file("test.sqlite").unwrap_or(());

    let pool = crate::db::get_connection_pool(Option::from("test.sqlite".to_owned()));

    run_migrations(&pool).await;
    clear_database(&pool).await;
    seed_database(&pool).await;

    let app = test::init_service(generate_app(&pool)).await;

    (app, pool)
}

#[actix_web::test]
async fn test_init_service() {
    let (app, _) = init_service().await;

    let req = test::TestRequest::with_uri("/").to_request();
    let res = app.call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
