#![feature(proc_macro_hygiene, decl_macro)]

mod aoc;
mod app;
mod leaders;
mod requests;

use app::AppSettings;
use leaders::EventManager;
use log::{error, info};
use rocket::routes;
use rocket_contrib::templates::Template;
use std::process::exit;
use std::sync::{Arc, RwLock};

const SETTINGS_FILE: &str = "settings";

fn main() {
    env_logger::init();

    info!("Loading settings");
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

    let event_mgr = EventManager::new(
        settings.leaderboard_ids.clone(),
        settings.session_cookie.clone(),
        settings.leaderboard_update_sec,
    );

    rocket::ignite()
        .manage(Arc::new(settings))
        .manage(Arc::new(RwLock::new(event_mgr)))
        .mount("/", routes![requests::index, requests::event_year])
        .attach(Template::fairing())
        .launch();
}
