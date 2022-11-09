pub use ::config::ConfigError;

use serde::Deserialize;

#[derive(Deserialize)]

// Struct to hold the configuration variables stored in env variables
pub struct Config {
    pub host: String,
    pub port: String,
    pub suggestion_number: u8,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut cfg = ::config::Config::new();
        cfg.merge(::config::Environment::new())?;
        cfg.try_into()
    }
}
