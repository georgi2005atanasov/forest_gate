use serde::Deserialize;
use config::{Config, ConfigError, Environment};

use crate::config::traits::Env;

#[derive(Debug, Clone, Deserialize)]
pub struct RedisSettings {
    pub redis_url: String,
}

impl Env for RedisSettings {
    fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok(); // Load .env file if it exists
        
        let settings = Config::builder()
            .add_source(Environment::default())
            .build()?;
            
        settings.try_deserialize()
    }
}