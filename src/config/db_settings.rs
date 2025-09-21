use serde::Deserialize;
use config::{Config, ConfigError, Environment};

use crate::config::traits::Env;

#[derive(Debug, Clone, Deserialize)]
pub struct DbSettings {
    pub database_url: String
}

impl Env for DbSettings {
    fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok(); // Loading .env file

        let settings = Config::builder()
            .add_source(Environment::default())
            .build()?;

        settings.try_deserialize()
    }
}