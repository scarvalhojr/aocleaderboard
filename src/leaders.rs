use crate::aoc::*;
use log::debug;
use serde::{Deserialize, Serialize};
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::convert::TryFrom;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

pub struct EventManager {
    leaderboard_ids: Vec<String>,
    session_cookie: String,
    update_sec: u64,
    events: HashMap<EventYear, Event>,
}

impl EventManager {
    pub fn new(
        leaderboard_ids: Vec<String>,
        session_cookie: String,
        update_sec: u64,
    ) -> Self {
        Self {
            leaderboard_ids,
            session_cookie,
            update_sec,
            events: HashMap::new(),
        }
    }

    fn get_event(&self, year: EventYear) -> Option<&Event> {
        self.events.get(&year).filter(|&event| {
            event.updated_at.elapsed().map_or(0, |dur| dur.as_secs())
                < self.update_sec
        })
    }

    fn update_event(&mut self, year: EventYear) -> Result<(), Box<dyn Error>> {
        if self.get_event(year).is_some() {
            debug!("{} event is already up to date", year);
            return Ok(());
        }

        debug!("Updating {} event", year);
        let updated_at = SystemTime::now();
        let members =
            fetch_members(year, &self.leaderboard_ids, &self.session_cookie)?;

        self.events.insert(year, Event::new(members, updated_at));
        Ok(())
    }
}

struct Event {
    members: HashSet<Member>,
    updated_at: SystemTime,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum LeaderboardOrder {
    #[serde(rename = "local_score")]
    LocalScore,

    #[serde(rename = "stars")]
    Stars,
}

impl TryFrom<&str> for LeaderboardOrder {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "local_score" => Ok(Self::LocalScore),
            "stars" => Ok(Self::Stars),
            _ => Err("Invalid leaderboard order"),
        }
    }
}

impl Event {
    fn new(members: HashSet<Member>, updated_at: SystemTime) -> Self {
        Self {
            members,
            updated_at,
        }
    }

    fn build_leaderboard(
        &self,
        order: LeaderboardOrder,
        as_of: Option<Timestamp>,
    ) -> Leaderboard {
        let mut scored_members = match order {
            LeaderboardOrder::LocalScore => self.local_score(as_of),
            LeaderboardOrder::Stars => self.star_score(as_of),
        };
        scored_members.sort_unstable();
        scored_members.reverse();
        Leaderboard::new(self.updated_at, scored_members)
    }

    fn local_score(&self, as_of: Option<Timestamp>) -> Vec<ScoredMember> {
        let mut puzzles = HashMap::new();
        for member in self.members.iter() {
            for (puzzle_id, ts) in member.iter_completed() {
                if as_of.map(|timestamp| *ts <= timestamp).unwrap_or(true) {
                    puzzles
                        .entry(*puzzle_id)
                        .or_insert_with(BinaryHeap::new)
                        .push(Reverse((ts, member)));
                }
            }
        }

        let mut scores = HashMap::new();
        let max_points = self.members.len();
        for (_, mut solutions) in puzzles.drain() {
            let mut puzzle_points = max_points;
            while let Some(Reverse((_, member))) = solutions.pop() {
                *scores.entry(member).or_insert(0) += puzzle_points;
                puzzle_points -= 1;
            }
        }

        self.members
            .iter()
            .map(|member| {
                ScoredMember::build(
                    member,
                    as_of,
                    *scores.get(member).unwrap_or(&0),
                )
            })
            .collect::<Vec<_>>()
    }

    fn star_score(&self, as_of: Option<Timestamp>) -> Vec<ScoredMember> {
        self.members
            .iter()
            .map(|member| {
                ScoredMember::build(member, as_of, member.star_count(as_of))
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Eq, Serialize)]
pub struct ScoredMember {
    id: MemberId,
    name: String,
    stars: Vec<CompletionLevel>,
    last_star: Timestamp,
    score: Score,
}

impl ScoredMember {
    fn build(member: &Member, as_of: Option<Timestamp>, score: Score) -> Self {
        Self {
            id: member.get_id(),
            name: member.get_name().clone(),
            stars: member.get_stars(as_of),
            last_star: member.get_last_star(as_of),
            score,
        }
    }

    pub fn get_score(&self) -> Score {
        self.score
    }
}

impl Ord for ScoredMember {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .cmp(&other.score)
            .then(other.last_star.cmp(&self.last_star))
    }
}

impl PartialOrd for ScoredMember {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScoredMember {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Serialize)]
pub struct Leaderboard {
    updated_at: SystemTime,
    members: Vec<ScoredMember>,
}

impl Leaderboard {
    fn new(updated_at: SystemTime, members: Vec<ScoredMember>) -> Self {
        Self {
            updated_at,
            members,
        }
    }

    pub fn updated_at(&self) -> SystemTime {
        self.updated_at
    }

    pub fn get_members(self) -> Vec<ScoredMember> {
        self.members
    }
}

pub fn get_leaderboard(
    event_mgr: Arc<RwLock<EventManager>>,
    year: EventYear,
    leaderboard_order: LeaderboardOrder,
    as_of: Option<Timestamp>,
) -> Result<Leaderboard, Box<dyn Error>> {
    loop {
        // TODO: handle LockResult errors
        debug!("Attempting to acquire read lock on {} event", year);
        if let Some(event) = event_mgr.read().unwrap().get_event(year) {
            debug!("Building leaderboard for {} event", year);
            return Ok(event.build_leaderboard(leaderboard_order, as_of));
        }

        // TODO: handle LockResult errors
        debug!(
            "{} event needs to be updated, attempting to acquire write lock",
            year
        );
        event_mgr.write().unwrap().update_event(year)?;
    }
}
