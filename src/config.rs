use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub github_key_path: String,
    pub matrix_user: String,
    pub matrix_password: String,
    pub matrix_json_store: String,
    pub database_path: String,
    pub address: String,
    pub port: u16,
}

pub fn get_config() -> Config {
    Figment::new()
        .merge(Toml::file("data/Config.toml"))
        .extract()
        .unwrap()
}
