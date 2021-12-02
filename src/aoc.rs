use chrono::{Datelike, FixedOffset, NaiveDate, TimeZone, Utc};
use futures::future::join_all;
use log::info;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use reqwest::redirect::Policy;
use reqwest::{Client, RequestBuilder};
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::{hash_map::Iter, HashMap, HashSet};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};

pub type EventYear = i32;
pub type MemberId = i32;
pub type PuzzleDay = u8;
pub type PuzzlePart = u8;
pub type PuzzleId = (PuzzleDay, PuzzlePart);
pub type Timestamp = i64;
pub type CompletionLevel = u8;
pub type Score = usize;

pub const FIRST_EVENT_YEAR: EventYear = 2015;
const NUM_PUZZLE_DAYS: PuzzleDay = 25;
const EVENT_START_DAY: u32 = 1;
const EVENT_START_MONTH: u32 = 12;
const RELEASE_TIMEZONE_OFFSET: i32 = -5 * 3600;

pub fn latest_event_year() -> i32 {
    let now = FixedOffset::east(RELEASE_TIMEZONE_OFFSET)
        .from_utc_datetime(&Utc::now().naive_utc());
    if now.month() < EVENT_START_MONTH {
        now.year() - 1
    } else {
        now.year()
    }
}

pub fn last_unlock_day(year: i32) -> i64 {
    let timezone: FixedOffset = FixedOffset::east(RELEASE_TIMEZONE_OFFSET);
    if let Some(event_start) = timezone
        .from_local_datetime(
            &NaiveDate::from_ymd(year, EVENT_START_MONTH, EVENT_START_DAY)
                .and_hms(0, 0, 0),
        )
        .single()
    {
        let duration = timezone
            .from_utc_datetime(&Utc::now().naive_utc())
            .signed_duration_since(event_start);
        if duration.num_milliseconds() >= 0 {
            return (1 + duration.num_days()).min(NUM_PUZZLE_DAYS.into());
        }
    }
    0
}

pub fn is_valid_event_year(year: i32) -> bool {
    year >= FIRST_EVENT_YEAR && year <= latest_event_year()
}

#[derive(Eq, Debug)]
pub struct Member {
    id: MemberId,
    name: String,
    completed: HashMap<PuzzleId, Timestamp>,
}

impl Member {
    fn new(id: MemberId, opt_name: Option<String>) -> Self {
        let name = opt_name.unwrap_or(format!("(anonymous user #{})", id));
        Self {
            id,
            name,
            completed: HashMap::new(),
        }
    }

    pub fn get_id(&self) -> MemberId {
        self.id
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_stars(&self, as_of: Option<Timestamp>) -> Vec<CompletionLevel> {
        let mut stars = vec![0; usize::from(NUM_PUZZLE_DAYS)];
        for (&(day, _), _) in self.completed.iter().filter(|&(_, ts)| {
            as_of.map(|timestamp| *ts <= timestamp).unwrap_or(true)
        }) {
            if day > 0 && day <= NUM_PUZZLE_DAYS {
                stars[usize::from(day - 1)] += 1;
            }
        }
        stars
    }

    pub fn get_last_star(&self, as_of: Option<Timestamp>) -> Timestamp {
        self.completed
            .iter()
            .map(|(_, ts)| *ts)
            .filter(|&ts| {
                as_of.map(|timestamp| ts <= timestamp).unwrap_or(true)
            })
            .max()
            .unwrap_or(0)
    }

    pub fn star_count(&self, as_of: Option<Timestamp>) -> Score {
        self.completed
            .iter()
            .filter(|&(_, ts)| {
                as_of.map(|timestamp| *ts <= timestamp).unwrap_or(true)
            })
            .count()
    }

    fn add_star(&mut self, puzzle_id: PuzzleId, timestamp: Timestamp) {
        self.completed.insert(puzzle_id, timestamp);
    }

    pub fn iter_completed(&self) -> Iter<PuzzleId, Timestamp> {
        self.completed.iter()
    }
}

impl Ord for Member {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Member {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Member {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Member {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug)]
pub struct ResponseFormatError {
    error: String,
}

impl ResponseFormatError {
    fn new(error: String) -> Self {
        Self { error }
    }
}

impl Error for ResponseFormatError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for ResponseFormatError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Response format error: {}", self.error)
    }
}

#[tokio::main]
pub async fn fetch_members(
    year: i32,
    leaderboard_ids: &[String],
    exclude_members: &HashSet<MemberId>,
    session_cookie: &str,
) -> Result<HashSet<Member>, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, HeaderValue::from_str(session_cookie)?);

    let client = Client::builder()
        .default_headers(headers)
        .redirect(Policy::none())
        .build()?;

    let responses = join_all(leaderboard_ids.iter().map(|leaderboard_id| {
        let url = format!(
            "https://adventofcode.com/{}/leaderboard/private/view/{}.json",
            year, leaderboard_id
        );
        info!("Fetching {}", url);
        fetch_leaderboard(client.get(&url))
    }))
    .await;

    let mut all_members = HashSet::new();
    for resp in responses {
        let mut members = resp?;
        info!("Fetched {} members", members.len());
        all_members.extend(
            members.drain().filter(|m| !exclude_members.contains(&m.id)),
        )
    }

    let star_count: usize = all_members
        .iter()
        .map(|member| member.completed.len())
        .sum();
    info!("{} unique members found", all_members.len());
    info!("{} stars collected in {} event", star_count, year);
    Ok(all_members)
}

async fn fetch_leaderboard(
    request: RequestBuilder,
) -> Result<HashSet<Member>, Box<dyn Error>> {
    request
        .send()
        .await?
        .json::<Value>()
        .await?
        .get("members")
        .and_then(|val| val.as_object())
        .map(|obj| obj.values())
        .ok_or_else(|| {
            Box::new(ResponseFormatError::new(
                "'members' field missing or not an object".to_string(),
            ))
        })?
        .map(|value| {
            Member::try_from(value)
                .map_err(|err| Box::new(ResponseFormatError::new(err)) as _)
        })
        .collect::<Result<HashSet<_>, _>>()
}

impl TryFrom<&Value> for Member {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let id = value
            .get("id")
            .and_then(|val| val.as_str())
            .ok_or_else(|| "'id' missing or not a string".to_string())?
            .parse::<i32>()
            .map_err(|err| format!("invalid 'id': {}", err))?;
        let name = value
            .get("name")
            .and_then(|val| val.as_str())
            .map(|s| s.to_string());

        let mut member = Member::new(id, name);

        let completed = value
            .get("completion_day_level")
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                "'completion_day_level' missing or invalid".to_string()
            })?;

        for (day_str, day_value) in completed.iter() {
            let day = day_str.parse::<PuzzleDay>().map_err(|err| {
                format!("invalid puzzle day {}: {}", day_str, err)
            })?;
            if let Some(parts_obj) = day_value.as_object() {
                for (part_str, parts_value) in parts_obj.iter() {
                    let part =
                        part_str.parse::<PuzzlePart>().map_err(|err| {
                            format!("invalid puzzle part {}: {}", part_str, err)
                        })?;
                    let timestamp = parts_value
                        .as_object()
                        .and_then(|obj| obj.get("get_star_ts"))
                        .ok_or_else(|| {
                            format!(
                                "'get_star_ts' missing for member {}, day {}",
                                id, day
                            )
                        })?
                        .as_i64()
                        .ok_or_else(|| {
                            format!(
                                "invalid 'get_star_ts' for member {}, day {}",
                                id, day
                            )
                        })?;
                    member.add_star((day, part), timestamp);
                }
            }
        }

        Ok(member)
    }
}
