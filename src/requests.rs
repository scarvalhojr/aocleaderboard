use crate::aoc::{is_valid_event_year, latest_event_year};
use crate::leaders::{get_leaderboard, EventManager, EventYear};
use chrono::{DateTime, FixedOffset};
use log::error;
use rocket::{get, http::RawStr, http::Status, request::FromFormValue, State};
use rocket_contrib::templates::Template;
use std::sync::{Arc, RwLock};

#[derive(Clone, Copy)]
pub struct AsOf(DateTime<FixedOffset>);

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
) -> Result<Template, Status> {
    render_leaderboard(event_mgr.clone(), latest_event_year(), as_of)
}

#[get("/<year>?<as_of>")]
pub fn event_year(
    event_mgr: State<Arc<RwLock<EventManager>>>,
    year: EventYear,
    as_of: Option<AsOf>,
) -> Result<Template, Status> {
    if is_valid_event_year(year) {
        render_leaderboard(event_mgr.clone(), year, as_of)
    } else {
        Err(Status::NotFound)
    }
}

fn render_leaderboard(
    event_mgr: Arc<RwLock<EventManager>>,
    year: EventYear,
    as_of: Option<AsOf>,
) -> Result<Template, Status> {
    // TODO: check as_of is not in the future
    get_leaderboard(event_mgr, year, as_of.map(|a| a.0))
        .map(|leaderboard| Template::render("leaderboard", &leaderboard))
        .map_err(|err| {
            error!("Failed to fetch {} event: {}", year, err);
            Status::InternalServerError
        })
}
