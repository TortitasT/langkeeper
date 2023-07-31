mod db;
pub mod models;
pub mod schema;

use std::sync::Mutex;

use actix_web::{
    get, post,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use diesel::{RunQueryDsl, SqliteConnection};
use garde::Validate;

use self::models::NewUser;

struct AppState {
    db: Mutex<SqliteConnection>,
}

#[post("/users")]
async fn users_new(user: web::Json<NewUser>, app_state: web::Data<AppState>) -> impl Responder {
    use crate::schema::users;

    let mut conn = app_state.db.lock().unwrap();

    let new_user = models::NewUser {
        name: user.name.clone(),
        email: user.email.clone(),
        password: user.password.clone(),
    };

    if let Err(error) = new_user.validate(&()) {
        return HttpResponse::BadRequest().body(error.to_string());
    }

    let result = diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut *conn);

    match result {
        Ok(_) => HttpResponse::Ok().body("User created"),
        Err(_) => HttpResponse::InternalServerError().body("Something went wrong"),
    }
}

#[get("/users")]
async fn users_index(app_state: web::Data<AppState>) -> impl Responder {
    use self::schema::users::dsl::users;
    use diesel::prelude::*;

    let mut conn = app_state.db.lock().unwrap();

    let results = users
        .limit(5)
        .load::<models::User>(&mut *conn)
        .expect("Error loading users");

    HttpResponse::Ok().json(results)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = db::establish_connection();

    let app_state = web::Data::new(AppState { db: Mutex::new(db) });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(users_new)
            .service(users_index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
