use crate::utils::error::{Error, Result};

use super::{db::ConfigEntity, repo::ConfigRepository, types::ConfigDto};
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::Pool;
use sqlx::PgPool;

#[derive(Clone)]
pub struct ConfigService {
    repo: ConfigRepository,
    redis_pool: Pool,
}

impl ConfigService {
    pub fn new(pool: PgPool, redis_pool: Pool) -> Self {
        Self {
            repo: ConfigRepository::new(pool),
            redis_pool: redis_pool,
        }
    }

    pub async fn get(&self) -> Result<ConfigDto> {
        let mut conn = self.redis_pool.get().await.map_err(Error::from)?;

        // Try Redis
        if let Ok::<String, _>(cached) = conn.get("config").await {
            if let Ok(dto) = serde_json::from_str::<ConfigDto>(&cached) {
                return Ok(dto);
            }
        }

        // Fallback to db
        let entity = self.repo.get_config().await.map_err(Error::from)?;
        Ok(ConfigDto::from(entity))
    }

    pub async fn update(&self, cfg: &ConfigDto) -> Result<()> {
        cfg.validate()?;
        let entity: ConfigEntity = cfg.into();

        self.repo
            .update_config(&entity)
            .await
            .map_err(Error::from)?;

        // update Redis
        let mut conn = self.redis_pool.get().await.map_err(Error::from)?;
        let _: () = conn
            .set("config", serde_json::to_string(cfg)?)
            .await
            .map_err(Error::from)?;

        Ok(())
    }
}
