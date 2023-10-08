use chrono::{DateTime, Datelike, Duration, Local, NaiveDateTime, Timelike};

pub fn get_last_monday_date(override_now: Option<DateTime<Local>>) -> NaiveDateTime {
    let now = match override_now {
        Some(now) => now,
        None => Local::now(),
    };

    let mut days_since_monday = now.weekday().num_days_from_monday();

    if days_since_monday == 0 && now.hour() < 9 {
        days_since_monday = 7;
    }

    let last_monday = now - Duration::days(days_since_monday.into());
    let last_monday = last_monday.date_naive().and_hms_opt(9, 0, 0);

    match last_monday {
        Some(date) => date,
        None => panic!("Failed to get last monday date"),
    }
}
