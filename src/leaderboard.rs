use crate::aoc::*;
use crate::leaders::*;
use crate::util::*;
use crate::AppSettings;
use chrono::{DateTime, FixedOffset, Utc};
use log::error;
use rocket::{http::RawStr, http::Status, request::FromFormValue};
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

pub fn render_leaderboard(
    settings: &AppSettings,
    event_mgr: Arc<RwLock<EventManager>>,
    year: EventYear,
    leaderboard_order: Option<LeaderboardOrder>,
    as_of: Option<AsOf>,
) -> Result<Template, Status> {
    let order = leaderboard_order.unwrap_or(settings.leaderboard_default_order);
    let leaderboard = get_leaderboard(
        event_mgr,
        year,
        order,
        as_of.map(|AsOf(dt)| dt.timestamp()),
    )
    .map_err(|err| {
        error!("Failed to fetch {} event: {}", year, err);
        // TODO: customize 500 page
        Status::InternalServerError
    })?;
    let context = Context::build(&settings, year, as_of, leaderboard, order);
    Ok(Template::render("leaderboard", &context))
}

#[derive(Serialize)]
struct Context<'a> {
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
    last_unlock_day: i64,
}

impl<'a> Context<'a> {
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
        let last_unlock_day = last_unlock_day(year);

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
            last_unlock_day,
        }
    }
}
