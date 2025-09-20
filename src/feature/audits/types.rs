use super::db::{AuditEvent, EventType, LogLevel};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Same shape as the entity (as requested)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditEventDto {
    pub id: Uuid,
    pub user_id: i64,
    pub event_type: EventType,
    pub log_level: LogLevel,
    pub session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<AuditEvent> for AuditEventDto {
    fn from(e: AuditEvent) -> Self {
        Self {
            id: e.id,
            user_id: e.user_id,
            event_type: e.event_type,
            log_level: e.log_level,
            session_id: e.session_id,
            created_at: e.created_at,
        }
    }
}

impl From<AuditEventDto> for AuditEvent {
    fn from(d: AuditEventDto) -> Self {
        Self {
            id: d.id,
            user_id: d.user_id,
            event_type: d.event_type,
            log_level: d.log_level,
            session_id: d.session_id,
            created_at: d.created_at,
        }
    }
}
