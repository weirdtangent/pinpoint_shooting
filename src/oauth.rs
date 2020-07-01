use diesel::prelude::*;

use crate::session::*;
use crate::settings::CONFIG;
use crate::shooter::*;
use crate::*;

use crate::model::{NewOauth, Oauth, Shooter};

pub fn create_oauth(
    connection: &PgConnection,
    vendor: &str,
    user_id: &str,
    shooterid: i32,
) -> Oauth {
    use crate::schema::oauth::dsl::*;

    let new_oauth = NewOauth {
        oauth_vendor: vendor.to_string(),
        oauth_user: user_id.to_string(),
        shooter_id: shooterid,
    };

    diesel::insert_into(oauth)
        .values(&new_oauth)
        .get_result(connection)
        .expect("Error saving new Oauth")
}

pub fn verify_google_oauth(session: &mut Session, token: &str, name: &str, email: &str) -> bool {
    let mut google = google_signin::Client::new();
    google.audiences.push(CONFIG.google_api_client_id.clone());

    let id_info = google.verify(&token).expect("Expected token to be valid");
    let token = id_info.sub;

    verify_token(session, "google".to_string(), &token, name, email)
}

pub fn verify_facebook_oauth(session: &mut Session, token: &str, name: &str, email: &str) -> bool {
    verify_token(session, "facebook".to_string(), &token, name, email)
}

fn verify_token(
    session: &mut Session,
    vendor: String,
    token: &str,
    name: &str,
    email: &str,
) -> bool {
    use crate::schema::oauth::dsl::*;
    use crate::schema::shooter::dsl::*;
    let connection = connect_pgsql();
    match oauth
        .filter(oauth_vendor.eq(&vendor))
        .filter(oauth_user.eq(&token))
        .first::<Oauth>(&connection)
    {
        // token WAS found in oauth table
        Ok(o) => {
            if let Some(id) = session.shooter_id {
                id == o.shooter_id
            } else {
                // log in user
                //if let Ok(s) = Shooter::belonging_to(&o).load::<Shooter>(&connection) {
                //    session.shooter_id = Some(shooter.shooter_id);
                //    session.shooter_name = Some(shooter.shooter_name);
                //    session.email_address = Some(shooter.email);
                true
                //} else {
                //    return false;
                //}
            }
        }
        // token not found in oauth table
        Err(diesel::NotFound) => match session.shooter_id {
            Some(id) => {
                create_oauth(&connection, &vendor, token, id);
                true
            }
            None => match shooter
                .filter(shooter_email.eq(&email))
                .first::<Shooter>(&connection)
            {
                // email address WAS found in shooter table
                Ok(s) => {
                    create_oauth(&connection, &vendor, token, s.shooter_id);
                    true
                }
                // email address not found in shooter table
                Err(diesel::NotFound) => {
                    let this_shooter = create_shooter(
                        &connection,
                        &String::from(name),
                        None,
                        &String::from(email),
                        &"active".to_string(),
                    );
                    session.shooter_id = Some(this_shooter.shooter_id);
                    create_oauth(&connection, &vendor, token, this_shooter.shooter_id);
                    true
                }
                Err(e) => {
                    panic!("Database error {}", e);
                }
            },
        },
        Err(e) => {
            panic!("Database error {}", e);
        }
    }
}
