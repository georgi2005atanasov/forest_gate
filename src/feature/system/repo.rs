use super::ConfigEntity;
use sqlx::PgPool;

#[derive(Clone)]
pub struct ConfigRepository {
    pub pool: PgPool,
}

impl ConfigRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_config(&self) -> sqlx::Result<ConfigEntity> {
        sqlx::query_as::<_, ConfigEntity>("SELECT * FROM config LIMIT 1")
            .fetch_one(&self.pool)
            .await
    }

    pub async fn update_config(&self, cfg: &ConfigEntity) -> sqlx::Result<()> {
        sqlx::query(
            r#"
                UPDATE config
                SET allow_recovery_codes = $1,
                    allow_refresh_tokens = $2,
                    token_validity_seconds = $3,
                    refresh_token_validity_seconds = $4,
                    ai_model = $5,
                    vector_similarity_threshold = $6
            "#,
        )
        .bind(cfg.allow_recovery_codes)
        .bind(cfg.allow_refresh_tokens)
        .bind(cfg.token_validity_seconds)
        .bind(cfg.refresh_token_validity_seconds)
        .bind(&cfg.ai_model)
        .bind(cfg.vector_similarity_threshold)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
