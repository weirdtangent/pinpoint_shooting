use config::{Config, Environment, File};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use rusoto_core::Region;
use rusoto_secretsmanager::{GetSecretValueRequest, SecretsManager, SecretsManagerClient};
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
    pub logconfig: LogConfig,
    pub google_api_client_id: String,
    pub google_api_client_secret: String,
    pub google_maps_api_key: String,
}

#[derive(Debug, Deserialize)]
struct AWSSecret {
    key: String,
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

    // now pull secrets from AWS
    // note, AWS secrets include PPS_ prefix for this application
    for secret in vec![
        "database_url",
        "google_api_client_id",
        "google_api_client_secret",
        "google_maps_api_key",
        "facebook_app_id",
        "facebook_app_secret",
    ] {
        config
            .set(secret, get_secret(format!("PPS_{}", secret)).key)
            .unwrap();
    }

    // config setup ready
    match config.try_into() {
        Ok(c) => c,
        Err(e) => panic!("error parsing config files: {}", e),
    }
});

fn get_secret(secret: String) -> AWSSecret {
    let secrets_manager = SecretsManagerClient::new(Region::UsEast1);
    match secrets_manager
        .get_secret_value(GetSecretValueRequest {
            secret_id: secret.clone(),
            ..Default::default()
        })
        .sync()
    {
        Ok(resp) => serde_json::from_str(&resp.secret_string.unwrap()).unwrap(),
        Err(err) => panic!("Could not retrieve secret {} from AWS: {:?}", secret, err),
    }
}
