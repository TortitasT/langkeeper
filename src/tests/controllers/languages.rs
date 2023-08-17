use crate::{
    resources::{
        languages::{LanguageStats, NewLanguage, PingRequest, PingResponse},
        users::{LoginUser, NewUser},
    },
    tests::init_service,
    DbPool,
};

use actix_http::Request;
use actix_web::{cookie::Cookie, dev::ServiceResponse, http::StatusCode, test};
use diesel::RunQueryDsl;

async fn get_session_cookie<'a>(
    app: impl actix_service::Service<Request, Response = ServiceResponse, Error = actix_web::Error>,
    pool: &DbPool,
) -> Cookie<'a> {
    diesel::insert_into(crate::schema::users::table)
        .values(NewUser {
            name: "test".to_owned(),
            email: "test@test.test".to_owned(),
            password: bcrypt::hash("secret".to_owned(), bcrypt::DEFAULT_COST).unwrap(),
        })
        .execute(&mut pool.get().unwrap())
        .unwrap();

    let req = test::TestRequest::post()
        .uri("/users/login")
        .set_json(LoginUser {
            email: "test@test.test".to_owned(),
            password: "secret".to_owned(),
        })
        .to_request();
    let res = test::call_service(&app, req).await;

    let session_id = res.headers().get("set-cookie").unwrap();
    let session_id_cookie = Cookie::parse_encoded(session_id.to_str().unwrap());

    session_id_cookie.unwrap().into_owned()
}

#[actix_web::test]
async fn test_ping_languages() {
    let (app, pool) = init_service().await;

    let session_cookie = get_session_cookie(&app, &pool).await;

    diesel::insert_into(crate::schema::languages::table)
        .values(NewLanguage {
            name: "TestLang".to_owned(),
            extension: "testlang".to_owned(),
        })
        .execute(&mut pool.get().unwrap())
        .unwrap();

    let res = test::TestRequest::post()
        .uri("/languages/ping")
        .cookie(session_cookie)
        .set_json(PingRequest {
            extension: "testlang".to_owned(),
        })
        .send_request(&app)
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let body: PingResponse = test::read_body_json(res).await;
    assert_eq!(body.language_name, "TestLang");
}

#[actix_web::test]
async fn test_get_stats() {
    let (app, pool) = init_service().await;

    let session_cookie = get_session_cookie(&app, &pool).await;

    diesel::insert_into(crate::schema::languages::table)
        .values(NewLanguage {
            name: "TestLang".to_owned(),
            extension: "testlang".to_owned(),
        })
        .execute(&mut pool.get().unwrap())
        .unwrap();

    test::TestRequest::post()
        .uri("/languages/ping")
        .cookie(session_cookie.clone())
        .set_json(PingRequest {
            extension: "testlang".to_owned(),
        })
        .send_request(&app)
        .await;

    let res = test::TestRequest::get()
        .uri("/languages/stats")
        .cookie(session_cookie)
        .send_request(&app)
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let body: Vec<LanguageStats> = test::read_body_json(res).await;

    assert_eq!(body.len(), 1);
    assert_eq!(body.get(0).unwrap().language_name, "TestLang");
    assert_eq!(body.get(0).unwrap().minutes, 0);
}
