#![feature(proc_macro_hygiene, decl_macro)]

#[path = "routes.rs"]
mod routes;
#[path = "settings.rs"]
mod settings;

use log::warn;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
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
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("env DATABASE_URL missing");
    PgConnection::establish(&db_url).expect(&format!("Error connecting to db"))
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
