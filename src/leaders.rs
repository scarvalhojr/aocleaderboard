use crate::aoc::*;
use chrono::{DateTime, FixedOffset, Utc};
use log::debug;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

pub type EventYear = i32;
pub type Score = usize;

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
            event.last_updated.elapsed().map_or(0, |dur| dur.as_secs())
                < self.update_sec
        })
    }

    fn update_event(&mut self, year: EventYear) -> Result<(), Box<dyn Error>> {
        debug!("Starting update_event for {} event", year);
        if self.get_event(year).is_some() {
            debug!("{} event is already up to date", year);
            return Ok(());
        }

        let members =
            fetch_members(year, &self.leaderboard_ids, &self.session_cookie)?;

        debug!("Building new event object for {}", year);
        self.events.insert(year, Event::new(members));
        Ok(())
    }
}

pub struct Event {
    last_updated: SystemTime,
    members: HashSet<Member>,
}

impl Event {
    fn new(members: HashSet<Member>) -> Self {
        Self {
            last_updated: SystemTime::now(),
            members,
        }
    }

    pub fn score_members(&self) -> Vec<ScoredMember> {
        debug!("Scoring members...");
        let members_count = self.members.len();

        let mut puzzles = HashMap::new();
        for member in self.members.iter() {
            let member_id = member.get_id();
            for (puzzle_id, ts) in member.completed_puzzles() {
                puzzles
                    .entry(*puzzle_id)
                    .or_insert_with(BTreeMap::new)
                    .entry(*ts)
                    .or_insert_with(HashSet::new)
                    .insert(member_id);
            }
        }

        let mut scores: HashMap<MemberId, Score> = HashMap::new();
        for (puzzle_id, solutions) in puzzles.iter() {
            let mut score = members_count;
            for (ts, members) in solutions.iter() {
                if members.len() != 1 {
                    // TODO: how to break ties?
                    debug!(
                        "Puzzle {:?} solved at {} by {:?}",
                        puzzle_id, ts, members
                    );
                }
                for member_id in members.iter() {
                    *scores.entry(*member_id).or_insert(0) += score;
                    score -= 1;
                }
            }
        }

        let mut scored_members = self
            .members
            .iter()
            .map(|member| ScoredMember {
                name: member.get_name().clone(),
                stars: Vec::new(),
                score: *scores.get(&member.get_id()).unwrap_or(&0),
            })
            .collect::<Vec<_>>();
        scored_members.sort_unstable_by(|a, b| b.score.cmp(&a.score));
        debug!("Scored {} members", members_count);
        scored_members
    }
}

#[derive(Serialize)]
pub struct ScoredMember {
    name: String,
    stars: Vec<CompletionLevel>,
    score: Score,
}

#[derive(Serialize)]
pub struct Leaderboard {
    year: EventYear,
    last_updated_str: String,
    as_of_str: String,
    scored_members: Vec<ScoredMember>,
}

impl Leaderboard {
    fn new(
        year: EventYear,
        last_updated: &SystemTime,
        as_of: &Option<DateTime<FixedOffset>>,
        scored_members: Vec<ScoredMember>,
    ) -> Self {
        let last_updated_str =
            Into::<DateTime<Utc>>::into(*last_updated).to_string();
        let as_of_str = as_of.map(|dt| dt.to_string()).unwrap_or_default();
        Self {
            year,
            last_updated_str,
            as_of_str,
            scored_members,
        }
    }
}

pub fn get_leaderboard(
    event_mgr: Arc<RwLock<EventManager>>,
    year: EventYear,
    as_of: Option<DateTime<FixedOffset>>,
) -> Result<Leaderboard, Box<dyn Error>> {
    loop {
        // TODO: handle LockResult errors
        debug!("Attempting to read {} event", year);
        // TODO: use as_of
        if let Some(event) = event_mgr.read().unwrap().get_event(year) {
            debug!("Returning members of {} event", year);
            return Ok(Leaderboard::new(
                year,
                &event.last_updated,
                &as_of,
                event.score_members(),
            ));
        }

        // TODO: handle LockResult errors
        debug!("Attempting to update {} event", year);
        event_mgr.write().unwrap().update_event(year)?;
    }
}
