use rocket::{http::Cookies, post, request::Form, FromForm};

use crate::routes::Nginx;
use crate::session::*;
use crate::settings::CONFIG;
use crate::*;

#[derive(FromForm)]
pub struct GoogleToken {
    pub idtoken: String,
}

#[post("/api/v1/tokensignin", data = "<google_token>")]
pub fn tokensignin(mut cookies: Cookies, nginx: Nginx, google_token: Form<GoogleToken>) -> String {
    let dynamodb = connect_dynamodb();
    let mut session = get_or_setup_session(&mut cookies, &nginx);

    let mut google = google_signin::Client::new();
    google.audiences.push(CONFIG.google_api_client_id.clone());

    let id_info = google
        .verify(&google_token.idtoken)
        .expect("Expected token to be valid");

    match session.verify_or_save_google_sub(&id_info.sub) {
        true => {
            save_session_to_ddb(&dynamodb, &mut session);
            id_info.sub
        }
        false => {
            delete_session_in_ddb(&dynamodb, &session.sessid);
            "error".to_string()
        }
    }
}
