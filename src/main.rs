#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod aoc;
mod leaders;
mod requests;

use leaders::Events;
use std::sync::{Arc, RwLock};

fn main() {
    rocket::ignite()
        .manage(Arc::new(RwLock::new(Events::new())))
        .mount("/", routes![requests::index, requests::event_year])
        .launch();
}
