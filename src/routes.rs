use rocket::{
    get,
    http::Cookies,
    request::{self, FromRequest, Request},
    response::NamedFile,
    Outcome,
};
use rocket_contrib::templates::Template;
use serde_json::json;
use std::path::Path;

use crate::session::*;
use crate::settings::CONFIG;

#[derive(Default)]
pub struct Nginx {
    pub x_user_agent: String,
    pub x_real_ip: String,
}

impl<'a, 'r> FromRequest<'a, 'r> for Nginx {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        Outcome::Success(Nginx {
            x_user_agent: request
                .headers()
                .get_one("X-User-Agent")
                .unwrap_or("")
                .to_string(),
            x_real_ip: request
                .headers()
                .get_one("X-Real-IP")
                .unwrap_or("")
                .to_string(),
        })
    }
}

#[get("/")]
pub fn index(mut cookies: Cookies, nginx: Nginx) -> rocket_contrib::templates::Template {
    let session = get_or_setup_session(&mut cookies, &nginx);

    let api_key = &CONFIG.google_maps_api_key;

    let context = json!({
        "title": "Dashboard",
        "google_maps_api_key": api_key,
        "user_name": session.user_name,
        "x_user_agent": nginx.x_user_agent,
        "x_real_ip": nginx.x_real_ip,
    });

    Template::render("index", &context)
}

#[get("/favicon.ico")]
pub fn favicon() -> Option<NamedFile> {
    NamedFile::open(Path::new("src/view/static/favicon.ico")).ok()
}
