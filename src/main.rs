#![feature(proc_macro_hygiene, decl_macro)]

mod aoc;
mod app;
mod leaders;
mod routes;

use app::AppSettings;
use env_logger::Builder;
use leaders::EventManager;
use log::{error, info, LevelFilter};
use rocket::routes;
use rocket_contrib::templates::Template;
use std::process::exit;
use std::sync::{Arc, RwLock};

const SETTINGS_FILE: &str = "settings";

fn main() {
    // TODO: currently Rocket doesn't provide a nice a nice way to write
    // app logs so we take over with env_logger - revisit this once issue
    // https://github.com/SergioBenitez/Rocket/issues/21 is resolved
    Builder::new()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let settings =
        AppSettings::load_from_file(SETTINGS_FILE).unwrap_or_else(|err| {
            error!("Failed to load settings: {}", err.to_string());
            exit(1);
        });

    info!("leaderboard_ids = {:?}", settings.leaderboard_ids);
    info!(
        "leaderboard_default_order = {}",
        serde_json::to_string(&settings.leaderboard_default_order).unwrap()
    );
    info!(
        "leaderboard_update_sec = {}",
        settings.leaderboard_update_sec
    );
    info!("exclude_members = {:?}", settings.exclude_members);
    if let Some(year) = settings.latest_event_year {
        info!("latest_event_year = {}", year);
    };

    let event_mgr = EventManager::new(
        settings.leaderboard_ids.clone(),
        settings.session_cookie.clone(),
        settings.leaderboard_update_sec,
        settings.exclude_members.iter().cloned().collect(),
    );

    rocket::ignite()
        .manage(Arc::new(settings))
        .manage(Arc::new(RwLock::new(event_mgr)))
        .mount(
            "/",
            routes![
                routes::leaderboard,
                routes::leaderboard_year,
                routes::events,
                routes::events_year
            ],
        )
        .attach(Template::fairing())
        .launch();
}
