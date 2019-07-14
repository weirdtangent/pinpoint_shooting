use rocket::{get, response::NamedFile};
use rocket_contrib::templates::Template;
use serde_json::json;
use std::path::Path;

use crate::settings::CONFIG;

#[get("/")]
pub fn index() -> rocket_contrib::templates::Template {
    let api_key = &CONFIG.google_maps_api_key;

    let context = json!({
        "title": "Dashboard",
        "google_maps_api_key": api_key,
    });

    Template::render("index", &context)
}

#[get("/favicon.ico")]
pub fn favicon() -> Option<NamedFile> {
    NamedFile::open(Path::new("src/view/static/favicon.ico")).ok()
}
