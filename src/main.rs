#![feature(proc_macro_hygiene, decl_macro)]

extern crate config;
extern crate diesel;
extern crate dotenv;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

mod settings;
use settings::CONFIG;
mod lib;
mod routes;

fn main() {
    println!("{:?}", CONFIG.lock().unwrap());

    lib::setup_logging();
    lib::setup_db();
    lib::start_webservice();
}
