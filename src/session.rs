use chrono::prelude::*;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rocket::http::{Cookie, Cookies};
use rusoto_dynamodb::{AttributeValue, DeleteItemInput, DynamoDb, GetItemInput, PutItemInput};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use time::Duration;

use crate::routes::Nginx;
use crate::*;

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Session {
    pub sessid: String,
    pub user_agent: String,
    pub shooter_id: Option<i32>,
    pub shooter_name: Option<String>,
    pub email_address: Option<String>,
    pub last_access: Option<DateTime<Utc>>,
    pub google_oauth: bool,
    pub facebook_oauth: bool,
}

// Check for sessid cookie and verify session or create new
// session to use - either way, return the session struct
pub fn get_or_setup_session(cookies: &mut Cookies, nginx: &Nginx) -> Session {
    // if we can pull sessid from cookies and validate it,
    // pull session from from dynamodb and return
    if let Some(cookie) = cookies.get_private("sessid") {
        if let Some(mut session) = verify_session_in_ddb(&cookie.value().to_string(), &nginx) {
            save_session_to_ddb(&mut session);
            return session;
        }
    }

    // otherwise, we'll need to start a new session
    let session = create_new_session(&nginx);
    let sess_cookie = Cookie::build("sessid", session.sessid.clone())
        .path("/")
        .secure(true)
        .finish();
    cookies.add_private(sess_cookie);

    session
}

// prepare new session with defaults, save and return
pub fn create_new_session(nginx: &Nginx) -> Session {
    let sessid = get_new_session_id(&nginx);

    let mut session = Session {
        sessid: sessid.clone(),
        user_agent: nginx.x_user_agent.clone(),
        ..Default::default()
    };

    save_session_to_ddb(&mut session);
    session
}

// generate new session id from user_agent and random
// characters, and make sure it is unique
fn get_new_session_id(nginx: &Nginx) -> String {
    let mut sessid = String::new();
    while sessid.is_empty() {
        let mut hasher = Sha256::new();
        let randstr: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(256)
            .collect::<String>();
        hasher.input(nginx.x_user_agent.clone() + &randstr);
        sessid = format!("{:x}", hasher.result());

        // if we find this new sessid in use already,
        // blank it out so we can try again
        if let Some(_) = verify_session_in_ddb(&sessid, &nginx) {
            sessid = "".to_string();
        }
    }
    sessid
}

// Search for sessid in dynamodb and verify session if found
// including to see if it has expired
fn verify_session_in_ddb(sessid: &String, nginx: &Nginx) -> Option<Session> {
    let dynamodb = connect_dynamodb();
    let applogger = &LOGGING.logger;

    // sessid must be exactly 64 ascii bytes of [0-9a-f] (32 bytes expressed as hex)
    if sessid.len() != 64 || !sessid.is_ascii() || !sessid.chars().all(|c| c.is_ascii_hexdigit()) {
        warn!(applogger, "sessid provided is wrong length or format");
        return None;
    }

    let attr_value = AttributeValue {
        s: Some(sessid.clone()),
        ..Default::default()
    };

    let mut key = HashMap::new();
    key.insert("sessid".to_string(), attr_value);

    let get_item_input = GetItemInput {
        table_name: "session".to_string(),
        key: key,
        ..Default::default()
    };

    // attempt to pull session from dynamodb using sess_id
    // and verify expire date and user_agent
    match dynamodb.get_item(get_item_input).sync() {
        Ok(item_output) => {
            let ddb_item = item_output.item?;
            let attr_value = ddb_item.get("session")?;
            let session_str = &attr_value.s.clone()?;
            let session: Session = serde_json::from_str(&session_str).unwrap();
            match (session.last_access, &session.user_agent) {
                (Some(l), ua) => {
                    if l > Utc::now() - Duration::minutes(CONFIG.sessions.expire)
                        && *ua == nginx.x_user_agent
                    {
                        debug!(applogger, "Session verified"; "sessid" => sessid);
                        Some(session)
                    } else {
                        debug!(applogger, "Session expired or user_agent does not match"; "sessid" => sessid);
                        delete_session_in_ddb(&session);
                        None
                    }
                }
                (None, _) => {
                    debug!(applogger, "Session failure: 'last_access' is blank for stored session"; "sessid" => sessid);
                    delete_session_in_ddb(&session);
                    None
                }
            }
        }
        Err(e) => {
            crit!(applogger, "Error in dynamodb"; "err" => e.to_string());
            None
        }
    }
}

// Store current session in dynamodb,
// including updating the last-access date/time
pub fn save_session_to_ddb(session: &mut Session) {
    let dynamodb = connect_dynamodb();
    let applogger = &LOGGING.logger;

    session.last_access = Some(Utc::now());

    let sessid_av = AttributeValue {
        s: Some(session.sessid.clone()),
        ..Default::default()
    };
    let session_av = AttributeValue {
        s: Some(serde_json::to_string(&session).unwrap()),
        ..Default::default()
    };
    let mut item = HashMap::new();
    item.insert("sessid".to_string(), sessid_av);
    item.insert("session".to_string(), session_av);

    let put_item_input = PutItemInput {
        table_name: "session".to_string(),
        item: item,
        ..Default::default()
    };

    match dynamodb.put_item(put_item_input).sync() {
        Ok(_) => {}
        Err(e) => {
            crit!(applogger, "Error in dynamodb"; "err" => e.to_string());
            panic!("Error in dynamodb: {}", e.to_string());
        }
    };
}

pub fn delete_session_in_ddb(session: &Session) {
    let applogger = &LOGGING.logger;
    let dynamodb = connect_dynamodb();

    let av = AttributeValue {
        s: Some(session.sessid.clone()),
        ..Default::default()
    };

    let mut key = HashMap::new();
    key.insert("sessid".to_string(), av);

    let delete_item_input = DeleteItemInput {
        table_name: "session".to_string(),
        key: key,
        ..Default::default()
    };

    match dynamodb.delete_item(delete_item_input).sync() {
        Ok(_) => {
            debug!(applogger, "Deleted invalid session from ddb"; "sessid" => &session.sessid);
        }
        Err(e) => {
            crit!(applogger, "Error in dynamodb"; "err" => e.to_string());
            panic!("Error in dynamodb: {}", e.to_string());
        }
    };
}
