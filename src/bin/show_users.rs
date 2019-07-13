extern crate ppslib;
extern crate diesel;

use self::ppslib::*;

fn main() {
    use ppslib::schema::users::dsl::*;

    ppslib::setup_db();
}
