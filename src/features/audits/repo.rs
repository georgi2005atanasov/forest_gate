// use super::{AuditEvent, EventType, LogLevel};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuditRepository {
    pub pool: PgPool,
}

impl AuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // pub async fn create(
    //     &self,
    //     user_id: i64,
    //     event_type: EventType,
    //     log_level: LogLevel,
    //     session_id: Option<Uuid>,
    // ) -> sqlx::Result<AuditEvent> {
    //     let rec = sqlx::query_as::<_, AuditEvent>(
    //         r#"
    //         INSERT INTO audit_events (user_id, event_type, log_level, session_id)
    //         VALUES ($1, $2, $3, $4)
    //         RETURNING id, user_id, event_type, log_level, session_id, created_at
    //         "#,
    //     )
    //     .bind(user_id)
    //     .bind(event_type)
    //     .bind(log_level)
    //     .bind(session_id)
    //     .fetch_one(&self.pool)
    //     .await?;

    //     Ok(rec)
    // }

    // pub async fn get_by_id(&self, id: Uuid) -> sqlx::Result<AuditEvent> {
    //     sqlx::query_as::<_, AuditEvent>(
    //         r#"
    //         SELECT id, user_id, event_type, log_level, session_id, created_at
    //         FROM audit_events
    //         WHERE id = $1
    //         "#,
    //     )
    //     .bind(id)
    //     .fetch_one(&self.pool)
    //     .await
    // }

    // pub async fn list_recent(&self, limit: i64) -> sqlx::Result<Vec<AuditEvent>> {
    //     sqlx::query_as::<_, AuditEvent>(
    //         r#"
    //         SELECT id, user_id, event_type, log_level, session_id, created_at
    //         FROM audit_events
    //         ORDER BY created_at DESC
    //         LIMIT $1
    //         "#,
    //     )
    //     .bind(limit)
    //     .fetch_all(&self.pool)
    //     .await
    // }

    // pub async fn list_by_user(&self, user_id: i64, limit: i64) -> sqlx::Result<Vec<AuditEvent>> {
    //     sqlx::query_as::<_, AuditEvent>(
    //         r#"
    //         SELECT id, user_id, event_type, log_level, session_id, created_at
    //         FROM audit_events
    //         WHERE user_id = $1
    //         ORDER BY created_at DESC
    //         LIMIT $2
    //         "#,
    //     )
    //     .bind(user_id)
    //     .bind(limit)
    //     .fetch_all(&self.pool)
    //     .await
    // }
}
