extern crate diesel;
extern crate ppslib;

use self::ppslib::*;
use std::io::stdin;

fn main() {
    let connection = ppslib::setup_db();

    println!("User name?");
    let mut user_name = String::new();
    stdin().read_line(&mut user_name).unwrap();
    let user_name = &user_name[..(user_name.len() - 1)];

    println!("Password?");
    let mut password = String::new();
    stdin().read_line(&mut password).unwrap();
    let password = &password[..(password.len() - 1)];

    println!("Email address?");
    let mut email = String::new();
    stdin().read_line(&mut email).unwrap();
    let email = &email[..(email.len() - 1)];

    println!("Real name?");
    let mut real_name = String::new();
    stdin().read_line(&mut real_name).unwrap();
    let real_name = &real_name[..(real_name.len() - 1)];

    let user = create_user(&connection, user_name, password, email, real_name);
    println!("Saved new user {} with id {}", user_name, user.user_id);
}
