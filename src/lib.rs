#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;

extern crate config;
extern crate crypto;
extern crate rand;
extern crate chrono;
extern crate dotenv;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

pub mod crypt;
pub mod models;
pub mod routes;
pub mod schema;
pub mod settings;

use log::warn;

use diesel::pg::PgConnection;
use diesel::prelude::*;

use rocket::routes;

use rocket_contrib::helmet::SpaceHelmet;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

use settings::CONFIG;

pub fn setup_logging() {
    simple_logger::init_with_level(log::Level::Info)
        .expect("Tried to start the logging system but it had already started");

    let run_level = &CONFIG.server.run_level;
    warn!("Running as run_level {}", run_level);
}

pub fn setup_db() -> PgConnection {
    PgConnection::establish(&CONFIG.database_url).expect(&format!("Error connecting to db"))
}

pub fn start_webservice() {
    let bind_address = &CONFIG.webservice.bind_address;
    let bind_port = &CONFIG.webservice.bind_port;

    // start rocket webservice
    let version = include_str!("version.txt");

    warn!(
        "Listening on {}:{} as version {}",
        bind_address, bind_port, version
    );
    rocket::ignite()
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
