//#![feature(proc_macro_hygiene, decl_macro)]

use config::{Config, ConfigError, Environment, File};
use once_cell::sync::Lazy;
use serde_derive::Deserialize;
use std::{sync::Mutex, env};

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

pub static CONFIG: Lazy<Mutex<Settings>> = Lazy::new(|| {
    let mut config = Config::default();
    let env = env::var("RUN_MODE").unwrap_or("development".into());

    config
        .merge(File::with_name("conf/default"))
        .unwrap()
        .merge(File::with_name(&format!("conf/{}", env)).required(false))
        .unwrap()
        .merge(File::with_name("conf/local").required(false))
        .unwrap()
        .merge(Environment::with_prefix("app"))
        .unwrap();
    match config.try_into() {
      Ok(c) => Mutex::new(c),
      Err(e) => panic!("error parsing config files: {}", e),
    }
});
