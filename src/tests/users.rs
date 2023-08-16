use crate::tests::init_service;

use actix_web::{http::StatusCode, test};
use diesel::RunQueryDsl;

#[actix_web::test]
async fn test_get_all_users_when_empty() {
    let (app, _) = init_service().await;

    let req = test::TestRequest::with_uri("/users").to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::OK);

    let body = test::read_body(res).await;
    assert_eq!(body, "[]");
}

#[actix_web::test]
async fn test_get_all_users_when_one_user() {
    let (app, pool) = init_service().await;

    diesel::insert_into(crate::schema::users::table)
        .values(crate::resources::NewUser {
            name: "test".to_owned(),
            email: "test@test.test".to_owned(),
            password: "test".to_owned(),
        })
        .execute(&mut pool.get().unwrap())
        .unwrap();

    let req = test::TestRequest::with_uri("/users").to_request();

    let res: Vec<crate::resources::ShowUser> = test::call_and_read_body_json(&app, req).await;

    assert_eq!(res.len(), 1);
}
