use crate::settings::CONFIG;
use crate::slog::Drain;
use once_cell::sync::Lazy;
use slog::FnValue;
use std::fs::OpenOptions;
use std::sync::Mutex;

#[derive(Debug)]
pub struct Logging {
    pub logger: slog::Logger,
}

pub static LOGGING: Lazy<Logging> = Lazy::new(|| {
    let logconfig = &CONFIG.logconfig;

    let logfile = &logconfig.applog_path;
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(logfile)
        .unwrap();

    let applogger = slog::Logger::root(
        Mutex::new(slog_bunyan::default(file)).fuse(),
        o!("location" => FnValue(move |info| {
        format!("{}:{} {}", info.file(), info.line(), info.module(), )
                })
        ),
    );

    Logging { logger: applogger }
});
