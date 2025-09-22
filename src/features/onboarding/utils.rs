use std::net::IpAddr;

use actix_web::HttpRequest;

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

pub fn parse_ip(s: &str) -> Option<IpAddr> {
    s.trim().parse::<IpAddr>().ok()
}

pub fn get_client_ip(req: &HttpRequest) -> Option<IpAddr> {
    // Prefer Forwarded: for=...
    if let Some(h) = req.headers().get("forwarded") {
        if let Ok(v) = h.to_str() {
            for part in v.split(';').flat_map(|x| x.split(',')) {
                let p = part.trim();
                if let Some(val) = p.strip_prefix("for=") {
                    let val = val.trim_matches('"');
                    if let Some(ip) = parse_ip(val) {
                        return Some(ip);
                    }
                }
            }
        }
    }
    if let Some(h) = req.headers().get("x-forwarded-for") {
        if let Ok(v) = h.to_str() {
            if let Some(first) = v.split(',').next() {
                if let Some(ip) = parse_ip(first) {
                    return Some(ip);
                }
            }
        }
    }
    req.peer_addr().map(|p| p.ip())
}

pub fn ip_to_bucket(ip: &IpAddr) -> String {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            format!("{}.{}.{}.0/24", o[0], o[1], o[2])
        }
        IpAddr::V6(v6) => {
            let s = v6.segments();
            format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
        }
    }
}

pub fn sha256_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(input.as_bytes());
    hex::encode(h.finalize())
}
