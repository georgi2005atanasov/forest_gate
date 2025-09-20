use sqlx::PgPool;
use super::{
    types::ConfigDto,
    db::ConfigEntity,
    repo::ConfigRepository
};

#[derive(Clone)]
pub struct ConfigService {
    repo: ConfigRepository,
}

impl ConfigService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            repo: ConfigRepository::new(pool),
        }
    }

    pub async fn get(&self) -> sqlx::Result<ConfigEntity> {
        self.repo.get_config().await
    }

    pub async fn update(&self, cfg: &ConfigEntity) -> sqlx::Result<()> {
        self.repo.update_config(cfg).await
    }
}