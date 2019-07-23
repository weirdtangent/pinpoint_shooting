#![feature(proc_macro_hygiene, decl_macro)]

extern crate ppslib;

use ppslib::*;
use slog::*;

use logging::LOGGING;
use settings::CONFIG;

fn main() {
    let applogger = &LOGGING.logger;
    let run_level = &CONFIG.server.run_level;
    warn!(applogger, "Service starting"; "run_level" => run_level);

    setup_db();
    start_webservice();
}
