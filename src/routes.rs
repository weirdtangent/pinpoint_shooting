use rocket::{get, http::Cookies, response::NamedFile};
use rocket_contrib::templates::Template;
use serde_json::json;
use std::path::Path;

use crate::session::*;
use crate::settings::CONFIG;

#[get("/")]
pub fn index(mut cookies: Cookies) -> rocket_contrib::templates::Template {
    let session = get_or_setup_session(&mut cookies);

    let api_key = &CONFIG.google_maps_api_key;

    let context = json!({
        "title": "Dashboard",
        "google_maps_api_key": api_key,
        "user_name": session.user_name,
    });

    Template::render("index", &context)
}

#[get("/favicon.ico")]
pub fn favicon() -> Option<NamedFile> {
    NamedFile::open(Path::new("src/view/static/favicon.ico")).ok()
}
