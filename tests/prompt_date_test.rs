use chrono::{Duration, Utc};
use github_secrets::prompt::format_date;

#[test]
fn test_format_date_days_ago() {
    let past_date = Utc::now() - Duration::days(5);
    let formatted = format_date(&past_date.to_rfc3339());
    assert!(formatted.contains("5 days ago"));
}

#[test]
fn test_format_date_hours_ago() {
    let past_date = Utc::now() - Duration::hours(3);
    let formatted = format_date(&past_date.to_rfc3339());
    assert!(formatted.contains("3 hours ago"));
}

#[test]
fn test_format_date_minutes_ago() {
    let past_date = Utc::now() - Duration::minutes(45);
    let formatted = format_date(&past_date.to_rfc3339());
    assert!(formatted.contains("45 minutes ago"));
}

#[test]
fn test_format_date_just_now() {
    let now = Utc::now();
    let formatted = format_date(&now.to_rfc3339());
    assert_eq!(formatted, "just now");
}

#[test]
fn test_format_date_very_recent() {
    // Less than a minute ago should be "just now"
    let past_date = Utc::now() - Duration::seconds(30);
    let formatted = format_date(&past_date.to_rfc3339());
    assert_eq!(formatted, "just now");
}

#[test]
fn test_format_date_invalid_format() {
    let invalid = "not a valid date";
    let formatted = format_date(invalid);
    assert_eq!(formatted, invalid);
}

#[test]
fn test_format_date_empty_string() {
    let formatted = format_date("");
    assert_eq!(formatted, "");
}

#[test]
fn test_format_date_multiple_days() {
    let past_date = Utc::now() - Duration::days(30);
    let formatted = format_date(&past_date.to_rfc3339());
    assert!(formatted.contains("30 days ago"));
}

#[test]
fn test_format_date_multiple_hours() {
    let past_date = Utc::now() - Duration::hours(23);
    let formatted = format_date(&past_date.to_rfc3339());
    assert!(formatted.contains("23 hours ago"));
}

#[test]
fn test_format_date_one_day() {
    let past_date = Utc::now() - Duration::days(1);
    let formatted = format_date(&past_date.to_rfc3339());
    assert!(formatted.contains("1 days ago"));
}

#[test]
fn test_format_date_one_hour() {
    let past_date = Utc::now() - Duration::hours(1);
    let formatted = format_date(&past_date.to_rfc3339());
    assert!(formatted.contains("1 hours ago"));
}

#[test]
fn test_format_date_one_minute() {
    let past_date = Utc::now() - Duration::minutes(1);
    let formatted = format_date(&past_date.to_rfc3339());
    assert!(formatted.contains("1 minutes ago"));
}
