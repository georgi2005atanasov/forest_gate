use super::types::SessionWs;
use deadpool_redis::redis::AsyncCommands; // re-export
use deadpool_redis::Pool as RedisPool;
use std::time::Duration;

pub async fn flush_worker(pool: RedisPool) {
    loop {
        let job_json: Option<String> = {
            let mut conn = match pool.get().await {
                Ok(c) => c,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };
            match conn
                .brpop::<_, (String, String)>(SessionWs::jobs_key(), 0.0)
                .await
            {
                Ok((_key, payload)) => Some(payload),
                Err(_) => None,
            }
        };

        if let Some(payload) = job_json {
            if let Ok(job) = serde_json::from_str::<serde_json::Value>(&payload) {
                let u = job["user_id"].as_str().unwrap_or_default();
                let d = job["device_id"].as_str().unwrap_or_default();
                let s = job["session_id"].as_str().unwrap_or_default();
                let buf = format!("ui:buffer:{}:{}:{}", u, d, s);

                if let Ok(mut conn) = pool.get().await {
                    // Read all events
                    let events: Vec<String> = conn.lrange(&buf, 0, -1).await.unwrap_or_default();
                    // Clear buffer
                    let _: Result<(), _> = conn.del(&buf).await;

                    if !events.is_empty() {
                        let actions_json = format!("[{}]", events.join(","));
                        // TODO: insert into Postgres here
                        println!(
                            "FLUSH -> user:{u} device:{d} session:{s} events: {}",
                            events.len()
                        );
                        // insert_user_interactions(u, d, s, actions_json).await?;
                    }
                }
            }
        }
    }
}
