use crate::utils::error::{Error, Result};
use deadpool_redis::redis::{self, aio::PubSub, AsyncCommands};
use deadpool_redis::Pool;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone)]
pub struct AuditService {
    redis_pool: Pool,
}

impl AuditService {
    pub fn new(redis_pool: Pool) -> Self {
        Self { redis_pool }
    }

    /// Append events and refresh the inactivity timer (60s).
    pub async fn append_events(&self, interaction_id: &str, events: &[String]) -> Result<()> {
        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| Error::Unexpected(format!("redis pool error: {e}")))?;

        let events_key = events_key(interaction_id);
        let timer_key = timer_key(interaction_id);

        let mut pipe = redis::pipe();
        pipe.atomic()
            .cmd("RPUSH")
            .arg(&events_key)
            .arg(events)
            .ignore()
            .cmd("SET")
            .arg(&timer_key)
            .arg("1")
            .arg("EX")
            .arg(60)
            .ignore();

        pipe.query_async(&mut *conn)
            .await
            .map_err(|e| Error::Unexpected(format!("redis pipeline error: {e}")))?;

        Ok(())
    }

    /// Start a background worker that listens for expired timer keys.
    /// When a timer key expires, we read & log the events and then delete them.
    ///
    /// `redis_url` should be the same Redis instance as the pool (e.g. from REDIS_URL).
    pub fn spawn_inactivity_flusher(&self, redis_url: &str) {
        let redis_url = redis_url.to_owned();
        tokio::spawn(async move {
            if let Err(e) = run_flusher(redis_url).await {
                tracing::error!("audit flusher stopped with error: {e:?}");
            }
        });
    }
}

fn events_key(interaction_id: &str) -> String {
    format!("audit:session:{interaction_id}:events")
}

fn timer_key(interaction_id: &str) -> String {
    format!("audit:session:{interaction_id}:timer")
}

fn parse_interaction_id_from_timer_key(timer_key: &str) -> Option<String> {
    // expecting: audit:session:{id}:timer
    let parts: Vec<&str> = timer_key.split(':').collect();
    if parts.len() == 4 && parts[0] == "audit" && parts[1] == "session" && parts[3] == "timer" {
        Some(parts[2].to_string())
    } else {
        None
    }
}

async fn run_flusher(redis_url: String) -> Result<()> {
    let client = redis::Client::open(redis_url.clone())
        .map_err(|e| Error::Unexpected(format!("redis client error: {e}")))?;

    // PubSub connection (dedicated)
    let mut pubsub: PubSub = client
        .get_async_pubsub()
        .await
        .map_err(|e| Error::Unexpected(format!("redis pubsub conn error: {e}")))?;

    // Subscribe to key expiration events
    pubsub
        .psubscribe("__keyevent@*__:expired")
        .await
        .map_err(|e| Error::Unexpected(format!("psubscribe error: {e}")))?;

    // Work connection for reads/deletes
    let mut work_conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| Error::Unexpected(format!("redis work conn error: {e}")))?;

    // Stream messages
    let mut stream = pubsub.on_message();

    while let Some(msg) = stream.next().await {
        // Redis key that expired
        let expired_key: String = msg
            .get_payload()
            .map_err(|e| Error::Unexpected(format!("payload error: {e}")))?;

        if let Some(interaction_id) = parse_interaction_id_from_timer_key(&expired_key) {
            let ev_key = events_key(&interaction_id);

            let events: Vec<String> = work_conn.lrange(&ev_key, 0, -1).await.unwrap_or_default();
            let _: () = redis::cmd("DEL")
                .arg(&ev_key)
                .query_async(&mut work_conn)
                .await
                .unwrap_or(());

            if !events.is_empty() {
                tracing::info!(target: "audit", "FLUSH interaction_id={} events={:?}", interaction_id, events);
            }
        }
    }

    Ok(())
}
