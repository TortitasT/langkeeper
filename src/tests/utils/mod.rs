use chrono::{DateTime, Local};

use crate::utils::get_last_monday_date;

#[actix_web::test]
async fn test_get_last_monday_date_on_monday_at_nine() {
    let mock_date = DateTime::parse_from_rfc3339("2021-01-04T09:00:00+01:00")
        .unwrap()
        .with_timezone(&Local);
    let last_monday = get_last_monday_date(Some(mock_date));

    assert_eq!(last_monday, mock_date.naive_local());
}

#[actix_web::test]
async fn test_get_last_monday_date_on_monday_at_nine_and_one() {
    let mock_date = DateTime::parse_from_rfc3339("2021-01-04T09:00:01+01:00")
        .unwrap()
        .with_timezone(&Local);
    let last_monday = get_last_monday_date(Some(mock_date));

    let expected_date = mock_date.date_naive().and_hms_opt(9, 0, 0).unwrap();

    assert_eq!(last_monday, expected_date);
}

#[actix_web::test]
async fn test_get_last_monday_date_on_monday_at_eight() {
    let mock_date = DateTime::parse_from_rfc3339("2021-01-04T08:00:00+01:00")
        .unwrap()
        .with_timezone(&Local);
    let last_monday = get_last_monday_date(Some(mock_date));

    let expected_date = DateTime::parse_from_rfc3339("2020-12-28T09:00:00+01:00")
        .unwrap()
        .with_timezone(&Local)
        .naive_local();

    assert_eq!(last_monday, expected_date);
}
