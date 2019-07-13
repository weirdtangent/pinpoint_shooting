#![feature(proc_macro_hygiene, decl_macro)]

extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;

mod settings;
use settings::Settings;

use std::error::Error;

use log::{debug, info};

// Load config file(s) into global static SETTINGS
lazy_static! {
    static ref SETTINGS: settings::Settings = { Settings::new().unwrap() };
}

fn main() {
    setup_logging();
    start_webservice();
}

fn setup_logging() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let run_level = &SETTINGS.server.run_level;
    info!("Running as run_level {}", run_level);
}

fn start_webservice() {
    // start rocket webservice
    let bind_address = &SETTINGS.webservice.bind_address;
    let bind_port = &SETTINGS.webservice.bind_port;
    let version = include_str!("version.txt");

    rocket::ignite().mount("/", routes![index]).launch();

    info!(
        "Listening on {}:{} as version {}",
        bind_address, bind_port, version
    );
}


#[get("/")]
fn index() -> &'static str {
    "Hello, World!"
}
