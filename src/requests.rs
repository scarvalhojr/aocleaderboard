use crate::aoc::{is_valid_event_year, latest_event_year};
use crate::leaders::{get_leaders, EventManager, EventYear};
use chrono::{DateTime, FixedOffset};
use rocket::{http::RawStr, request::FromFormValue, State};
use std::sync::{Arc, RwLock};

#[derive(Clone, Copy)]
pub struct AsOf(DateTime<FixedOffset>);

impl AsOf {
    pub fn timesamp(&self) -> i64 {
        self.0.timestamp()
    }
}

impl<'v> FromFormValue<'v> for AsOf {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<AsOf, &'v RawStr> {
        DateTime::parse_from_rfc3339(&form_value.url_decode_lossy())
            .map(AsOf)
            .map_err(|_| form_value)
    }
}

#[get("/?<as_of>")]
pub fn index(
    event_mgr: State<Arc<RwLock<EventManager>>>,
    as_of: Option<AsOf>,
) -> String {
    render_leaderboard(event_mgr.clone(), latest_event_year(), as_of)
}

#[get("/<year>?<as_of>")]
pub fn event_year(
    event_mgr: State<Arc<RwLock<EventManager>>>,
    year: EventYear,
    as_of: Option<AsOf>,
) -> Option<String> {
    if is_valid_event_year(year) {
        Some(render_leaderboard(event_mgr.clone(), year, as_of))
    } else {
        None
    }
}

fn render_leaderboard(
    event_mgr: Arc<RwLock<EventManager>>,
    year: EventYear,
    as_of: Option<AsOf>,
) -> String {
    let leaders = get_leaders(event_mgr, year, as_of.map(|a| a.timesamp()));
    if let Some(AsOf(datetime)) = as_of {
        format!("Year {}, as of {}:\n{:?}", year, datetime, leaders)
    } else {
        format!("Year {}, as of now:\n{:?}", year, leaders)
    }
}
