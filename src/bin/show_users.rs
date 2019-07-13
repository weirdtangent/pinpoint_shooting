extern crate diesel;
extern crate ppslib;

use self::diesel::prelude::*;
use self::models::*;
use self::ppslib::*;
use ppslib::models::User;

fn main() {
    use ppslib::schema::users::dsl::*;

    let connection = ppslib::setup_db();
    let results = users
        .filter(email.ne(""))
        .limit(5)
        .load::<User>(&connection)
        .expect("Error loading users");

    println!("Displaying {} users", results.len());
    for user in results {
        println!("{} is {}", user.real_name, user.email);
    }
}
