use config::{Config, Environment, File};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Server {
    pub run_level: String,
}

#[derive(Debug, Deserialize)]
pub struct Sessions {
    pub expire: i64,
}

#[derive(Debug, Deserialize)]
pub struct WebService {
    pub bind_address: String,
    pub bind_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub applog_path: String,
    pub weblog_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub webservice: WebService,
    pub sessions: Sessions,
    pub database_url: String,
    pub google_maps_api_key: String,
    pub logconfig: LogConfig,
}

pub static CONFIG: Lazy<Settings> = Lazy::new(|| {
    dotenv().ok();
    let mut config = Config::default();
    let run_level = env::var("PPS_RUN_MODE").unwrap_or("development".into());
    println!("Reading conf/{}.toml", run_level);

    config
        .merge(File::with_name("conf/default"))
        .unwrap()
        .merge(File::with_name(&format!("conf/{}", run_level)).required(false))
        .unwrap()
        .merge(File::with_name("conf/local").required(false))
        .unwrap()
        .merge(Environment::with_prefix("PPS"))
        .unwrap()
        .merge(Environment::new())
        .unwrap();
    match config.try_into() {
        Ok(c) => c,
        Err(e) => panic!("error parsing config files: {}", e),
    }
});
