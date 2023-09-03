use actix_web::web::Json;
use actix_web::{get, post, web::Data, HttpResponse, Responder};
use chrono::{Datelike, TimeZone};
use maud::{html, Markup};

use crate::resources::languages::{LanguageStats, PingRequest, PingResponse};
use crate::schema::*;
use crate::utils::get_last_monday_date;
use crate::{middlewares::auth::AuthMiddleware, DbPool};
use diesel::prelude::*;

pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(language_controller_ping);
    cfg.service(language_controller_stats);
    cfg.service(language_controller_stats_htmx);
    cfg.service(language_controller_total_time_htmx);
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
            if request.extension == "" {
                return HttpResponse::NoContent().body("Language not found");
            }

            create_language(&request.extension, &mut *conn)
        }
    }; // TODO: can be done inside get_or_create_user_languages in one query
    let users_languages =
        get_or_create_user_languages(&auth_middleware.user_id, &language, &mut *conn);

    let users_languages_weekly =
        get_or_create_user_languages_weekly(&auth_middleware.user_id, &language, &mut *conn);

    let last_update = chrono::Utc
        .from_local_datetime(&users_languages.updated_at)
        .unwrap();

    let seconds_since_last_update = chrono::Utc::now()
        .signed_duration_since(last_update)
        .num_seconds();

    match seconds_since_last_update {
        i64::MIN..=1 => {}
        2..=900 => {
            // 15 minutes
            diesel::update(&users_languages)
                .set((
                    users_languages::seconds
                        .eq(users_languages.seconds + seconds_since_last_update),
                    users_languages::updated_at.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(&mut *conn)
                .unwrap();

            diesel::update(&users_languages_weekly)
                .set(
                    users_languages_weekly::seconds
                        .eq(users_languages_weekly.seconds + seconds_since_last_update),
                )
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

    let duration = chrono::Duration::seconds(users_languages.seconds);

    return HttpResponse::Ok().json(PingResponse {
        user_id: auth_middleware.user_id,
        language_id: language.id,
        language_name: language.name,
        language_extension: language.extension,
        hours: duration.num_hours(),
        minutes: duration.num_minutes() % 60,
        seconds: duration.num_seconds() % 60,
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

        let duration = chrono::Duration::seconds(user_language.seconds);

        stats.push(LanguageStats {
            language_id: language.id,
            language_name: language.name,
            language_extension: language.extension,
            hours: duration.num_hours(),
            minutes: duration.num_minutes() % 60,
            seconds: duration.num_seconds() % 60,
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
        .order(users_languages::seconds.desc())
        .load::<crate::models::UserLanguage>(&mut *conn)
        .unwrap();

    let mut stats = Vec::new();

    for user_language in users_languages {
        let language = languages::dsl::languages
            .find(user_language.language_id)
            .first::<crate::models::Language>(&mut *conn)
            .unwrap();

        let duration = chrono::Duration::seconds(user_language.seconds);

        stats.push(LanguageStats {
            language_id: language.id,
            language_name: language.name,
            language_extension: language.extension,
            hours: duration.num_hours(),
            minutes: duration.num_minutes() % 60,
            seconds: duration.num_seconds() % 60,
        });
    }

    let html = html!(
        ul {
            @for stat in stats {
                li class="page--dashboard__widget--stats__stat" {
                    div class="page--dashboard__widget--stats__stat__name" {
                        (stat.language_name)
                    }
                    div class="page--dashboard__widget--stats__stat__extension" {
                        "." (stat.language_extension)
                    }
                    div class="page--dashboard__widget--stats__stat__time" {
                        (format!("{:02}", stat.hours))
                        ":"
                        (format!("{:02}", stat.minutes))
                        ":"
                        (format!("{:02}", stat.seconds))
                    }
                }
            }
        }
    );

    return Markup::into_string(html);
}

#[get("/htmx/languages/total-time")]
pub async fn language_controller_total_time_htmx(
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

        let duration = chrono::Duration::seconds(user_language.seconds);

        stats.push(LanguageStats {
            language_id: language.id,
            language_name: language.name,
            language_extension: language.extension,
            hours: duration.num_hours(),
            minutes: duration.num_minutes() % 60,
            seconds: duration.num_seconds() % 60,
        });
    }

    let total_hours = stats.iter().fold(0, |acc, stat| acc + stat.hours);

    HttpResponse::Ok().body(format!(
        "{} {}",
        total_hours,
        match total_hours {
            1 => "hour",
            _ => "hours",
        }
    ))
}

fn get_language_by_extension(
    extension: &str,
    conn: &mut diesel::SqliteConnection,
) -> Result<crate::models::Language, diesel::result::Error> {
    languages::dsl::languages
        .filter(crate::schema::languages::extension.eq(extension))
        .first::<crate::models::Language>(conn)
}

fn create_language(
    extension: &str,
    conn: &mut diesel::SqliteConnection,
) -> crate::models::Language {
    diesel::insert_into(languages::dsl::languages)
        .values((
            languages::name.eq(extension),
            languages::extension.eq(extension),
        ))
        .execute(conn)
        .unwrap();

    languages::dsl::languages
        .filter(languages::extension.eq(extension))
        .first::<crate::models::Language>(conn)
        .unwrap()
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
                    users_languages::seconds.eq(0),
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

fn get_or_create_user_languages_weekly(
    user_id: &i32,
    language: &crate::models::Language,
    conn: &mut diesel::SqliteConnection,
) -> crate::models::UserLanguageWeekly {
    let last_monday = get_last_monday_date();

    let users_languages_weekly =
        users_languages_weekly::dsl::users_languages_weekly
            .filter(users_languages_weekly::user_id.eq(user_id))
            .filter(users_languages_weekly::language_id.eq(language.id))
            .filter(users_languages_weekly::created_at.gt(last_monday).and(
                users_languages_weekly::created_at.lt(last_monday + chrono::Duration::days(7)),
            ))
            .first::<crate::models::UserLanguageWeekly>(conn);

    match users_languages_weekly {
        Ok(users_languages_weekly) => users_languages_weekly,
        Err(_) => {
            diesel::insert_into(users_languages_weekly::dsl::users_languages_weekly)
                .values((
                    users_languages_weekly::user_id.eq(user_id),
                    users_languages_weekly::language_id.eq(language.id),
                    users_languages_weekly::seconds.eq(0),
                ))
                .execute(conn)
                .unwrap();

            users_languages_weekly::dsl::users_languages_weekly
                .select(users_languages_weekly::all_columns)
                .filter(users_languages_weekly::user_id.eq(user_id))
                .filter(users_languages_weekly::language_id.eq(language.id))
                .filter(users_languages_weekly::created_at.gt(last_monday).and(
                    users_languages_weekly::created_at.lt(last_monday + chrono::Duration::days(7)),
                ))
                .first::<crate::models::UserLanguageWeekly>(conn)
                .unwrap()
        }
    }
}
