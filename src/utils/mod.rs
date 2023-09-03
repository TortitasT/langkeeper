use chrono::{Datelike, Duration, NaiveDateTime, Utc};

pub fn get_last_monday_date() -> NaiveDateTime {
    let now = Utc::now();
    let days_since_monday = now.weekday().num_days_from_monday() + 1;
    let last_monday = now - Duration::days(days_since_monday.into());

    last_monday.naive_utc()
}
