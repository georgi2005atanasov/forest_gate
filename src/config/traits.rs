use config::ConfigError;

pub trait Env: Sized {
    fn from_env() -> Result<Self, ConfigError>;
}