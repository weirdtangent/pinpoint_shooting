#![feature(proc_macro_hygiene, decl_macro)]

extern crate crypto;
extern crate rand;
extern crate config;
#[macro_use]
extern crate diesel;
extern crate chrono;
extern crate dotenv;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

//#[path = "models.rs"]
pub mod models;
//#[path = "routes.rs"]
pub mod routes;
//#[path = "schema.rs"]
pub mod schema;
//#[path = "settings.rs"]
pub mod settings;
pub mod crypt;

use log::warn;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::env;

use rocket::routes;

use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

use settings::CONFIG;

pub fn setup_logging() {
    let config = CONFIG.lock().unwrap();

    simple_logger::init_with_level(log::Level::Info)
        .expect("Tried to start the logging system but it had already started");
    let run_level = &config.server.run_level;
    warn!("Running as run_level {}", run_level);
}

pub fn setup_db() -> PgConnection {
    let config = CONFIG.lock().unwrap();

    PgConnection::establish(&config.database_url).expect(&format!("Error connecting to db"))
}

pub fn start_webservice() {
    let config = CONFIG.lock().unwrap();

    // start rocket webservice
    let bind_address = &config.webservice.bind_address;
    let bind_port = &config.webservice.bind_port;
    let version = include_str!("version.txt");

    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![routes::index])
        .mount("/img", StaticFiles::from("src/view/static/img"))
        .mount("/css", StaticFiles::from("src/view/static/css"))
        .mount("/js", StaticFiles::from("src/view/static/js"))
        .launch();

    warn!(
        "Listening on {}:{} as version {}",
        bind_address, bind_port, version
    );
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
