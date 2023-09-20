use chrono::{Datelike, Duration, NaiveDateTime, Utc};

pub fn get_last_monday_date() -> NaiveDateTime {
    let now = Utc::now();
    let days_since_monday = now.weekday().num_days_from_monday();
    let last_monday = now - Duration::days(days_since_monday.into());
    let last_monday = last_monday.date().and_hms(9, 0, 0);

    last_monday.naive_utc()
}
