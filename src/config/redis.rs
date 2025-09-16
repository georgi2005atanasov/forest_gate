use deadpool_redis::{Config, Pool, Runtime};

pub fn create_pool(redis_url: &str) -> Result<Pool, deadpool_redis::CreatePoolError> {
    let cfg = Config::from_url(redis_url);
    cfg.create_pool(Some(Runtime::Tokio1))
}