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
    pub user_id: Option<u32>,
    pub user_name: Option<String>,
    pub last_access: Option<DateTime<Utc>>,
}

// Check for sessid cookie and verify session or create new
// session to use - either way, return the session struct
pub fn get_or_setup_session(cookies: &mut Cookies, nginx: &Nginx) -> Session {
    let applogger = &LOGGING.logger;
    let dynamodb = connect_dynamodb();

    // if we can pull sessid from cookies and validate it,
    // pull session from cache or from storage and return
    if let Some(cookie) = cookies.get_private("sessid") {
        debug!(applogger, "Cookie found, verifying"; "sessid" => cookie.value());

        // verify from dynamodb, update session with last-access if good
        if let Some(mut session) =
            verify_session_in_ddb(&dynamodb, &cookie.value().to_string(), &nginx)
        {
            save_session_to_ddb(&dynamodb, &mut session, &nginx);
            return session;
        }
    }

    let sessid = get_new_session_id(&dynamodb, &nginx);
    let sess_cookie = Cookie::build("sessid", sessid.clone())
        .path("/")
        .secure(true)
        .finish();
    cookies.add_private(sess_cookie);

    let mut session = Session {
        sessid: sessid.clone(),
        ..Default::default()
    };

    save_session_to_ddb(&dynamodb, &mut session, &nginx);
    session
}

// otherwise, start a new, empty session to use for this user
// and make sure the random sessid we pick isn't ALREADY in use
fn get_new_session_id(dynamodb: &DynamoDbClient, nginx: &Nginx) -> String {
    let mut sessid = String::new();
    while sessid.is_empty() {
        let mut hasher = Sha256::new();
        let randstr: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(256)
            .collect::<String>();
        hasher.input(nginx.x_user_agent.clone() + &randstr);
        sessid = format!("{:x}", hasher.result());
        if let Some(_) = verify_session_in_ddb(&dynamodb, &sessid, &nginx) {
            sessid = "".to_string();
        }
    }
    sessid
}

// Search for sessid in dynamodb and verify session if found
// including to see if it has expired
fn verify_session_in_ddb(
    dynamodb: &DynamoDbClient,
    sessid: &String,
    nginx: &Nginx,
) -> Option<Session> {
    let applogger = &LOGGING.logger;

    // sessid must be exactly 64 ascii bytes of [0-9a-f] (32 bytes expressed as hex)
    if sessid.len() != 64 || !sessid.is_ascii() {
        warn!(applogger, "sessid wrong length"; "length" => sessid.len());
        return None;
    }
    if !sessid.chars().all(|c| c.is_ascii_hexdigit()) {
        warn!(applogger, "sessid not only ascii hexdigits"; "sessid" => sessid);
        return None;
    }

    let av = AttributeValue {
        s: Some(sessid.clone()),
        ..Default::default()
    };

    let mut key = HashMap::new();
    key.insert("sessid".to_string(), av);

    let get_item_input = GetItemInput {
        table_name: "session".to_string(),
        key: key,
        ..Default::default()
    };

    match dynamodb.get_item(get_item_input).sync() {
        Ok(item_output) => match item_output.item {
            Some(item) => match item.get("session") {
                Some(session) => match &session.s {
                    Some(string) => {
                        let session: Session = serde_json::from_str(&string).unwrap();
                        match (session.last_access, &session.user_agent) {
                            (Some(last), ua) => {
                                if last > Utc::now() - Duration::minutes(CONFIG.sessions.expire)
                                    && *ua == nginx.x_user_agent
                                {
                                    debug!(applogger, "Session verified"; "sessid" => sessid);
                                    Some(session)
                                } else {
                                    debug!(applogger, "Session expired or user_agent does not match"; "sessid" => sessid);
                                    delete_session_in_ddb(dynamodb, sessid);
                                    None
                                }
                            }
                            (None, _) => {
                                debug!(applogger, "'last_access' is blank for stored session"; "sessid" => sessid);
                                delete_session_in_ddb(dynamodb, sessid);
                                None
                            }
                        }
                    }
                    None => {
                        debug!(applogger, "'session' attribute is empty for stored session"; "sessid" => sessid);
                        delete_session_in_ddb(dynamodb, sessid);
                        None
                    }
                },
                None => {
                    debug!(applogger, "No 'session' attribute found for stored session"; "sessid" => sessid);
                    delete_session_in_ddb(dynamodb, sessid);
                    None
                }
            },
            None => {
                debug!(applogger, "Session not found in dynamodb"; "sessid" => sessid);
                None
            }
        },
        Err(e) => {
            crit!(applogger, "Error in dynamodb"; "err" => e.to_string());
            panic!("Error in dynamodb: {}", e.to_string());
        }
    }
}

// Write current session to dynamodb, update last-access date/time too
fn save_session_to_ddb(dynamodb: &DynamoDbClient, session: &mut Session, nginx: &Nginx) {
    let applogger = &LOGGING.logger;

    session.last_access = Some(Utc::now());
    session.user_agent = nginx.x_user_agent.clone();

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

// Delete session from dynamodb
fn delete_session_in_ddb(dynamodb: &DynamoDbClient, sessid: &String) {
    let applogger = &LOGGING.logger;

    let av = AttributeValue {
        s: Some(sessid.clone()),
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
            debug!(applogger, "Deleted invalid session from ddb"; "sessid" => sessid);
        }
        Err(e) => {
            crit!(applogger, "Error in dynamodb"; "err" => e.to_string());
            panic!("Error in dynamodb: {}", e.to_string());
        }
    };
}
