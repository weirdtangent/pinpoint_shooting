#![feature(proc_macro_hygiene, decl_macro)]

extern crate ppslib;

use ppslib::*;

fn main() {
    setup_logging();
    setup_db();
    start_webservice();
}
