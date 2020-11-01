#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate log;
extern crate config;

mod aoc;
mod leaders;
mod requests;

use config::{Config, ConfigError, File};
use leaders::EventManager;
use std::convert::TryInto;
use std::process::exit;
use std::sync::{Arc, RwLock};

fn build_event_manager() -> Result<EventManager, ConfigError> {
    let mut settings = Config::default();

    // Set default values
    settings.set_default("leaderboard_update_sec", 5 * 60)?;

    // Load settings from file
    settings.merge(File::with_name("settings"))?;

    let leaderboard_update_sec = settings
        .get_int("leaderboard_update_sec")?
        .try_into()
        .map_err(|_| {
            ConfigError::Message(
                "leaderboard_update_sec must not be negative".to_string(),
            )
        })?;
    let session_cookie = settings.get_str("session_cookie")?;
    let leaderboard_ids = settings
        .get_array("leaderboard_ids")?
        .into_iter()
        .map(|v| v.into_str())
        .collect::<Result<Vec<_>, _>>()?;

    info!("leaderboard_ids = {:?}", leaderboard_ids);
    info!("leaderboard_update_sec = {}", leaderboard_update_sec);

    Ok(EventManager::new(
        leaderboard_ids,
        session_cookie,
        leaderboard_update_sec,
    ))
}

fn main() {
    env_logger::init();

    info!("Loading settings");
    let event_manager = build_event_manager().unwrap_or_else(|err| {
        error!("Failed to load settings: {}", err.to_string());
        exit(1);
    });

    rocket::ignite()
        .manage(Arc::new(RwLock::new(event_manager)))
        .mount("/", routes![requests::index, requests::event_year])
        .launch();
}
