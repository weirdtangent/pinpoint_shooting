#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
extern crate chrono;
extern crate config;
extern crate crypto;
extern crate dotenv;
extern crate rand;
extern crate rocket;
extern crate rocket_contrib;
extern crate rocket_slog;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate slog;
extern crate slog_bunyan;

pub mod crypt;
pub mod logging;
pub mod models;
pub mod routes;
pub mod schema;
pub mod settings;

use std::fs::OpenOptions;
use std::sync::Mutex;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use slog::*;

use rocket::routes;
use rocket_slog::SlogFairing;

use rocket_contrib::helmet::SpaceHelmet;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

use logging::LOGGING;
use settings::CONFIG;

pub fn setup_logging() {
    let applogger = &LOGGING.logger;

    let run_level = &CONFIG.server.run_level;
    warn!(applogger, "Service starting"; "run_level" => run_level);
}

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
    rocket::ignite()
        .attach(fairing)
        .attach(Template::fairing())
        .attach(SpaceHelmet::default())
        .mount("/", routes![routes::index, routes::favicon])
        .mount("/img", StaticFiles::from("src/view/static/img"))
        .mount("/css", StaticFiles::from("src/view/static/css"))
        .mount("/js", StaticFiles::from("src/view/static/js"))
        .launch();
}

use self::models::{NewUser, User};

pub fn create_user<'a>(
    connection: &PgConnection,
    user_name: &'a str,
    password: &'a str,
    email: &'a str,
    real_name: &'a str,
) -> User {
    use schema::users;

    let new_user = NewUser {
        user_name: user_name,
        password: password,
        email: email,
        real_name: real_name,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(connection)
        .expect("Error saving new user")
}
