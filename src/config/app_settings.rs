use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
}

impl Settings {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        dotenvy::dotenv().ok(); // Load .env file if it exists
        
        let settings = config::Config::builder()
            .add_source(config::Environment::default())
            .build()?;
            
        settings.try_deserialize()
    }
}