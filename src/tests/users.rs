use crate::tests::init_service;

use actix_http::header::Header;
use actix_web::{
    cookie::{Cookie, CookieBuilder},
    http::StatusCode,
    test,
};
use bcrypt::bcrypt;
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

#[actix_web::test]
async fn test_login() {
    let (app, pool) = init_service().await;

    // Create user
    diesel::insert_into(crate::schema::users::table)
        .values(crate::resources::NewUser {
            name: "test".to_owned(),
            email: "test@test.test".to_owned(),
            password: bcrypt::hash("secret".to_owned(), bcrypt::DEFAULT_COST).unwrap(),
        })
        .execute(&mut pool.get().unwrap())
        .unwrap();

    // Login
    let req = test::TestRequest::post()
        .uri("/users/login")
        .set_json(crate::resources::LoginUser {
            email: "test@test.test".to_owned(),
            password: "secret".to_owned(),
        })
        .to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::OK);

    let session_id = res.headers().get("set-cookie").unwrap();
    let session_id_cookie = Cookie::parse(session_id.to_str().unwrap()).unwrap();

    let res = test::TestRequest::get()
        .uri("/users/me")
        .cookie(session_id_cookie)
        .send_request(&app)
        .await;

    println!("\n{:?}", res.into_body());

    // assert_eq!(res.status(), StatusCode::OK);
}
