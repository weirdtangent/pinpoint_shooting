#![feature(proc_macro_hygiene, decl_macro)]

extern crate ppslib;

use ppslib::settings::CONFIG;

fn main() {
    println!("{:?}", CONFIG.lock().unwrap());

    ppslib::setup_logging();
    ppslib::setup_db();
    ppslib::start_webservice();
}
