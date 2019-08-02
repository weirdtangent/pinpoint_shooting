#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;

pub mod crypt;
pub mod logging;
pub mod models;
pub mod routes;
pub mod schema;
pub mod session;
pub mod settings;

use std::fs::OpenOptions;
use std::sync::Mutex;

use diesel::prelude::*;
use slog::*;

use rocket::routes;
use rocket_slog::SlogFairing;

use rocket_contrib::helmet::SpaceHelmet;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

use logging::LOGGING;
use settings::CONFIG;

pub fn setup_db() -> PgConnection {
    PgConnection::establish(&CONFIG.database_url).expect(&format!("Error connecting to db"))
}

pub fn start_webservice() {
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

    // start rocket webservice
    let err = rocket::ignite()
        .attach(fairing)
        .attach(Template::fairing())
        .attach(SpaceHelmet::default())
        .mount("/", routes![routes::favicon, routes::index])
        .mount("/img", StaticFiles::from("src/view/static/img"))
        .mount("/css", StaticFiles::from("src/view/static/css"))
        .mount("/js", StaticFiles::from("src/view/static/js"))
        .launch();

    println!("Error, Rocket failed to init: {}", err);
}
