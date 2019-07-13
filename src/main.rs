extern crate config;
extern crate handlebars_iron as hbs;
extern crate iron;
extern crate serde;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

mod settings;
use settings::Settings;

use std::error::Error;

use log::{debug, info};

use iron::prelude::{Chain, Iron, Request, Response};
use iron::{status, Set};

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
    // setup Handlebars templating directory
    let mut hbse = hbs::HandlebarsEngine::new();
    hbse.add(Box::new(hbs::DirectorySource::new("./src/view/", ".hbs")));
    if let Err(r) = hbse.reload() {
        panic!("{:?}", r.description());
    }

    //
    let mut chain = Chain::new(|_: &mut Request| {
        let mut resp = Response::new();
        resp.set_mut(hbs::Template::new("index", "".to_string()))
            .set_mut(status::Ok);
        Ok(resp)
    });
    chain.link_after(hbse);

    // start iron webservice
    let bind_address = &SETTINGS.webservice.bind_address;
    let bind_port = &SETTINGS.webservice.bind_port;
    let _server_guard = Iron::new(chain)
        .http(format!("{}:{}", bind_address, bind_port))
        .unwrap();
    let version = include_str!("version.txt");

    info!(
        "Listening on {}:{} as version {}",
        bind_address, bind_port, version
    );
}
