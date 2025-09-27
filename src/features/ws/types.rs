use actix::prelude::*;
use actix_web::rt::spawn;
use actix_web_actors::ws;
use deadpool_redis::redis::{AsyncCommands, RedisResult}; // <- use re-exported traits
use deadpool_redis::Pool as RedisPool;
use serde_json::Value as JsonValue;
use std::time::Duration;

pub(super) struct SessionWs {
    pub(super) user_id: String,
    pub(super) device_id: String,
    pub(super) session_id: String,
    pub(super) redis: RedisPool,
    pub(super) flush_handle: Option<SpawnHandle>,
}

impl SessionWs {
    pub(super) fn new(
        user_id: String,
        device_id: String,
        session_id: String,
        redis: RedisPool,
    ) -> Self {
        Self {
            user_id,
            device_id,
            session_id,
            redis,
            flush_handle: None,
        }
    }
    fn buf_key(&self) -> String {
        format!(
            "ui:buffer:{}:{}:{}",
            self.user_id, self.device_id, self.session_id
        )
    }
    pub(super) fn jobs_key() -> &'static str {
        "ui:flush_jobs"
    }

    fn schedule_inactivity_flush(&mut self, ctx: &mut <Self as Actor>::Context) {
        if let Some(h) = self.flush_handle.take() {
            ctx.cancel_future(h);
        }
        let uid = self.user_id.clone();
        let did = self.device_id.clone();
        let sid = self.session_id.clone();
        let pool = self.redis.clone();

        let handle = ctx.run_later(Duration::from_secs(120), move |_act, _ctx| {
            // enqueue a flush job
            let job = serde_json::json!({ "user_id": uid, "device_id": did, "session_id": sid })
                .to_string();
            spawn(async move {
                if let Ok(mut conn) = pool.get().await {
                    // RPUSH jobs list
                    let _: RedisResult<()> = conn.rpush(SessionWs::jobs_key(), job).await;
                }
            });
        });
        self.flush_handle = Some(handle);
    }
}

impl Actor for SessionWs {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.schedule_inactivity_flush(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SessionWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let text = text.to_string();
                let event: JsonValue = serde_json::from_str(&text)
                    .unwrap_or_else(|_| serde_json::json!({ "raw": text }));
                let payload = serde_json::json!({
                    "ts": chrono::Utc::now().timestamp_millis(),
                    "event": event
                })
                .to_string();

                ctx.text(r#"{"status":"stored"}"#);

                let key = self.buf_key();
                let pool = self.redis.clone();
                spawn(async move {
                    if let Ok(mut conn) = pool.get().await {
                        // RPUSH event into per-session buffer
                        let _ = conn.rpush::<_, _, ()>(key, payload).await;
                    }
                });

                self.schedule_inactivity_flush(ctx);
            }
            Ok(ws::Message::Ping(bytes)) => ctx.pong(&bytes),
            Ok(ws::Message::Close(_)) => ctx.stop(),
            _ => {}
        }
    }
}
