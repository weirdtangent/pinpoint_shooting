use rocket::{http::Cookies, post, request::Form, FromForm};

use crate::oauth::*;
use crate::routes::Nginx;
use crate::session::*;

#[derive(FromForm)]
pub struct OAuthReq {
    pub g_token: Option<String>,  // google login req
    pub fb_token: Option<String>, // facebook login req`
    pub name: String,
    pub email: String,
}

#[post("/api/v1/tokensignin", data = "<oauth_req>")]
pub fn tokensignin(mut cookies: Cookies, nginx: Nginx, oauth_req: Form<OAuthReq>) -> String {
    let mut session = get_or_setup_session(&mut cookies, &nginx);

    if let Some(token) = &oauth_req.g_token {
        match verify_google_oauth(&mut session, &token, &oauth_req.name, &oauth_req.email) {
            true => {
                session.google_oauth = true;
                save_session_to_ddb(&mut session);
                "success".to_string()
            }
            false => {
                session.google_oauth = false;
                save_session_to_ddb(&mut session);
                "failed".to_string()
            }
        }
    } else if let Some(token) = &oauth_req.fb_token {
        match verify_facebook_oauth(&mut session, &token, &oauth_req.name, &oauth_req.email) {
            true => {
                session.facebook_oauth = true;
                save_session_to_ddb(&mut session);
                "success".to_string()
            }
            false => {
                session.facebook_oauth = false;
                save_session_to_ddb(&mut session);
                "failed".to_string()
            }
        }
    } else {
        "no token sent".to_string()
    }
}
