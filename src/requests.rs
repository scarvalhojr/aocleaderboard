use crate::aoc::{is_valid_event_year, latest_event_year, EventYear};
use crate::leaders::{
    get_leaderboard, EventManager, Leaderboard, ScoredMember,
};
use crate::AppSettings;
use chrono::{DateTime, FixedOffset, Utc};
use conv::ConvUtil;
use log::error;
use rocket::{get, http::RawStr, http::Status, request::FromFormValue, State};
use rocket_contrib::templates::Template;
use serde::Serialize;
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
    settings: State<Arc<AppSettings>>,
    event_mgr: State<Arc<RwLock<EventManager>>>,
    as_of: Option<AsOf>,
) -> Result<Template, Status> {
    render_leaderboard(&settings, event_mgr.clone(), latest_event_year(), as_of)
}

#[get("/<year>?<as_of>")]
pub fn event_year(
    settings: State<Arc<AppSettings>>,
    event_mgr: State<Arc<RwLock<EventManager>>>,
    year: EventYear,
    as_of: Option<AsOf>,
) -> Result<Template, Status> {
    if is_valid_event_year(year) {
        render_leaderboard(&settings, event_mgr.clone(), year, as_of)
    } else {
        Err(Status::NotFound)
    }
}

#[derive(Serialize)]
struct Context<'a> {
    year: EventYear,
    as_of_str: Option<String>,
    leaderboard_name: &'a str,
    members: Vec<ScoredMember>,
    leaderboard_update_sec: u64,
    updated_at_str: String,
    rank_str: Vec<String>,
    score_str: Vec<String>,
    table_head_pad: String,
}

fn number_width(num: usize) -> usize {
    1 + num.value_as::<f64>().unwrap_or(0_f64).log10().floor() as usize
}

impl<'a> Context<'a> {
    fn build(
        year: EventYear,
        as_of: Option<AsOf>,
        leaderboard_name: &'a str,
        leaderboard: Leaderboard,
        leaderboard_update_sec: u64,
    ) -> Self {
        let as_of_str = as_of.map(|AsOf(dt)| dt.to_string());
        let updated_at_str =
            Into::<DateTime<Utc>>::into(leaderboard.updated_at())
                .format("%F %T %Z")
                .to_string();
        let members = leaderboard.get_members();
        let rank_width = number_width(members.len());
        let rank_str = (0..members.len())
            .map(|r| format!("{:width$}) ", r + 1, width = rank_width))
            .collect::<Vec<_>>();
        let score_width = number_width(1 + members
            .get(0)
            .map(|member| member.get_score())
            .unwrap_or(0)
        );
        let score_str = members
            .iter()
            .map(|m| format!("{:width$} ", m.get_score(), width = score_width))
            .collect::<Vec<_>>();
        let table_head_pad = vec![' '; rank_width + score_width + 3]
            .into_iter()
            .collect();

        Self {
            year,
            as_of_str,
            leaderboard_name,
            members,
            leaderboard_update_sec,
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
    as_of: Option<AsOf>,
) -> Result<Template, Status> {
    // TODO: check as_of is not in the future
    get_leaderboard(event_mgr, year, as_of.map(|AsOf(dt)| dt.timestamp()))
        .map(|leaderboard| {
            let context = Context::build(
                year,
                as_of,
                &settings.leaderboard_name,
                leaderboard,
                settings.leaderboard_update_sec,
            );
            Template::render("leaderboard", &context)
        })
        .map_err(|err| {
            error!("Failed to fetch {} event: {}", year, err);
            Status::InternalServerError
        })
}
