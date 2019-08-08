#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;

pub mod api;
pub mod crypt;
pub mod logging;
pub mod model;
pub mod oauth;
pub mod routes;
pub mod schema;
pub mod session;
pub mod settings;
pub mod shooter;

#[cfg(test)]
mod tests;

use std::fs::OpenOptions;
use std::sync::Mutex;

use diesel::prelude::*;
use slog::*;

use rocket::routes;
use rocket_contrib::helmet::SpaceHelmet;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use rocket_slog::SlogFairing;

use rusoto_core::Region;
use rusoto_dynamodb::DynamoDbClient;

use logging::LOGGING;
use settings::CONFIG;

pub fn connect_pgsql() -> PgConnection {
    PgConnection::establish(&CONFIG.database_url).expect(&format!("Error connecting to db"))
}

pub fn connect_dynamodb() -> DynamoDbClient {
    DynamoDbClient::new(Region::UsEast1)
}

pub fn rocket_prep() -> rocket::Rocket {
    let applogger = &LOGGING.logger;

    // start weblogger with json logs
    let logconfig = &CONFIG.logconfig;
    let logfile = &logconfig.weblog_path;
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(logfile)
        .unwrap();

    let weblogger = slog::Logger::root(Mutex::new(slog_bunyan::default(file)).fuse(), o!());

    let fairing = SlogFairing::new(weblogger);

    let bind_address = &CONFIG.webservice.bind_address;
    let bind_port = &CONFIG.webservice.bind_port;
    let version = include_str!("version.txt").trim_end_matches("\n");

    warn!(
        applogger,
        "Waiting for connections..."; "address" => bind_address, "port" => bind_port, "version" => version);

    // init rocket webservice
    rocket::ignite()
        .attach(fairing)
        .attach(Template::fairing())
        .attach(SpaceHelmet::default())
        .mount(
            "/",
            routes![routes::favicon, routes::index, api::tokensignin],
        )
        .mount("/img", StaticFiles::from("src/view/static/img"))
        .mount("/css", StaticFiles::from("src/view/static/css"))
        .mount("/js", StaticFiles::from("src/view/static/js"))
}

pub fn start_application() {
    let err = rocket_prep().launch();

    println!("Error, Rocket failed to init: {}", err);
}
