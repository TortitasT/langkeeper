use actix_session::Session;
use actix_web::{
    get, post,
    web::{Data, Form, Json},
    HttpRequest, HttpResponse, Responder,
};

use garde::Validate;


use crate::{
    jwt::{decode_auth_jwt},
    mailer::{send_verification_email},
    middlewares::auth::AuthMiddleware,
    models::User,
    resources::users::{LoginUser, NewUser, ShowUser},
    schema::users::dsl::users,
};
use diesel::prelude::*;
use diesel::RunQueryDsl;

pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(user_controller_list);
    cfg.service(user_controller_login);
    cfg.service(user_controller_login_htmx);
    cfg.service(user_controller_create);
    cfg.service(user_controller_create_htmx);
    cfg.service(user_controller_show);
    cfg.service(user_controller_verify);
}

#[get("/users")]
pub async fn user_controller_list(db_pool: Data<crate::DbPool>) -> impl Responder {
    let mut conn = db_pool.get().unwrap();

    let results = match users
        .select((
            crate::schema::users::id,
            crate::schema::users::name,
            crate::schema::users::email,
            crate::schema::users::created_at,
            crate::schema::users::updated_at,
        )) // TODO: is there a way to avoid doing this? I just want the fields from the struct
        .limit(100)
        .load::<ShowUser>(&mut *conn)
    {
        Ok(found_users) => found_users,
        Err(_) => return HttpResponse::InternalServerError().body("Something went wrong"),
    };

    HttpResponse::Ok().json(results)
}

#[post("/users/login")]
pub async fn user_controller_login(
    user: Json<LoginUser>,
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

#[post("/htmx/users/login")]
pub async fn user_controller_login_htmx(
    user: Form<LoginUser>,
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

                HttpResponse::Ok()
                    .insert_header(("HX-Redirect", "/dashboard"))
                    .body("User logged in")
            } else {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Something went wrong"),
    }
}

#[post("/users")]
pub async fn user_controller_create(
    user: Json<NewUser>,
    db_pool: Data<crate::DbPool>,
) -> impl Responder {
    let mut conn = db_pool.get().unwrap();

    let mut new_user = NewUser {
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

#[post("/htmx/users")]
pub async fn user_controller_create_htmx(
    user: Form<NewUser>,
    db_pool: Data<crate::DbPool>,
) -> impl Responder {
    let mut conn = db_pool.get().unwrap();

    let mut new_user = NewUser {
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
        Ok(_) => {
            let user = users
                .filter(crate::schema::users::email.eq(&user.email))
                .first::<crate::models::User>(&mut *conn)
                .unwrap();

            send_verification_email(&user);

            HttpResponse::Created()
                .insert_header(("HX-Redirect", "/guide"))
                .body("User created")
        }
        Err(_) => HttpResponse::InternalServerError().body("Something went wrong"),
    }
}

#[get("/users/me")]
pub async fn user_controller_show(
    db_pool: Data<crate::DbPool>,
    auth_middleware: AuthMiddleware,
) -> impl actix_web::Responder {
    let mut conn = db_pool.get().unwrap();

    let user = users
        .select((
            crate::schema::users::id,
            crate::schema::users::name,
            crate::schema::users::email,
            crate::schema::users::created_at,
            crate::schema::users::updated_at,
        ))
        .find(auth_middleware.user_id)
        .first::<ShowUser>(&mut *conn);

    match user {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().body("Something went wrong"),
    }
}

#[get("/users/verify/{token}")]
pub async fn user_controller_verify(
    db_pool: Data<crate::DbPool>,
    req: HttpRequest,
) -> impl actix_web::Responder {
    let token = req.match_info().get("token").unwrap();
    let mut conn = db_pool.get().unwrap();

    let user_id = decode_auth_jwt(token);
    let user_id = match user_id {
        Ok(user_id) => user_id.sub,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid token"),
    };

    let user = users
        .filter(crate::schema::users::id.eq(user_id))
        .first::<User>(&mut *conn);

    if let Err(_) = user {
        return HttpResponse::Unauthorized().body("Invalid token");
    }

    let result = diesel::update(users.find(user_id))
        .set(crate::schema::users::verified.eq(1))
        .execute(&mut *conn);

    match result {
        Ok(_) => HttpResponse::Ok().body("User verified successfully"),
        Err(_) => HttpResponse::InternalServerError()
            .body("Something went wrong, please try again later :("),
    }
}
