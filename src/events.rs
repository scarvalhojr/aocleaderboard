use crate::aoc::*;
use crate::AppSettings;
use rocket_contrib::templates::Template;
use serde::Serialize;

pub fn render_events(settings: &AppSettings, year: EventYear) -> Template {
    let latest_year = match settings.latest_event_year {
        Some(y) => y.max(latest_event_year()),
        _ => latest_event_year(),
    };
    let events = (FIRST_EVENT_YEAR..=latest_year).rev().collect::<Vec<_>>();
    let context = Context { year, events };
    Template::render("events", &context)
}

#[derive(Serialize)]
struct Context {
    year: EventYear,
    events: Vec<EventYear>,
}
