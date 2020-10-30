use crate::aoc::{fetch_leaderboard, is_valid_event_year};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::{Arc, RwLock};
use std::time::Instant;

const LEADERBOARD_UPDATE_SEC: u64 = 10 * 60;

pub type EventYear = i32;

pub struct Events {
    events: HashMap<EventYear, Event>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    fn get_event(&self, year: EventYear) -> Option<&Event> {
        self.events.get(&year)
    }

    fn update_event(&mut self, year: EventYear) -> Option<&Event> {
        if let Some(event) = Event::fetch_and_build(year) {
            self.events.insert(year, event);
            self.events.get(&year)
        } else {
            None
        }
    }
}

struct Event {
    last_updated: Instant,
    members: BinaryHeap<Member>,
}

impl Event {
    fn fetch_and_build(year: EventYear) -> Option<Self> {
        if !is_valid_event_year(year) {
            None
        } else {
            Some(Self {
                last_updated: Instant::now(),
                members: BinaryHeap::new(),
            })
        }
    }

    fn is_up_to_date(&self) -> bool {
        self.last_updated.elapsed().as_secs() < LEADERBOARD_UPDATE_SEC
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
    events: Arc<RwLock<Events>>,
    year: EventYear,
    _timestamp: Option<i64>,
) -> BinaryHeap<Member> {
    // TODO: handle LockResult errors
    if let Some(event) = events.read().unwrap().get_event(year) {
        if event.is_up_to_date() {
            return event.members.clone();
        }
    }

    // TODO: handle LockResult errors
    if let Some(event) = events.write().unwrap().update_event(year) {
        event.members.clone()
    } else {
        BinaryHeap::new()
    }
}

