use rocket::{http::Cookies, post, request::Form, FromForm};

use crate::routes::Nginx;
use crate::session::*;
use crate::settings::CONFIG;
use crate::*;

#[derive(FromForm)]
pub struct OAuthToken {
    pub g_token: Option<String>,
    pub fb_token: Option<String>,
}

#[post("/api/v1/tokensignin", data = "<oauth_token>")]
pub fn tokensignin(mut cookies: Cookies, nginx: Nginx, oauth_token: Form<OAuthToken>) -> String {
    let dynamodb = connect_dynamodb();
    let mut session = get_or_setup_session(&mut cookies, &nginx);

    if let Some(g) = &oauth_token.g_token {
        let mut google = google_signin::Client::new();
        google.audiences.push(CONFIG.google_api_client_id.clone());

        let id_info = google.verify(&g).expect("Expected token to be valid");

        match session.verify_or_save_google_sub(&id_info.sub) {
            true => {
                save_session_to_ddb(&dynamodb, &mut session);
                "success".to_string()
            }
            false => {
                delete_session_in_ddb(&dynamodb, &session.sessid);
                "mismatch".to_string()
            }
        }
    } else if let Some(fb) = &oauth_token.fb_token {
        match session.verify_or_save_fb_sub(&fb) {
            true => {
                save_session_to_ddb(&dynamodb, &mut session);
                "success".to_string()
            }
            false => {
                delete_session_in_ddb(&dynamodb, &session.sessid);
                "mismatch".to_string()
            }
        }
    } else {
        "no token sent".to_string()
    }
}
