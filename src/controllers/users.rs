use actix_session::Session;
use actix_web::{
    get, post,
    web::{Data, Json},
    HttpResponse, Responder,
};
use garde::Validate;

use crate::schema::users::dsl::users;
use diesel::prelude::*;
use diesel::RunQueryDsl;

#[get("/users")]
pub async fn user_controller_list(db_pool: Data<crate::DbPool>) -> impl Responder {
    let mut conn = db_pool.get().unwrap();

    let results = users
        .limit(100)
        .load::<crate::models::User>(&mut *conn)
        .expect("Error loading users");

    HttpResponse::Ok().json(results)
}

#[post("/users")]
pub async fn user_controller_create(
    user: Json<crate::models::NewUser>,
    db_pool: Data<crate::DbPool>,
) -> impl Responder {
    let mut conn = db_pool.get().unwrap();

    let mut new_user = crate::models::NewUser {
        name: user.name.clone(),
        email: user.email.clone(),
        password: user.password.clone(),
    };

    if let Err(error) = new_user.validate(&()) {
        return HttpResponse::BadRequest().body(error.to_string());
    }

    new_user.password = match bcrypt::hash(&user.password, bcrypt::DEFAULT_COST) {
        Ok(password) => password.to_string(),
        Err(_) => return HttpResponse::InternalServerError().body("Something went wrong"),
    };

    let result = diesel::insert_into(users)
        .values(&new_user)
        .execute(&mut *conn);

    match result {
        Ok(_) => HttpResponse::Created().body("User created"),
        Err(_) => HttpResponse::InternalServerError().body("Something went wrong"),
    }
}

#[post("/users/login")]
pub async fn user_controller_login(
    user: Json<crate::models::LoginUser>,
    db_pool: Data<crate::DbPool>,
    session: Session,
) -> impl Responder {
    let mut conn = db_pool.get().unwrap();

    let result = users
        .filter(crate::schema::users::email.eq(&user.email))
        .first::<crate::models::User>(&mut *conn);

    match result {
        Ok(user) => {
            let valid = match bcrypt::verify(&user.password, &user.password) {
                Ok(valid) => valid,
                Err(_) => return HttpResponse::InternalServerError().body("Something went wrong"),
            };

            if valid {
                // session.insert("langmer_token", jwt);
                HttpResponse::Ok().body("User logged in")
            } else {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        Err(_) => HttpResponse::Unauthorized().body("Invalid credentials"),
    }
}
