use crate::features::clients::{OpenRouterClient, OrMessage};
use crate::utils::error::{Error, Result};
use deadpool_redis::redis::{self, aio::PubSub, AsyncCommands};
use deadpool_redis::Pool;
use futures::StreamExt;
use std::path::Path;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;

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

    /// Start a background worker that listens for expired timer keys and flushes summaries.
    ///
    /// `redis_url` should be the same instance as your pool.
    pub fn spawn_inactivity_flusher(
        &self,
        redis_url: &str,
        pg_url: &str, // unused for now
        openrouter_client: OpenRouterClient, // pass owned; moved into task
    ) {
        let redis_url = redis_url.to_owned();
        let pg_url = pg_url.to_owned();
        tokio::spawn(async move {
            if let Err(e) = run_flusher(redis_url, pg_url, openrouter_client).await {
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

async fn run_flusher(
    redis_url: String,
    _pg_url: String, // not used now
    openrouter_client: OpenRouterClient,
) -> Result<()> {
    let redis_client = redis::Client::open(redis_url.clone())
        .map_err(|e| Error::Unexpected(format!("redis client error: {e}")))?;

    // PubSub connection (dedicated)
    let mut pubsub: PubSub = redis_client
        .get_async_pubsub()
        .await
        .map_err(|e| Error::Unexpected(format!("redis pubsub conn error: {e}")))?;

    // Subscribe to key expiration events
    pubsub
        .psubscribe("__keyevent@*__:expired")
        .await
        .map_err(|e| Error::Unexpected(format!("psubscribe error: {e}")))?;

    // Work connection for reads/deletes
    let mut work_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| Error::Unexpected(format!("redis work conn error: {e}")))?;

    // Ensure interactions folder exists
    fs::create_dir_all("interactions")
        .await
        .map_err(|e| Error::Unexpected(format!("create_dir interactions failed: {e}")))?;

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

            if events.is_empty() {
                continue;
            }

            // 1) Ask the LLM for a clean, fluent summary.
            let summary = match summarize_events(&openrouter_client, &interaction_id, &events).await
            {
                Ok(s) => s,
                Err(err) => {
                    tracing::warn!("summary via OpenRouter failed: {err:?}; falling back to raw list");
                    fallback_summary(&events)
                }
            };

            // 2) Write/append a markdown entry to ./interactions/{interaction_id}.md
            if let Err(e) = write_interaction_markdown(&interaction_id, &events, &summary).await {
                tracing::error!("writing markdown failed for {interaction_id}: {e:?}");
            } else {
                tracing::info!(target: "audit", "FLUSHED {interaction_id} -> interactions/{interaction_id}.md");
            }
        }
    }

    Ok(())
}

/// Compose messages and call OpenRouter to get a short summary.
async fn summarize_events(
    client: &OpenRouterClient,
    interaction_id: &str,
    events: &[String],
) -> Result<String> {
    // Keep it compact to avoid token limits if the list is huge
    let (prefix, list) = if events.len() > 200 {
        ("(truncated to first 200 events)", &events[..200])
    } else {
        ("", events)
    };

    let events_block = list.join("\n- ");

    // === System & user instructions (simple, fluent English) ===
    let system = OrMessage {
        role: "system".into(),
        content: r#"You write short, fluent summaries of user interactions for internal audit logs.
- Use simple, clear English (B1–B2 level).
- Past tense; 3–6 sentences total.
- Group similar actions; avoid duplicates and noise.
- Infer the user's goal when clear, but do not invent facts.
- If the user asked questions, include them as: The user asked: "…".
- Do NOT include IDs or internal metadata in the prose."#.into(),
    };

    let user_content = format!(
        "Create a brief summary for interaction_id: {interaction_id} {prefix}\nEvents:\n- {events_block}"
    );
    let user = OrMessage {
        role: "user".into(),
        content: user_content,
    };

    // Use default model from env (OPENROUTER_MODEL) if you like; otherwise pass Some("model")
    let text = client
        .chat_text(&[system, user], None, Some(0.2), Some(300))
        .await?;

    Ok(text)
}

/// If LLM fails, write a basic fallback.
fn fallback_summary(events: &[String]) -> String {
    // Very simple: first and last actions + count
    let count = events.len();
    let first = events.first().cloned().unwrap_or_default();
    let last = events.last().cloned().unwrap_or_default();
    format!(
        "User performed {count} actions. They started with: \"{first}\" and later: \"{last}\". See the list below for details."
    )
}

/// Append a markdown section for this flush.
async fn write_interaction_markdown(
    interaction_id: &str,
    events: &[String],
    summary: &str,
) -> Result<()> {
    let path = format!("interactions/{interaction_id}.md");
    let file_exists = Path::new(&path).exists();

    // Build markdown block
    let ts = chrono::Utc::now().to_rfc3339();

    // Events as bullets
    let events_md = if events.is_empty() {
        "- (no events)".to_string()
    } else {
        events.iter().map(|e| format!("- {}", e)).collect::<Vec<_>>().join("\n")
    };

    let mut block = String::new();

    if !file_exists {
        // New file header
        block.push_str(&format!("# Interaction {interaction_id}\n\n"));
    }

    block.push_str(&format!(
        "## Flush at {ts}\n\n### Summary\n{}\n\n### Events\n{}\n\n---\n\n",
        summary.trim(),
        events_md
    ));

    // Ensure file and append
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .map_err(|e| Error::Unexpected(format!("open {path} failed: {e}")))?;

    file.write_all(block.as_bytes())
        .await
        .map_err(|e| Error::Unexpected(format!("write {path} failed: {e}")))?;

    file.flush()
        .await
        .map_err(|e| Error::Unexpected(format!("flush {path} failed: {e}")))?;

    Ok(())
}
