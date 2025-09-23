use deadpool_redis::Pool;
use sqlx::PgPool;

use crate::utils::error::Result;

pub struct UserService {
    pool: PgPool,
    redis_pool: Pool,
}

impl UserService {
    pub fn new(pool: PgPool, redis_pool: Pool) -> Self {
        Self { pool, redis_pool }
    }

    pub async fn login() -> Result<()> {
        return Ok(());
    }
}
