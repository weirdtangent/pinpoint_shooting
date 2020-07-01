use diesel::prelude::*;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

use crate::model::{NewShooter, Shooter};

pub fn create_shooter<'a>(
    connection: &PgConnection,
    name: &'a str,
    password: Option<&'a str>,
    email: &'a str,
    status: &'a str,
) -> Shooter {
    use crate::schema::shooter::dsl::*;

    let new_shooter = NewShooter {
        shooter_name: name.to_string(),
        shooter_password: match password {
            Some(p) => p.to_string(),
            None => thread_rng()
                .sample_iter(&Alphanumeric)
                .take(64)
                .collect::<String>(),
        },
        shooter_status: status.to_string(),
        shooter_email: email.to_string(),
        shooter_real_name: name.to_string(),
    };

    diesel::insert_into(shooter)
        .values(&new_shooter)
        .get_result(connection)
        .expect("Error saving new Shooter")
}
