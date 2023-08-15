use actix_web::web::Json;
use actix_web::{post, web::Data, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

// use crate::models::*;
use crate::schema::*;
use crate::{middlewares::auth::AuthMiddleware, DbPool};
use diesel::prelude::*;

pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(language_controller_ping);
}

#[derive(Deserialize)]
pub struct PingRequest {
    pub extension: String,
}

#[derive(Serialize)]
pub struct PingResponse {
    pub user_id: i32,
    pub language_id: i32,
    pub language_name: String,
    pub language_extension: String,
    pub minutes: i32,
}

#[post("/languages/ping")]
pub async fn language_controller_ping(
    request: Json<PingRequest>,
    db_pool: Data<DbPool>,
    auth_middleware: AuthMiddleware,
) -> impl Responder {
    let mut conn = db_pool.get().unwrap();

    let user = match get_user_by_id(auth_middleware.user_id, &mut *conn) {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::BadRequest().body("User not found");
        }
    };
    let language = match get_language_by_extension(&request.extension, &mut *conn) {
        Ok(language) => language,
        Err(_) => {
            return HttpResponse::BadRequest().body("Language not found");
        }
    };
    let users_languages = match get_or_create_user_languages(&user, &language, &mut *conn) {
        Ok(users_languages) => users_languages,
        Err(_) => {
            return HttpResponse::BadRequest().body("User language not found");
        }
    };

    diesel::update(&users_languages)
        .set(users_languages::minutes.eq(users_languages.minutes + 1))
        .execute(&mut *conn)
        .unwrap();

    let response = PingResponse {
        user_id: user.id,
        language_id: language.id,
        language_name: language.name,
        language_extension: language.extension,
        minutes: users_languages.minutes + 1,
    };

    HttpResponse::Ok().json(response)
}

fn get_user_by_id(
    user_id: i32,
    conn: &mut diesel::SqliteConnection,
) -> Result<crate::models::User, diesel::result::Error> {
    users::dsl::users
        .find(user_id)
        .first::<crate::models::User>(conn)
}

fn get_language_by_extension(
    extension: &str,
    conn: &mut diesel::SqliteConnection,
) -> Result<crate::models::Language, diesel::result::Error> {
    languages::dsl::languages
        .filter(crate::schema::languages::extension.eq(extension))
        .first::<crate::models::Language>(conn)
}

fn get_or_create_user_languages(
    user: &crate::models::User,
    language: &crate::models::Language,
    conn: &mut diesel::SqliteConnection,
) -> Result<crate::models::UserLanguage, diesel::result::Error> {
    let users_languages = users_languages::dsl::users_languages
        .filter(users_languages::user_id.eq(user.id))
        .filter(users_languages::language_id.eq(language.id))
        .first::<crate::models::UserLanguage>(conn);

    match users_languages {
        Ok(users_languages) => Ok(users_languages),
        Err(_) => {
            diesel::insert_into(users_languages::dsl::users_languages)
                .values((
                    users_languages::user_id.eq(user.id),
                    users_languages::language_id.eq(language.id),
                    users_languages::minutes.eq(0),
                ))
                .execute(conn)
                .unwrap();

            users_languages::dsl::users_languages
                .find((user.id, language.id))
                .first::<crate::models::UserLanguage>(conn)
        }
    }
}