use config::{Config, Environment, File};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use serde_derive::Deserialize;
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
    pub database_url: String,
    pub google_maps_api_key: String,
}

pub static CONFIG: Lazy<Settings> = Lazy::new(|| {
    dotenv().ok();
    let mut config = Config::default();
    let env = env::var("PPS_RUN_MODE").unwrap_or("development".into());

    config
        .merge(File::with_name("conf/default"))
        .unwrap()
        .merge(File::with_name(&format!("conf/{}", env)).required(false))
        .unwrap()
        .merge(File::with_name("conf/local").required(false))
        .unwrap()
        .merge(Environment::with_prefix("PPS"))
        .unwrap();
    match config.try_into() {
        Ok(c) => c,
        Err(e) => panic!("error parsing config files: {}", e),
    }
});
