use diesel::r2d2::ConnectionManager;
use maud::html;
use r2d2::PooledConnection;

use crate::mailer::send_mail;
use crate::models::User;
use crate::resources::languages::LanguageStats;
use crate::schema::users::dsl::*;
use crate::schema::{languages, users_languages_weekly};
use crate::utils::get_last_monday_date;
use crate::{
    db,
    logger::{log, LogLevel},
};
use actix_jobs::Job;
use diesel::prelude::*;

pub struct WeeklyReportJob {}

impl Job for WeeklyReportJob {
    fn cron(&self) -> &str {
        "0 0 9 * * 2 *"
    }

    fn run(&mut self) {
        actix_rt::spawn(async move {
            log("Starting weekly report", LogLevel::Info);

            let last_monday = get_last_monday_date(None);

            let mut conn = db::get_connection_pool(None).get().unwrap();

            let users_result = users
                .filter(crate::schema::users::verified.eq(1))
                .select(crate::schema::users::all_columns)
                .load::<User>(&mut conn)
                .unwrap();

            for user in users_result {
                send_weekly_report(&user, &last_monday, &mut conn).await;
            }
        });
    }
}

pub async fn send_weekly_report(
    user: &User,
    last_monday: &chrono::NaiveDateTime,
    conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
) {
    let stats = match get_weekly_stats(user, &last_monday, conn) {
        Some(stats) => stats,
        None => {
            log(&format!("No stats for user {}", user.name), LogLevel::Info);
            return;
        }
    };

    let html = html! {
        h1 {
            "Hello, " (user.name) "!"
        }

        p {
            "Here is your weekly language report:"
        }

        table style="border-spacing: 15px;" {
            tr {
                th {
                    "Language"
                }
                th {
                    "Time spent"
                }
            }
            @for stat in stats {
                tr {
                    td {
                        (stat.language_name) " (." (stat.language_extension) ")"
                    }
                    td {
                        (stat.hours) "h " (stat.minutes) "m " (stat.seconds) "s"
                    }
                }
            }
        }
    };

    send_mail(
        &user.email,
        "Your weekly Langkeeper report",
        &html.into_string(),
        true,
    )
    .await;
}

pub fn get_weekly_stats(
    user: &User,
    last_monday: &chrono::NaiveDateTime,
    conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
) -> Option<Vec<LanguageStats>> {
    let reports = users_languages_weekly::table
        .filter(users_languages_weekly::user_id.eq(user.id))
        .filter(users_languages_weekly::created_at.gt(last_monday).and(
            users_languages_weekly::created_at.lt(last_monday.clone() + chrono::Duration::days(7)),
        ))
        .order(users_languages_weekly::seconds.desc())
        .load::<crate::models::UserLanguageWeekly>(conn)
        .unwrap();

    if reports.len() == 0 {
        return None;
    }

    let mut stats = Vec::new();

    for report in reports {
        let language = languages::dsl::languages
            .find(report.language_id)
            .first::<crate::models::Language>(conn)
            .unwrap();

        let duration = chrono::Duration::seconds(report.seconds);

        stats.push(LanguageStats {
            language_id: language.id,
            language_name: language.name,
            language_extension: language.extension,
            hours: duration.num_hours(),
            minutes: duration.num_minutes() % 60,
            seconds: duration.num_seconds() % 60,
        });
    }

    Some(stats)
}
