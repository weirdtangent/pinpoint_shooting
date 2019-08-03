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
    println!("Service starting at {} runlevel", run_level);
    println!(
        "Setting up to listen on {}:{}",
        &CONFIG.webservice.bind_address, &CONFIG.webservice.bind_port
    );
    println!("Application logging to {}", &CONFIG.logconfig.applog_path);
    println!(
        "Rocket framework logging to {}",
        &CONFIG.logconfig.weblog_path
    );

    start_application();
}
