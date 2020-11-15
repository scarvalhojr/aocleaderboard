use crate::aoc::*;
use crate::leaders::*;
use crate::AppSettings;
use chrono::{DateTime, FixedOffset, Utc};
use conv::ConvUtil;
use log::error;
use rocket::{get, http::RawStr, http::Status, request::FromFormValue, State};
use rocket_contrib::templates::Template;
use serde::Serialize;
use std::convert::TryFrom;
use std::sync::{Arc, RwLock};

#[derive(Clone, Copy)]
pub struct AsOf(DateTime<FixedOffset>);

impl<'v> FromFormValue<'v> for AsOf {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        DateTime::parse_from_rfc3339(form_value.url_decode_lossy().as_str())
            .map(AsOf)
            .map_err(|_| form_value)
    }
}

impl<'v> FromFormValue<'v> for LeaderboardOrder {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        Self::try_from(form_value.url_decode_lossy().as_str())
            .map_err(|_| form_value)
    }
}

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

#[derive(Serialize)]
struct LeaderboardContext<'a> {
    year: EventYear,
    as_of_str: Option<String>,
    leaderboard_name: &'a str,
    members: Vec<ScoredMember>,
    leaderboard_order: LeaderboardOrder,
    leaderboard_default_order: &'a LeaderboardOrder,
    leaderboard_update_sec: u64,
    updated_at_str: String,
    rank_str: Vec<String>,
    score_str: Vec<String>,
    table_head_pad: String,
}

fn number_width(num: usize) -> usize {
    1 + num.value_as::<f64>().unwrap_or(0_f64).log10().floor() as usize
}

impl<'a> LeaderboardContext<'a> {
    fn build(
        settings: &'a AppSettings,
        year: EventYear,
        as_of: Option<AsOf>,
        leaderboard: Leaderboard,
        leaderboard_order: LeaderboardOrder,
    ) -> Self {
        let as_of_str = as_of.map(|AsOf(dt)| dt.to_string());
        let updated_at_str =
            Into::<DateTime<Utc>>::into(leaderboard.updated_at())
                .format("%F %T %Z")
                .to_string();
        let members = leaderboard.get_members();
        let rank_width = number_width(members.len());
        let rank_str = (1..=members.len())
            .map(|rank| format!("{:width$}", rank, width = rank_width))
            .collect::<Vec<_>>();
        let score_width = number_width(
            members.get(0).map(|member| member.get_score()).unwrap_or(0),
        );
        let score_str = members
            .iter()
            .map(|m| format!("{:width$}", m.get_score(), width = score_width))
            .collect::<Vec<_>>();
        let table_head_pad =
            vec![' '; rank_width + score_width].into_iter().collect();

        Self {
            year,
            as_of_str,
            leaderboard_name: &settings.leaderboard_name,
            members,
            leaderboard_order,
            leaderboard_default_order: &settings.leaderboard_default_order,
            leaderboard_update_sec: settings.leaderboard_update_sec,
            updated_at_str,
            rank_str,
            score_str,
            table_head_pad,
        }
    }
}

fn render_leaderboard(
    settings: &AppSettings,
    event_mgr: Arc<RwLock<EventManager>>,
    year: EventYear,
    leaderboard_order: Option<LeaderboardOrder>,
    as_of: Option<AsOf>,
) -> Result<Template, Status> {
    let order = leaderboard_order.unwrap_or(settings.leaderboard_default_order);
    get_leaderboard(
        event_mgr,
        year,
        order,
        as_of.map(|AsOf(dt)| dt.timestamp()),
    )
    .map(|leaderboard| {
        let context = LeaderboardContext::build(
            &settings,
            year,
            as_of,
            leaderboard,
            order,
        );
        Template::render("leaderboard", &context)
    })
    .map_err(|err| {
        error!("Failed to fetch {} event: {}", year, err);
        // TODO: customize 500 page
        Status::InternalServerError
    })
}

#[derive(Serialize)]
struct EventsContext {
    year: EventYear,
    events: Vec<EventYear>,
}

#[get("/events")]
pub fn events(settings: State<Arc<AppSettings>>) -> Template {
    let year = settings.latest_event_year.unwrap_or_else(latest_event_year);
    render_event(&settings, year)
}

#[get("/<year>/events")]
pub fn events_year(
    settings: State<Arc<AppSettings>>,
    year: EventYear,
) -> Result<Template, Status> {
    if Some(year) == settings.latest_event_year || is_valid_event_year(year) {
        Ok(render_event(&settings, year))
    } else {
        // TODO: customize 404 page
        Err(Status::NotFound)
    }
}

fn render_event(settings: &AppSettings, year: EventYear) -> Template {
    let latest_year = match settings.latest_event_year {
        Some(y) => y.max(latest_event_year()),
        _ => latest_event_year(),
    };
    let events = (FIRST_EVENT_YEAR..=latest_year).rev().collect::<Vec<_>>();
    let context = EventsContext { year, events };
    Template::render("events", &context)
}
