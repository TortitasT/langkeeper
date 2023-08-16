use actix_http::Request;
use actix_service::Service;
use actix_web::{dev::ServiceResponse, http::StatusCode, test};

use crate::{generate_app, DbPool};

async fn run_migrations(_pool: &DbPool) {
    // let conn = _pool.get().unwrap();
    //
    // TODO: handle manually for now
}

async fn init_service(
) -> impl actix_service::Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
    let pool = crate::db::get_connection_pool(Option::from("test.db".to_owned()));

    run_migrations(&pool).await;
    test::init_service(generate_app(pool)).await
}

#[actix_web::test]
async fn test_init_service() {
    let app = init_service().await;

    let req = test::TestRequest::with_uri("/").to_request();
    let res = app.call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_get_all_users_when_empty() {
    let app = init_service().await;

    let req = test::TestRequest::with_uri("/users").to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::OK);

    let body = test::read_body(res).await;
    assert_eq!(body, "[]");
}
