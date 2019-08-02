use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rocket::http::{Cookie, Cookies};
use sha2::{Digest, Sha256};

#[derive(Default)]
pub struct Session {
    pub sessid: String,
    pub user_id: Option<u32>,
    pub user_name: Option<String>,
}

pub fn get_or_setup_session(cookies: &mut Cookies) -> Session {
    // if we can pull sessid from cookies and validate it,
    // pull session from cache or from storage and return
    if let Some(cookie) = cookies.get_private("sessid") {
        println!("Cookie: {}", cookie.value());
        Session {
            sessid: cookie.value().to_string(),
            user_id: Some(1),
            user_name: Some(String::from("Jeff")),
        }
    }
    // otherwise, start a new, empty session to use for this user
    else {
        let mut hasher = Sha256::new();
        let randstr: String = thread_rng().sample_iter(&Alphanumeric).take(256).collect();
        hasher.input(randstr);
        let sessid = format!("{:x}", hasher.result());

        cookies.add_private(Cookie::new("sessid", sessid.clone()));
        Session {
            sessid: sessid,
            ..Default::default()
        }
    }
}
