use crate::aoc::*;
use log::debug;
use serde::Serialize;
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};
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

impl Event {
    fn new(members: HashSet<Member>, updated_at: SystemTime) -> Self {
        Self {
            members,
            updated_at,
        }
    }

    fn build_leaderboard(&self) -> Leaderboard {
        Leaderboard::new(self.updated_at, self.score_members())
    }

    fn score_members(&self) -> Vec<ScoredMember> {
        let mut puzzles = HashMap::new();
        for member in self.members.iter() {
            for (puzzle_id, ts) in member.iter_completed() {
                puzzles
                    .entry(*puzzle_id)
                    .or_insert_with(BinaryHeap::new)
                    .push(Reverse((ts, member)));
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

        let mut scored_members = self
            .members
            .iter()
            .map(|member| ScoredMember::new(member, *scores.get(member).unwrap_or(&0)))
            .collect::<Vec<_>>();
        scored_members.sort_unstable();
        scored_members.reverse();
        scored_members
    }
}

#[derive(Eq, Serialize)]
pub struct ScoredMember {
    id: MemberId,
    name: String,
    stars: Vec<CompletionLevel>,
    score: Score,
}

impl ScoredMember {
    fn new(member: &Member, score: Score) -> Self {
        Self {
            id: member.get_id(),
            name: member.get_name().clone(),
            stars: member.get_stars(),
            score,
        }
    }

    pub fn get_score(&self) -> Score {
        self.score
    }
}

impl Ord for ScoredMember {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
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
    _as_of: Option<Timestamp>,
) -> Result<Leaderboard, Box<dyn Error>> {
    loop {
        // TODO: handle LockResult errors
        debug!("Attempting to read {} event", year);
        // TODO: use as_of
        if let Some(event) = event_mgr.read().unwrap().get_event(year) {
            debug!("Building leaderboard for {} event", year);
            return Ok(event.build_leaderboard());
        }

        // TODO: handle LockResult errors
        debug!("Attempting to update {} event", year);
        event_mgr.write().unwrap().update_event(year)?;
    }
}
