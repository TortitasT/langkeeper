#[cfg(test)]
mod tests {
    use crate::{
        controllers::users::{user_controller_create, user_controller_list},
        db,
        models::{NewUser, User},
    };

    use actix_session::{storage::CookieSessionStore, SessionMiddleware};
    use actix_web::{cookie::Key, http, test, web::Data, App};
    use diesel::prelude::*;
    use diesel::Connection;
    use r2d2::Pool;

    #[actix_web::test]
    async fn test_create_users() {
        let pool = db::get_connection_pool(Some("test.db".to_string()));
        let mut conn = pool.get().unwrap();
        run_migrations(&mut conn);

        // let mut conn = pool.get().unwrap();
        // conn.begin_test_transaction().unwrap();
        // diesel::delete(crate::schema::users::table)
        //     .execute(&mut conn)
        //     .unwrap();

        let mut app = test::init_service(
            App::new()
                .wrap(
                    SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                        .cookie_secure(false) // TODO: Remove this in production so only HTTPS is allowed
                        .build(),
                )
                .app_data(Data::new(pool.clone()))
                .service(user_controller_list)
                .service(user_controller_create),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/users")
            .set_json(&NewUser {
                name: "Johnhh".to_string(),
                email: "john@doe.es".to_string(),
                password: "12345678ABC".to_string(),
            })
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED);

        // let req = test::TestRequest::get().uri("/users").to_request();
        // let resp = test::call_service(&mut app, req).await;
        // assert_eq!(resp.status(), http::StatusCode::OK);
        //
        // let bytes = test::read_body(resp).await;
        // let body = std::str::from_utf8(&bytes).unwrap();
        // let users: Vec<User> = serde_json::from_str(body).unwrap();
        // assert_eq!(users.len(), 1);
    }

    fn run_migrations(pool: &Pool<SqliteConnection>) {
        let mut conn = pool.get().unwrap();
        embedded_migrations::run(&mut conn).unwrap();
    }
}
