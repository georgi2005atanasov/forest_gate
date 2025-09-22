use deadpool_redis::{
    redis::{self, RedisError},
    Pool,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct RateLimiter {
    pool: Pool,
}

impl RateLimiter {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn hit(&self, key: &str, limit: u32, window_ms: u64) -> Result<u64, RedisError> {
        const LUA: &str = r#"
            redis.call("ZREMRANGEBYSCORE", KEYS[1], 0, ARGV[1] - ARGV[2])
            local count = redis.call("ZCARD", KEYS[1])
            if count >= tonumber(ARGV[3]) then
              return count
            end
            redis.call("ZADD", KEYS[1], ARGV[1], ARGV[1])
            redis.call("EXPIRE", KEYS[1], ARGV[4])
            return count + 1
        "#;

        let mut conn = self.pool.get().await.map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::IoError,
                "deadpool get failed",
                e.to_string(),
            ))
        })?;

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let ttl_s = (window_ms / 1000).max(1);

        let count: i64 = redis::cmd("EVAL")
            .arg(LUA) // the script
            .arg(1) // number of keys
            .arg(key) // KEYS[1]
            .arg(now_ms) // ARGV[1]
            .arg(window_ms) // ARGV[2]
            .arg(limit) // ARGV[3]
            .arg(ttl_s) // ARGV[4]
            .query_async(&mut *conn)
            .await?;

        Ok(count as u64)
    }
}
