use actix_web::web::Json;
use actix_web::{get, post, web::Data, HttpResponse, Responder};
use chrono::TimeZone;
use maud::{html, Markup};

use crate::resources::languages::{LanguageStats, PingRequest, PingResponse};
use crate::schema::*;
use crate::{middlewares::auth::AuthMiddleware, DbPool};
use diesel::prelude::*;

pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(language_controller_ping);
    cfg.service(language_controller_stats);
    cfg.service(language_controller_stats_htmx);
}

#[post("/languages/ping")]
pub async fn language_controller_ping(
    request: Json<PingRequest>,
    db_pool: Data<DbPool>,
    auth_middleware: AuthMiddleware,
) -> impl Responder {
    let mut conn = db_pool.get().unwrap();

    let language = match get_language_by_extension(&request.extension, &mut *conn) {
        Ok(language) => language,
        Err(_) => {
            return HttpResponse::BadRequest().body("Language not found");
        }
    }; // TODO: can be done inside get_or_create_user_languages in one query
    let users_languages =
        get_or_create_user_languages(&auth_middleware.user_id, &language, &mut *conn);

    let last_update = chrono::Utc
        .from_local_datetime(&users_languages.updated_at)
        .unwrap();

    let minutes_since_last_update = chrono::Utc::now()
        .signed_duration_since(last_update)
        .num_minutes();

    match minutes_since_last_update {
        i64::MIN..=0 => {}
        1..=15 => {
            diesel::update(&users_languages)
                .set((
                    users_languages::minutes
                        .eq(users_languages.minutes + minutes_since_last_update as i32),
                    users_languages::updated_at.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(&mut *conn)
                .unwrap();
        }
        _ => {
            diesel::update(&users_languages)
                .set(users_languages::updated_at.eq(chrono::Utc::now().naive_utc()))
                .execute(&mut *conn)
                .unwrap();
        }
    }

    return HttpResponse::Ok().json(PingResponse {
        user_id: auth_middleware.user_id,
        language_id: language.id,
        language_name: language.name,
        language_extension: language.extension,
        minutes: users_languages.minutes,
        minutes_since_last_update: minutes_since_last_update as i32,
    });
}

#[get("/languages/stats")]
pub async fn language_controller_stats(
    db_pool: Data<DbPool>,
    auth_middleware: AuthMiddleware,
) -> impl Responder {
    let mut conn = db_pool.get().unwrap();

    let users_languages = users_languages::dsl::users_languages
        .filter(users_languages::user_id.eq(auth_middleware.user_id))
        .load::<crate::models::UserLanguage>(&mut *conn)
        .unwrap();

    let mut stats = Vec::new();

    for user_language in users_languages {
        let language = languages::dsl::languages
            .find(user_language.language_id)
            .first::<crate::models::Language>(&mut *conn)
            .unwrap();

        stats.push(LanguageStats {
            language_id: language.id,
            language_name: language.name,
            language_extension: language.extension,
            minutes: user_language.minutes,
        });
    }

    return HttpResponse::Ok().json(stats);
}

#[get("/htmx/languages/stats")]
pub async fn language_controller_stats_htmx(
    db_pool: Data<DbPool>,
    auth_middleware: AuthMiddleware,
) -> impl Responder {
    let mut conn = db_pool.get().unwrap();

    let users_languages = users_languages::dsl::users_languages
        .filter(users_languages::user_id.eq(auth_middleware.user_id))
        .load::<crate::models::UserLanguage>(&mut *conn)
        .unwrap();

    let mut stats = Vec::new();

    for user_language in users_languages {
        let language = languages::dsl::languages
            .find(user_language.language_id)
            .first::<crate::models::Language>(&mut *conn)
            .unwrap();

        stats.push(LanguageStats {
            language_id: language.id,
            language_name: language.name,
            language_extension: language.extension,
            minutes: user_language.minutes,
        });
    }

    let html = html!(
        tbody {
            @for stat in stats {
                tr {
                    td {
                        (stat.language_name)
                    }
                    td {
                        (stat.language_extension)
                    }
                    td {
                        (stat.minutes)
                    }
                }
            }
        }
    );

    return Markup::into_string(html);
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
    user_id: &i32,
    language: &crate::models::Language,
    conn: &mut diesel::SqliteConnection,
) -> crate::models::UserLanguage {
    let users_languages = users_languages::dsl::users_languages
        .filter(users_languages::user_id.eq(user_id))
        .filter(users_languages::language_id.eq(language.id))
        .first::<crate::models::UserLanguage>(conn);

    match users_languages {
        Ok(users_languages) => users_languages,
        Err(_) => {
            diesel::insert_into(users_languages::dsl::users_languages)
                .values((
                    users_languages::user_id.eq(user_id),
                    users_languages::language_id.eq(language.id),
                    users_languages::minutes.eq(0),
                ))
                .execute(conn)
                .unwrap();

            users_languages::dsl::users_languages
                .find((user_id, language.id))
                .first::<crate::models::UserLanguage>(conn)
                .unwrap()
        }
    }
}
