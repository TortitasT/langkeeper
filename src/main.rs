mod db;
pub mod models;
pub mod schema;

use actix_web::{
    get, post,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use diesel::prelude::*;
use diesel::{RunQueryDsl, SqliteConnection};
use garde::Validate;

use self::models::NewUser;

type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

#[post("/users")]
async fn users_new(user: web::Json<NewUser>, db_pool: web::Data<DbPool>) -> impl Responder {
    use crate::schema::users;

    let mut conn = db_pool.get().unwrap();

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
async fn users_index(db_pool: web::Data<DbPool>) -> impl Responder {
    use self::schema::users::dsl::users;

    let mut conn = db_pool.get().unwrap();

    let results = users
        .limit(100)
        .load::<models::User>(&mut *conn)
        .expect("Error loading users");

    HttpResponse::Ok().json(results)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = db::get_connection_pool();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(users_new)
            .service(users_index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
