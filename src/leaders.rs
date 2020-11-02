use crate::aoc::*;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub type EventYear = i32;
pub type Score = i32;

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

#[derive(Clone, Debug)]
pub struct Event {
    last_updated: Instant,
    members: HashSet<Member>,
}

impl Event {
    fn new(members: HashSet<Member>) -> Self {
        Self {
            last_updated: Instant::now(),
            members,
        }
    }

    pub fn get_scores(&self) -> HashMap<MemberId, Score> {
        let _ts: HashMap<
            (PuzzleDay, PuzzlePart),
            HashMap<Timestamp, HashSet<MemberId>>,
        > = HashMap::new();
        for member in &self.members {
            for _puzzle in member.completed_puzzles() {
                // TODO
            }
        }
        HashMap::new()
    }
}

pub fn get_event(
    event_mgr: Arc<RwLock<EventManager>>,
    year: EventYear,
) -> Result<Event, Box<dyn Error>> {
    loop {
        // TODO: handle LockResult errors
        debug!("Attempting to read {} event", year);
        if let Some(event) = event_mgr.read().unwrap().get_event(year) {
            debug!("Returning members of {} event", year);
            return Ok(event.clone());
        }

        // TODO: handle LockResult errors
        debug!("Attempting to update {} event", year);
        event_mgr.write().unwrap().update_event(year)?;
    }
}
