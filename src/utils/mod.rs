use chrono::{DateTime, Datelike, Duration, Local, NaiveDateTime};

pub fn get_last_monday_date(override_now: Option<DateTime<Local>>) -> NaiveDateTime {
    let now = match override_now {
        Some(now) => now,
        None => Local::now(),
    };

    let days_since_monday = now.weekday().num_days_from_monday();
    let last_monday = now - Duration::days(days_since_monday.into());
    let last_monday = last_monday.date_naive().and_hms_opt(9, 0, 0);

    match last_monday {
        Some(date) => date,
        None => panic!("Failed to get last monday date"),
    }
}
