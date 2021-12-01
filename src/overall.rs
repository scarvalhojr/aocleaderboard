use crate::aoc::*;
use crate::leaders::*;
use crate::util::*;
use crate::AppSettings;
use chrono::{DateTime, Utc};
use log::error;
use rocket::http::Status;
use rocket_contrib::templates::Template;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

const MIN_COL_WIDTH: usize = 6;

pub fn render_overall(
    settings: &AppSettings,
    event_mgr: Arc<RwLock<EventManager>>,
    leaderboard_order: Option<LeaderboardOrder>,
) -> Result<Template, Status> {
    let from_year = FIRST_EVENT_YEAR;
    let to_year = settings.latest_event_year.unwrap_or_else(latest_event_year);
    let order = leaderboard_order.unwrap_or(settings.leaderboard_default_order);
    let leaderboard =
        build_overall_leaderboard(event_mgr, order, from_year, to_year)
            .map_err(|err| {
                error!("Failed to fetch events: {}", err);
                // TODO: customize 500 page
                Status::InternalServerError
            })?;
    let context = Context::build(settings, to_year, leaderboard, order);
    Ok(Template::render("overall", &context))
}

fn build_overall_leaderboard(
    event_mgr: Arc<RwLock<EventManager>>,
    order: LeaderboardOrder,
    from_year: EventYear,
    to_year: EventYear,
) -> Result<OverallLeaderboard, Box<dyn Error>> {
    let years = (from_year..=to_year).into_iter().collect::<Vec<_>>();
    let mut updated_at = SystemTime::now();
    let mut member_map: HashMap<MemberId, OverallScoredMember> = HashMap::new();

    // TODO: fetch leaderboards concurrently
    for &year in years.iter() {
        let leaderboard =
            get_leaderboard(event_mgr.clone(), year, order, None)?;
        updated_at = updated_at.min(leaderboard.updated_at());
        for member in leaderboard.get_members() {
            member_map
                .entry(member.get_id())
                .and_modify(|m| m.add_score(year, member.get_score()))
                .or_insert_with(|| OverallScoredMember::from(&member, year));
        }
    }

    let mut members = member_map
        .into_iter()
        .map(|(_, member)| member)
        .collect::<Vec<_>>();
    members.sort_unstable();
    members.reverse();

    Ok(OverallLeaderboard {
        updated_at,
        years,
        members,
    })
}

#[derive(Serialize)]
struct OverallLeaderboard {
    updated_at: SystemTime,
    years: Vec<EventYear>,
    members: Vec<OverallScoredMember>,
}

#[derive(Eq, Serialize)]
struct OverallScoredMember {
    id: MemberId,
    name: String,
    scores: HashMap<EventYear, Score>,
    last_star: Timestamp,
    overall_score: Score,
}

impl OverallScoredMember {
    fn from(member: &ScoredMember, year: EventYear) -> Self {
        Self {
            id: member.get_id(),
            name: member.get_name(),
            scores: vec![(year, member.get_score())].into_iter().collect(),
            last_star: member.get_last_star(),
            overall_score: member.get_score(),
        }
    }

    fn add_score(&mut self, year: EventYear, score: Score) {
        self.scores.insert(year, score);
        self.overall_score = self.scores.values().sum();
    }
}

impl Ord for OverallScoredMember {
    fn cmp(&self, other: &Self) -> Ordering {
        self.overall_score
            .cmp(&other.overall_score)
            .then(other.last_star.cmp(&self.last_star))
            .then(other.id.cmp(&self.id))
    }
}

impl PartialOrd for OverallScoredMember {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for OverallScoredMember {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Serialize)]
struct Context<'a> {
    year: EventYear,
    leaderboard_name: &'a str,
    updated_at: String,
    leaderboard_update_sec: u64,
    leaderboard_order: LeaderboardOrder,
    leaderboard_default_order: &'a LeaderboardOrder,
    table_head_pad: String,
    years: Vec<EventYear>,
    col_header: Vec<String>,
    rank: Vec<String>,
    overall_score: Vec<String>,
    scores: Vec<String>,
    member_name: Vec<String>,
}

impl<'a> Context<'a> {
    fn build(
        settings: &'a AppSettings,
        year: EventYear,
        leaderboard: OverallLeaderboard,
        leaderboard_order: LeaderboardOrder,
    ) -> Self {
        let leaderboard_name = &settings.leaderboard_name;
        let updated_at = Into::<DateTime<Utc>>::into(leaderboard.updated_at)
            .format("%F %T %Z")
            .to_string();
        let leaderboard_update_sec = settings.leaderboard_update_sec;
        let years = leaderboard.years.clone();

        let col_width = years
            .iter()
            .map(|year| {
                leaderboard
                    .members
                    .iter()
                    .map(|member| member.scores.get(year).unwrap_or(&0))
                    .max()
                    .unwrap_or(&0)
            })
            .max()
            .map(|max_score| (2 + number_width(*max_score)).max(MIN_COL_WIDTH))
            .unwrap_or(MIN_COL_WIDTH);
        let col_header = years
            .iter()
            .map(|year| {
                format!("{:width$}", year, width = col_width)
                    .replace(' ', "&nbsp;")
            })
            .collect();

        let rank_width = number_width(leaderboard.members.len());
        let rank = (1..=leaderboard.members.len())
            .map(|rank| format!("{:width$}) ", rank, width = rank_width))
            .collect::<Vec<_>>();

        let score_width = number_width(
            leaderboard
                .members
                .get(0)
                .map(|member| member.overall_score)
                .unwrap_or(0),
        );
        let overall_score = leaderboard
            .members
            .iter()
            .map(|member| {
                format!("{:width$}", member.overall_score, width = score_width,)
            })
            .collect::<Vec<_>>();

        let table_head_pad = vec![' '; rank_width + score_width + 2]
            .into_iter()
            .collect();

        let scores = leaderboard
            .members
            .iter()
            .map(|member| {
                years
                    .iter()
                    .map(|year| member.scores.get(year).unwrap_or(&0))
                    .map(|score| format!("{:width$}", score, width = col_width))
                    .collect::<Vec<_>>()
                    .concat()
            })
            .collect();

        let member_name = leaderboard
            .members
            .iter()
            .map(|member| member.name.clone())
            .collect();

        Self {
            year,
            leaderboard_name,
            updated_at,
            leaderboard_update_sec,
            leaderboard_order,
            leaderboard_default_order: &settings.leaderboard_default_order,
            table_head_pad,
            years,
            col_header,
            rank,
            overall_score,
            scores,
            member_name,
        }
    }
}
