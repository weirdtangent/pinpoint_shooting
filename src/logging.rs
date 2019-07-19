use crate::settings::CONFIG;
use crate::sloggers::{Build, Config};
use once_cell::sync::Lazy;

#[derive(Debug)]
pub struct Logging {
    pub logger: slog::Logger,
}

pub static LOGGING: Lazy<Logging> = Lazy::new(|| {
    let logconfig = &CONFIG.logconfig;

    let builder = logconfig.try_to_builder().unwrap();
    let logger = builder.build().unwrap();

    Logging { logger: logger }
});
