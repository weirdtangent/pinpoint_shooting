#![feature(proc_macro_hygiene, decl_macro)]

extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

mod settings;
use settings::Settings;

use log::{debug, info, warn};

use rocket_contrib::templates::Template;
use rocket_contrib::serve::StaticFiles;

// Load config file(s) into global static SETTINGS
lazy_static! {
    static ref SETTINGS: settings::Settings = { Settings::new().unwrap() };
}

fn main() {
    setup_logging();
    start_webservice();
}

fn start_webservice() {
    // start rocket webservice
    let bind_address = &SETTINGS.webservice.bind_address;
    let bind_port = &SETTINGS.webservice.bind_port;
    let version = include_str!("version.txt");

    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index])
        .mount("/img", StaticFiles::from("src/view/static/img"))
        .mount("/css", StaticFiles::from("src/view/static/css"))
        .mount("/js", StaticFiles::from("src/view/static/js"))
        .launch();

    warn!(
        "Listening on {}:{} as version {}",
        bind_address, bind_port, version
    );
}

fn setup_logging() {
    simple_logger::init_with_level(log::Level::Info)
        .expect("Tried to start the logging system but it had already started");
    let run_level = &SETTINGS.server.run_level;
    warn!("Running as run_level {}", run_level);
}

#[get("/")]
fn index() -> rocket_contrib::templates::Template {
    let context = json!({"title": "Greeting", "greeting": "Welcome to templates"});
    Template::render("index", &context)
}
