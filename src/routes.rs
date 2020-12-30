use crate::aoc::*;
use crate::events::*;
use crate::leaderboard::*;
use crate::leaders::*;
use crate::overall::*;
use crate::AppSettings;
use rocket::{get, http::Status, State};
use rocket_contrib::templates::Template;
use std::sync::{Arc, RwLock};

#[get("/?<as_of>&<order>")]
pub fn leaderboard(
    settings: State<Arc<AppSettings>>,
    event_mgr: State<Arc<RwLock<EventManager>>>,
    as_of: Option<AsOf>,
    order: Option<LeaderboardOrder>,
) -> Result<Template, Status> {
    let year = settings.latest_event_year.unwrap_or_else(latest_event_year);
    render_leaderboard(&settings, event_mgr.clone(), year, order, as_of)
}

#[get("/<year>?<as_of>&<order>")]
pub fn leaderboard_year(
    settings: State<Arc<AppSettings>>,
    event_mgr: State<Arc<RwLock<EventManager>>>,
    year: EventYear,
    as_of: Option<AsOf>,
    order: Option<LeaderboardOrder>,
) -> Result<Template, Status> {
    if Some(year) == settings.latest_event_year || is_valid_event_year(year) {
        render_leaderboard(&settings, event_mgr.clone(), year, order, as_of)
    } else {
        // TODO: customize 404 page
        Err(Status::NotFound)
    }
}

#[get("/events")]
pub fn events(settings: State<Arc<AppSettings>>) -> Template {
    let year = settings.latest_event_year.unwrap_or_else(latest_event_year);
    render_events(&settings, year)
}

#[get("/<year>/events")]
pub fn events_year(
    settings: State<Arc<AppSettings>>,
    year: EventYear,
) -> Result<Template, Status> {
    if Some(year) == settings.latest_event_year || is_valid_event_year(year) {
        Ok(render_events(&settings, year))
    } else {
        // TODO: customize 404 page
        Err(Status::NotFound)
    }
}

#[get("/overall?<order>")]
pub fn overall(
    settings: State<Arc<AppSettings>>,
    event_mgr: State<Arc<RwLock<EventManager>>>,
    order: Option<LeaderboardOrder>,
) -> Result<Template, Status> {
    render_overall(&settings, event_mgr.clone(), order)
}
