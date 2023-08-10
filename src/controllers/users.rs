use actix_session::Session;
use actix_web::{
    get, post,
    web::{Data, Json},
    HttpResponse, Responder,
};
use garde::Validate;

use crate::{jwt, schema::users::dsl::users};
use diesel::prelude::*;
use diesel::RunQueryDsl;

#[get("/users")]
pub async fn user_controller_list(db_pool: Data<crate::DbPool>) -> impl Responder {
    let mut conn = db_pool.get().unwrap();

    let results = users
        .select((
            crate::schema::users::id,
            crate::schema::users::name,
            crate::schema::users::email,
            crate::schema::users::created_at,
            crate::schema::users::updated_at,
        )) // TODO: is there a way to avoid doing this? I just want the fields from the struct
        .limit(100)
        .load::<crate::resources::ShowUser>(&mut *conn)
        .expect("Error loading users");

    HttpResponse::Ok().json(results)
}

#[post("/user/login")]
pub async fn user_controller_login(
    user: Json<crate::resources::LoginUser>,
    db_pool: Data<crate::DbPool>,
    session: Session,
) -> impl Responder {
    let mut conn = db_pool.get().unwrap();

    let result = users
        .filter(crate::schema::users::email.eq(&user.email))
        .first::<crate::models::User>(&mut *conn);

    let result_user = match result {
        Ok(user) => user,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid credentials"),
    };

    match bcrypt::verify(&user.password, &result_user.password) {
        Ok(valid) => {
            if valid {
                let jwt = crate::jwt::generate_auth_jwt(&result_user).unwrap();
                session.insert("token", jwt).unwrap();

                HttpResponse::Ok().body("User logged in")
            } else {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Something went wrong"),
    }
}

#[post("/user")]
pub async fn user_controller_create(
    user: Json<crate::resources::NewUser>,
    db_pool: Data<crate::DbPool>,
) -> impl Responder {
    let mut conn = db_pool.get().unwrap();

    let mut new_user = crate::resources::NewUser {
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

#[actix_web::get("/user")]
async fn user_controller_show(session: Session) -> impl actix_web::Responder {
    let token = match session.get::<String>("token") {
        Ok(token) => match token {
            Some(token) => token,
            None => return actix_web::HttpResponse::Ok().body("No token"),
        },
        Err(_) => return actix_web::HttpResponse::Ok().body("No token"),
    };

    let decoded = match jwt::decode_auth_jwt(&token) {
        Ok(decoded) => decoded,
        Err(_) => return actix_web::HttpResponse::Ok().body("Invalid token"),
    };

    actix_web::HttpResponse::Ok().body(decoded.sub.to_string())
}
