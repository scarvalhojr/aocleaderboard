use crate::aoc::fetch_leaderboards;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub type EventYear = i32;

#[derive(Debug)]
pub enum EventError {
    NotFound,
    HttpError(String),
}

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
            event.last_updated.elapsed().as_secs() < self.update_sec
        })
    }

    fn update_event(&mut self, year: EventYear) -> Result<(), EventError> {
        debug!("Starting update_event for {} event", year);
        if self.get_event(year).is_some() {
            debug!("{} event is already up to date", year);
            return Ok(())
        }

        // TODO: handle errors
        let _json = fetch_leaderboards(
            year,
            &self.leaderboard_ids,
            &self.session_cookie,
        )
        .map_err(|err| EventError::HttpError(err.to_string()))?;

        debug!("Building new event object for {}", year);
        let last_updated = Instant::now();

        // Build members
        let members = Vec::new();

        let event = Event::new(last_updated, members);
        self.events.insert(year, event);
        debug!("Stored new event object for {}", year);
        Ok(())
    }
}

struct Event {
    last_updated: Instant,
    members: Vec<Member>,
}

impl Event {
    fn new(last_updated: Instant, members: Vec<Member>) -> Self {
        Self {
            last_updated,
            members,
        }
    }
}

#[derive(Clone, Debug, Eq)]
pub struct Member {
    id: u32,
    name: String,
    score: u32,
}

impl Ord for Member {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score).then(self.id.cmp(&other.id))
    }
}

impl PartialOrd for Member {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Member {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub fn get_leaders(
    event_mgr: Arc<RwLock<EventManager>>,
    year: EventYear,
    _timestamp: Option<i64>,
) -> Result<Vec<Member>, EventError> {
    loop {
        // TODO: handle LockResult errors
        debug!("Attempting to read {} event", year);
        if let Some(event) = event_mgr.read().unwrap().get_event(year) {
            debug!("Returning members of {} event", year);
            return Ok(event.members.clone());
        }

        // TODO: handle LockResult errors
        debug!("Attempting to update {} event", year);
        event_mgr.write().unwrap().update_event(year)?;
    }
}
