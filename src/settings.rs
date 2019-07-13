use config::{Config, ConfigError, Environment, File};
use std::env;

#[derive(Debug, Deserialize)]
pub struct Server {
    pub run_level: String,
}

#[derive(Debug, Deserialize)]
pub struct WebService {
    pub bind_address: String,
    pub bind_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub webservice: WebService,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::default();
        let env = env::var("RUN_MODE").unwrap_or("development".into());

        settings
            .merge(File::with_name("conf/default"))
            .unwrap()
            .merge(File::with_name(&format!("conf/{}", env)).required(false))
            .unwrap()
            .merge(File::with_name("conf/local").required(false))
            .unwrap()
            .merge(Environment::with_prefix("app"))
            .unwrap();
        settings.try_into()
    }
}
