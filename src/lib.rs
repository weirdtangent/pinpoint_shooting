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
extern crate slog;
extern crate slog_bunyan;
extern crate sloggers;

pub mod crypt;
pub mod logging;
pub mod models;
pub mod routes;
pub mod schema;
pub mod settings;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use slog::*;
use sloggers::file::*;
use sloggers::types::Severity;
use sloggers::Build;

use rocket::routes;
use rocket_slog::SlogFairing;

use rocket_contrib::helmet::SpaceHelmet;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

use logging::LOGGING;
use settings::CONFIG;

pub fn setup_logging() {
    let logger = &LOGGING.logger;

    let run_level = &CONFIG.server.run_level;
    warn!(logger, "Service starting"; "run_level" => run_level);
}

pub fn setup_db() -> PgConnection {
    PgConnection::establish(&CONFIG.database_url).expect(&format!("Error connecting to db"))
}

pub fn start_webservice() {
    let logger = &LOGGING.logger;

    let weblog_path = &CONFIG.webservice.weblog_path;
    let bind_address = &CONFIG.webservice.bind_address;
    let bind_port = &CONFIG.webservice.bind_port;

    // start rocket webservice
    let version = include_str!("version.txt").trim_end_matches("\n");

    let mut builder = FileLoggerBuilder::new(weblog_path);
    builder.level(Severity::Debug);
    let weblogger = builder.build().unwrap();
    let fairing = SlogFairing::new(weblogger);

    warn!(
        logger,
        "Waiting for connections..."; "address" => bind_address, "port" => bind_port, "version" => version);
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
