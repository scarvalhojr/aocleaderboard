use chrono::{Datelike, FixedOffset, TimeZone, Utc};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};

const FIRST_EVENT_YEAR: i32 = 2015;
const EVENT_START_MONTH: u32 = 12;
const RELEASE_TIMEZONE_OFFSET: i32 = -5 * 3600;
const SESSION_COOKIE: &str = "session=53616c7465645f5fac1be7a37b30d67982448ad247a86c8d3bedc4360fbcf8de50a305f6b5e32752aa51b838ed678860";
const LEADERBOARD_ID: u32 = 372543;

pub fn latest_event_year() -> i32 {
    let now = FixedOffset::east(RELEASE_TIMEZONE_OFFSET)
        .from_utc_datetime(&Utc::now().naive_utc());
    if now.month() < EVENT_START_MONTH {
        now.year() - 1
    } else {
        now.year()
    }
}

pub fn is_valid_event_year(year: i32) -> bool {
    year >= FIRST_EVENT_YEAR && year <= latest_event_year()
}

pub fn fetch_leaderboard(year: i32) -> reqwest::Result<String> {
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, HeaderValue::from_static(SESSION_COOKIE));

    let client = Client::builder().default_headers(headers).build()?;
    let url = format!(
        "https://adventofcode.com/{}/leaderboard/private/view/{}.json",
        year, LEADERBOARD_ID
    );
    client.get(&url).send()?.text()
}
