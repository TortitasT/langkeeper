use crate::{
    jobs::weekly_report::send_weekly_report,
    logger::{log, LogLevel},
    utils::get_last_monday_date,
};

use diesel::prelude::*;

pub async fn send_report() -> std::io::Result<()> {
    let email = std::env::args()
        .nth(2)
        .expect("Please provide an email address");

    log(
        &format!("Sending manual report to {}", email),
        LogLevel::Info,
    );

    let pool = crate::db::get_connection_pool(None);
    let mut conn = pool.get().unwrap();

    let user = crate::schema::users::dsl::users
        .filter(crate::schema::users::dsl::email.eq(email))
        .first::<crate::models::User>(&mut conn)
        .expect("User not found");

    let last_monday = get_last_monday_date();

    send_weekly_report(&user, &last_monday, &mut conn).await;

    Ok(())
}
