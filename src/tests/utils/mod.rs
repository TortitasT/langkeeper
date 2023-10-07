use chrono::{DateTime, Local};

use crate::utils::get_last_monday_date;

#[actix_web::test]
async fn test_get_last_monday_date() {
    let mock_date = DateTime::parse_from_rfc3339("2021-01-04T08:00:00+00:00")
        .unwrap()
        .with_timezone(&Local);
    let last_monday = get_last_monday_date(Some(mock_date));

    assert_eq!(last_monday, mock_date.naive_local());
}
