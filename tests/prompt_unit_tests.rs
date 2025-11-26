use chrono::{Duration, Utc};
use github_secrets::prompt::format_date;

#[test]
fn test_format_date_days_hours_minutes_and_invalid() {
    // 3 days ago
    let three_days_ago = (Utc::now() - Duration::days(3)).to_rfc3339();
    let s = format_date(&three_days_ago);
    assert!(s.contains("days ago"));

    // 2 hours ago
    let two_hours_ago = (Utc::now() - Duration::hours(2)).to_rfc3339();
    let s2 = format_date(&two_hours_ago);
    assert!(s2.contains("hours ago") || s2.contains("hours"));

    // 5 minutes ago
    let five_minutes_ago = (Utc::now() - Duration::minutes(5)).to_rfc3339();
    let s3 = format_date(&five_minutes_ago);
    assert!(s3.contains("minutes ago") || s3.contains("minutes"));

    // invalid format should return original string
    let invalid = "not-a-date";
    let s4 = format_date(invalid);
    assert_eq!(s4, invalid.to_string());
}
